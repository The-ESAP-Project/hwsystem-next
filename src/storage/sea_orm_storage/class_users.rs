//! 班级用户关联存储操作

use super::SeaOrmStorage;
use crate::entity::class_users::{ActiveModel, Column, Entity as ClassUsers};
use crate::entity::classes::{Column as ClassColumn, Entity as Classes};
use crate::entity::users::{self, Entity as Users};
use crate::errors::{HWSystemError, Result};
use crate::models::{
    PaginationInfo,
    class_users::{
        entities::{ClassUser, ClassUserRole},
        requests::{ClassUserListQuery, UpdateClassUserRequest},
        responses::ClassUserListResponse,
    },
    classes::{entities::Class, requests::ClassListQuery, responses::ClassListResponse},
};
use crate::utils::escape_like_pattern;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    Set,
};

impl SeaOrmStorage {
    /// 获取班级成员数量
    pub async fn count_class_members_impl(&self, class_id: i64) -> Result<i64> {
        let count = ClassUsers::find()
            .filter(Column::ClassId.eq(class_id))
            .count(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询班级成员数量失败: {e}")))?;

        Ok(count as i64)
    }

    /// 加入班级
    pub async fn join_class_impl(
        &self,
        user_id: i64,
        class_id: i64,
        role: ClassUserRole,
    ) -> Result<ClassUser> {
        let now = chrono::Utc::now().timestamp();

        let model = ActiveModel {
            class_id: Set(class_id),
            user_id: Set(user_id),
            role: Set(role.to_string()),
            joined_at: Set(now),
            ..Default::default()
        };

        let result = model
            .insert(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("加入班级失败: {e}")))?;

        Ok(result.into_class_user())
    }

    /// 离开班级
    pub async fn leave_class_impl(&self, user_id: i64, class_id: i64) -> Result<bool> {
        let result = ClassUsers::delete_many()
            .filter(
                Condition::all()
                    .add(Column::UserId.eq(user_id))
                    .add(Column::ClassId.eq(class_id)),
            )
            .exec(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("离开班级失败: {e}")))?;

        Ok(result.rows_affected > 0)
    }

    /// 更新班级用户信息（通过 user_id 和 class_id）
    pub async fn update_class_user_impl(
        &self,
        class_id: i64,
        user_id: i64,
        update: UpdateClassUserRequest,
    ) -> Result<Option<ClassUser>> {
        // 先检查班级用户是否存在
        let existing = ClassUsers::find()
            .filter(Column::UserId.eq(user_id))
            .filter(Column::ClassId.eq(class_id))
            .one(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询班级用户失败: {e}")))?;

        let class_user = match existing {
            Some(cu) => cu,
            None => return Ok(None),
        };

        let mut model = ActiveModel {
            id: Set(class_user.id),
            ..Default::default()
        };

        if let Some(role) = update.role {
            model.role = Set(role.to_string());
        }

        let result = model
            .update(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("更新班级用户失败: {e}")))?;

        Ok(Some(result.into_class_user()))
    }

    /// 分页列出班级用户
    pub async fn list_class_users_with_pagination_impl(
        &self,
        class_id: i64,
        query: ClassUserListQuery,
    ) -> Result<ClassUserListResponse> {
        let (page, page_size) = query.pagination.normalized();

        let mut select = ClassUsers::find()
            .filter(Column::ClassId.eq(class_id))
            .order_by_desc(Column::JoinedAt);

        // role 过滤
        if let Some(ref role) = query.role {
            select = select.filter(Column::Role.eq(role.to_string()));
        }

        // search 过滤 - 查 users 表的 username/display_name
        if let Some(ref search) = query.search
            && !search.trim().is_empty()
        {
            let escaped = escape_like_pattern(search.trim());
            let matching_user_ids: Vec<i64> = Users::find()
                .filter(
                    Condition::any()
                        .add(users::Column::Username.contains(&escaped))
                        .add(users::Column::DisplayName.contains(&escaped)),
                )
                .all(&self.db)
                .await
                .map_err(|e| HWSystemError::database_operation(format!("搜索用户失败: {e}")))?
                .into_iter()
                .map(|u| u.id)
                .collect();

            select = select.filter(Column::UserId.is_in(matching_user_ids));
        }

        // 分页查询
        let paginator = select.paginate(&self.db, page_size);
        let total = paginator
            .num_items()
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询班级用户总数失败: {e}")))?;

        let pages = paginator
            .num_pages()
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询班级用户页数失败: {e}")))?;

        let users = paginator
            .fetch_page(page - 1)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询班级用户列表失败: {e}")))?;

        Ok(ClassUserListResponse {
            items: users.into_iter().map(|m| m.into_class_user()).collect(),
            pagination: PaginationInfo {
                page: page as i64,
                page_size: page_size as i64,
                total: total as i64,
                total_pages: pages as i64,
            },
        })
    }

    /// 分页列出用户所在的班级
    pub async fn list_user_classes_with_pagination_impl(
        &self,
        user_id: i64,
        query: ClassListQuery,
    ) -> Result<ClassListResponse> {
        let (page, page_size) = query.pagination.normalized();

        // 查询用户加入的班级 ID
        let class_user_records = ClassUsers::find()
            .filter(Column::UserId.eq(user_id))
            .all(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询用户班级关联失败: {e}")))?;

        let class_ids: Vec<i64> = class_user_records.iter().map(|cu| cu.class_id).collect();

        if class_ids.is_empty() {
            return Ok(ClassListResponse {
                items: vec![],
                pagination: PaginationInfo {
                    page: page as i64,
                    page_size: page_size as i64,
                    total: 0,
                    total_pages: 0,
                },
            });
        }

        let mut select = Classes::find().filter(ClassColumn::Id.is_in(class_ids));

        // 搜索条件
        if let Some(ref search) = query.search
            && !search.trim().is_empty()
        {
            let escaped = escape_like_pattern(search.trim());
            select = select.filter(ClassColumn::Name.contains(&escaped));
        }

        // 排序
        select = select.order_by_desc(ClassColumn::CreatedAt);

        // 分页查询
        let paginator = select.paginate(&self.db, page_size);
        let total = paginator
            .num_items()
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询用户班级总数失败: {e}")))?;

        let pages = paginator
            .num_pages()
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询用户班级页数失败: {e}")))?;

        let classes = paginator
            .fetch_page(page - 1)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询用户班级列表失败: {e}")))?;

        Ok(ClassListResponse {
            items: classes.into_iter().map(|m| m.into_class()).collect(),
            pagination: PaginationInfo {
                page: page as i64,
                page_size: page_size as i64,
                total: total as i64,
                total_pages: pages as i64,
            },
        })
    }

    /// 获取用户在班级中的信息
    pub async fn get_class_user_by_user_id_and_class_id_impl(
        &self,
        user_id: i64,
        class_id: i64,
    ) -> Result<Option<ClassUser>> {
        let result = ClassUsers::find()
            .filter(
                Condition::all()
                    .add(Column::UserId.eq(user_id))
                    .add(Column::ClassId.eq(class_id)),
            )
            .one(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询班级用户失败: {e}")))?;

        Ok(result.map(|m| m.into_class_user()))
    }

    /// 通过班级用户 ID 获取班级用户信息
    pub async fn get_class_user_by_id_impl(&self, class_user_id: i64) -> Result<Option<ClassUser>> {
        let result = ClassUsers::find_by_id(class_user_id)
            .one(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询班级用户失败: {e}")))?;

        Ok(result.map(|m| m.into_class_user()))
    }

    /// 根据班级 ID 和邀请码获取班级及用户信息
    pub async fn get_class_and_class_user_by_class_id_and_code_impl(
        &self,
        class_id: i64,
        invite_code: &str,
        user_id: i64,
    ) -> Result<(Option<Class>, Option<ClassUser>)> {
        // 获取班级（验证 ID 和邀请码）
        let class = Classes::find_by_id(class_id)
            .filter(ClassColumn::InviteCode.eq(invite_code))
            .one(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询班级失败: {e}")))?
            .map(|m| m.into_class());

        // 获取班级用户
        let class_user = self
            .get_class_user_by_user_id_and_class_id_impl(user_id, class_id)
            .await?;

        Ok((class, class_user))
    }

    /// 获取班级所有成员（不分页，用于内部统计/导出）
    pub async fn list_all_class_users_impl(&self, class_id: i64) -> Result<Vec<ClassUser>> {
        let result = ClassUsers::find()
            .filter(Column::ClassId.eq(class_id))
            .order_by_desc(Column::JoinedAt)
            .all(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询班级用户失败: {e}")))?;

        Ok(result.into_iter().map(|m| m.into_class_user()).collect())
    }
}
