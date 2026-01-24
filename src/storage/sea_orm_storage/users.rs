use super::SeaOrmStorage;
use crate::entity::users::{ActiveModel, Column, Entity as Users};
use crate::errors::{HWSystemError, Result};
use crate::models::{
    PaginationInfo,
    users::{
        entities::{User, UserStatus},
        requests::{CreateUserRequest, UpdateUserRequest, UserListQuery},
        responses::UserListResponse,
    },
};
use crate::utils::escape_like_pattern;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    Set,
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
        let page = query.page.unwrap_or(1).max(1) as u64;
        let size = query.size.unwrap_or(10).clamp(1, 100) as u64;

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
        let paginator = select.paginate(&self.db, size);
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
                size: size as i64,
                total: total as i64,
                pages: pages as i64,
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
}
