use super::SeaOrmStorage;
use crate::entity::users::{ActiveModel, Column, Entity as Users};
use crate::errors::{HWSystemError, Result};
use crate::models::{
    PaginationInfo,
    users::{
        entities::{User, UserRole, UserStatus},
        requests::{CreateUserRequest, UpdateUserRequest, UserListQuery},
        responses::{UserListResponse, UserStatsResponse},
    },
};
use crate::utils::escape_like_pattern;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};

impl SeaOrmStorage {
    /// 创建用户
    pub async fn create_user_impl(&self, req: CreateUserRequest) -> Result<User> {
        let now = chrono::Utc::now().timestamp();

        let model = ActiveModel {
            username: Set(req.username),
            email: Set(req.email),
            password_hash: Set(req.password),
            role: Set(req.role.to_string()),
            status: Set(UserStatus::Active.to_string()),
            display_name: Set(req.display_name),
            avatar_url: Set(req.avatar_url),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        let result = model
            .insert(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("创建用户失败: {e}")))?;

        Ok(result.into_user())
    }

    /// 通过 ID 获取用户
    pub async fn get_user_by_id_impl(&self, id: i64) -> Result<Option<User>> {
        let result = Users::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询用户失败: {e}")))?;

        Ok(result.map(|m| m.into_user()))
    }

    /// 通过用户名获取用户
    pub async fn get_user_by_username_impl(&self, username: &str) -> Result<Option<User>> {
        let result = Users::find()
            .filter(Column::Username.eq(username))
            .one(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询用户失败: {e}")))?;

        Ok(result.map(|m| m.into_user()))
    }

    /// 通过邮箱获取用户
    pub async fn get_user_by_email_impl(&self, email: &str) -> Result<Option<User>> {
        let result = Users::find()
            .filter(Column::Email.eq(email))
            .one(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询用户失败: {e}")))?;

        Ok(result.map(|m| m.into_user()))
    }

    /// 通过用户名或邮箱获取用户
    pub async fn get_user_by_username_or_email_impl(
        &self,
        identifier: &str,
    ) -> Result<Option<User>> {
        let result = Users::find()
            .filter(
                Condition::any()
                    .add(Column::Username.eq(identifier))
                    .add(Column::Email.eq(identifier)),
            )
            .one(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询用户失败: {e}")))?;

        Ok(result.map(|m| m.into_user()))
    }

    /// 分页列出用户
    pub async fn list_users_with_pagination_impl(
        &self,
        query: UserListQuery,
    ) -> Result<UserListResponse> {
        let (page, page_size) = query.pagination.normalized();

        let mut select = Users::find();

        // 搜索条件
        if let Some(ref search) = query.search
            && !search.trim().is_empty()
        {
            let escaped = escape_like_pattern(search.trim());
            select = select.filter(
                Condition::any()
                    .add(Column::Username.contains(&escaped))
                    .add(Column::Email.contains(&escaped))
                    .add(Column::DisplayName.contains(&escaped)),
            );
        }

        // 角色筛选
        if let Some(ref role) = query.role {
            select = select.filter(Column::Role.eq(role.to_string()));
        }

        // 状态筛选
        if let Some(ref status) = query.status {
            select = select.filter(Column::Status.eq(status.to_string()));
        }

        // 排序
        select = select.order_by_desc(Column::CreatedAt);

        // 分页查询
        let paginator = select.paginate(&self.db, page_size);
        let total = paginator
            .num_items()
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询用户总数失败: {e}")))?;

        let pages = paginator
            .num_pages()
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询用户页数失败: {e}")))?;

        let users = paginator
            .fetch_page(page - 1)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询用户列表失败: {e}")))?;

        Ok(UserListResponse {
            items: users.into_iter().map(|m| m.into_user()).collect(),
            pagination: PaginationInfo {
                page: page as i64,
                page_size: page_size as i64,
                total: total as i64,
                total_pages: pages as i64,
            },
        })
    }

    /// 更新用户最后登录时间
    pub async fn update_last_login_impl(&self, id: i64) -> Result<bool> {
        let now = chrono::Utc::now().timestamp();

        let result = Users::update_many()
            .col_expr(Column::LastLogin, sea_orm::sea_query::Expr::value(now))
            .filter(Column::Id.eq(id))
            .exec(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("更新最后登录时间失败: {e}")))?;

        Ok(result.rows_affected > 0)
    }

    /// 更新用户信息
    pub async fn update_user_impl(
        &self,
        id: i64,
        update: UpdateUserRequest,
    ) -> Result<Option<User>> {
        // 先检查用户是否存在
        let existing = self.get_user_by_id_impl(id).await?;
        if existing.is_none() {
            return Ok(None);
        }

        let now = chrono::Utc::now().timestamp();

        let mut model = ActiveModel {
            id: Set(id),
            updated_at: Set(now),
            ..Default::default()
        };

        if let Some(email) = update.email {
            model.email = Set(email);
        }

        if let Some(password) = update.password {
            model.password_hash = Set(password);
        }

        if let Some(role) = update.role {
            model.role = Set(role.to_string());
        }

        if let Some(status) = update.status {
            model.status = Set(status.to_string());
        }

        if let Some(display_name) = update.display_name {
            model.display_name = Set(Some(display_name));
        }

        if let Some(avatar_url) = update.avatar_url {
            model.avatar_url = Set(Some(avatar_url));
        }

        model
            .update(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("更新用户失败: {e}")))?;

        self.get_user_by_id_impl(id).await
    }

    /// 删除用户
    pub async fn delete_user_impl(&self, id: i64) -> Result<bool> {
        let result = Users::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("删除用户失败: {e}")))?;

        Ok(result.rows_affected > 0)
    }

    /// 统计用户数量
    pub async fn count_users_impl(&self) -> Result<u64> {
        let count = Users::find()
            .count(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("统计用户数量失败: {e}")))?;

        Ok(count)
    }

    /// 批量检查用户名是否已存在，返回已存在的用户名列表
    pub async fn check_usernames_exist_impl(&self, usernames: &[String]) -> Result<Vec<String>> {
        if usernames.is_empty() {
            return Ok(vec![]);
        }

        let existing = Users::find()
            .filter(Column::Username.is_in(usernames.iter().cloned()))
            .all(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("检查用户名失败: {e}")))?;

        Ok(existing.into_iter().map(|u| u.username).collect())
    }

    /// 批量检查邮箱是否已存在，返回已存在的邮箱列表
    pub async fn check_emails_exist_impl(&self, emails: &[String]) -> Result<Vec<String>> {
        if emails.is_empty() {
            return Ok(vec![]);
        }

        let existing = Users::find()
            .filter(Column::Email.is_in(emails.iter().cloned()))
            .all(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("检查邮箱失败: {e}")))?;

        Ok(existing.into_iter().map(|u| u.email).collect())
    }

    /// 列出所有用户（用于导出，限制数量）
    pub async fn list_all_users_for_export_impl(&self, limit: u64) -> Result<Vec<User>> {
        use crate::entity::users::Model;
        let users: Vec<Model> = Users::find()
            .order_by_asc(Column::Id)
            .limit(limit)
            .all(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询用户失败: {e}")))?;

        Ok(users.into_iter().map(|m| m.into_user()).collect())
    }

    /// 列出用户（用于导出，支持筛选）
    pub async fn list_users_for_export_filtered_impl(
        &self,
        limit: u64,
        role: Option<UserRole>,
        status: Option<UserStatus>,
        search: Option<&str>,
    ) -> Result<Vec<User>> {
        let mut select = Users::find();

        // 搜索条件
        if let Some(search) = search
            && !search.trim().is_empty()
        {
            let escaped = escape_like_pattern(search.trim());
            select = select.filter(
                Condition::any()
                    .add(Column::Username.contains(&escaped))
                    .add(Column::Email.contains(&escaped))
                    .add(Column::DisplayName.contains(&escaped)),
            );
        }

        // 角色筛选
        if let Some(ref role) = role {
            select = select.filter(Column::Role.eq(role.to_string()));
        }

        // 状态筛选
        if let Some(ref status) = status {
            select = select.filter(Column::Status.eq(status.to_string()));
        }

        let users = select
            .order_by_asc(Column::Id)
            .limit(limit)
            .all(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询用户失败: {e}")))?;

        Ok(users.into_iter().map(|m| m.into_user()).collect())
    }

    /// 获取用户综合统计
    pub async fn get_user_stats_impl(
        &self,
        user_id: i64,
        role: UserRole,
    ) -> Result<UserStatsResponse> {
        use crate::entity::class_users::{Column as ClassUserColumn, Entity as ClassUsers};
        use crate::entity::classes::{Column as ClassColumn, Entity as Classes};

        let now = chrono::Utc::now();
        let is_teacher = matches!(role, UserRole::Teacher | UserRole::Admin);

        // 1. 获取班级数量和学生总数
        let (class_count, total_students) = if is_teacher {
            // 教师：获取管理的班级
            let class_users = ClassUsers::find()
                .filter(ClassUserColumn::UserId.eq(user_id))
                .filter(ClassUserColumn::Role.eq("teacher"))
                .all(&self.db)
                .await
                .map_err(|e| HWSystemError::database_operation(format!("查询教师班级失败: {e}")))?;

            let mut class_ids: std::collections::HashSet<i64> =
                class_users.iter().map(|cu| cu.class_id).collect();

            // 也查询作为班级创建者的班级
            let owned_classes = Classes::find()
                .filter(ClassColumn::TeacherId.eq(user_id))
                .all(&self.db)
                .await
                .map_err(|e| {
                    HWSystemError::database_operation(format!("查询创建的班级失败: {e}"))
                })?;

            for class in owned_classes {
                class_ids.insert(class.id);
            }

            let class_count = class_ids.len() as i64;

            // 统计所有班级的学生数
            let mut total_students = 0i64;
            for class_id in &class_ids {
                let count = ClassUsers::find()
                    .filter(ClassUserColumn::ClassId.eq(*class_id))
                    .filter(ClassUserColumn::Role.is_in(["student", "class_representative"]))
                    .count(&self.db)
                    .await
                    .map_err(|e| {
                        HWSystemError::database_operation(format!("查询班级学生数失败: {e}"))
                    })? as i64;
                total_students += count;
            }

            (class_count, total_students)
        } else {
            // 学生：获取加入的班级
            let class_count = ClassUsers::find()
                .filter(ClassUserColumn::UserId.eq(user_id))
                .count(&self.db)
                .await
                .map_err(|e| HWSystemError::database_operation(format!("查询用户班级失败: {e}")))?
                as i64;

            (class_count, 0)
        };

        // 2. 获取作业统计
        let (homework_pending, homework_submitted, homework_graded, pending_review) = if is_teacher
        {
            // 教师统计
            let (_, pending_review, _, _) = self.get_teacher_homework_stats_impl(user_id).await?;
            (0, 0, 0, pending_review)
        } else {
            // 学生统计
            let (pending, submitted, graded, _) = self.get_my_homework_stats_impl(user_id).await?;
            (pending, submitted, graded, 0)
        };

        Ok(UserStatsResponse {
            class_count,
            total_students,
            homework_pending,
            homework_submitted,
            homework_graded,
            pending_review,
            server_time: now.to_rfc3339(),
        })
    }
}
