//! 班级存储操作

use super::SeaOrmStorage;
use crate::entity::classes::{ActiveModel, Column, Entity as Classes};
use crate::errors::{HWSystemError, Result};
use crate::models::{
    PaginationInfo,
    classes::{
        entities::Class,
        requests::{ClassListQuery, CreateClassRequest, UpdateClassRequest},
        responses::ClassListResponse,
    },
};
use crate::utils::random_code::generate_random_code;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Set,
};

impl SeaOrmStorage {
    /// 创建班级
    pub async fn create_class_impl(&self, req: CreateClassRequest) -> Result<Class> {
        let now = chrono::Utc::now().timestamp();
        let invite_code = generate_random_code(8); // 自动生成邀请码

        let model = ActiveModel {
            teacher_id: Set(req.teacher_id),
            name: Set(req.name),
            description: Set(req.description),
            invite_code: Set(invite_code),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        let result = model
            .insert(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("创建班级失败: {e}")))?;

        Ok(result.into_class())
    }

    /// 通过 ID 获取班级
    pub async fn get_class_by_id_impl(&self, class_id: i64) -> Result<Option<Class>> {
        let result = Classes::find_by_id(class_id)
            .one(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询班级失败: {e}")))?;

        Ok(result.map(|m| m.into_class()))
    }

    /// 通过邀请码获取班级
    pub async fn get_class_by_code_impl(&self, invite_code: &str) -> Result<Option<Class>> {
        let result = Classes::find()
            .filter(Column::InviteCode.eq(invite_code))
            .one(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询班级失败: {e}")))?;

        Ok(result.map(|m| m.into_class()))
    }

    /// 分页列出班级
    pub async fn list_classes_with_pagination_impl(
        &self,
        query: ClassListQuery,
    ) -> Result<ClassListResponse> {
        let page = query.page.unwrap_or(1).max(1) as u64;
        let size = query.size.unwrap_or(10).clamp(1, 100) as u64;

        let mut select = Classes::find();

        // 教师筛选
        if let Some(teacher_id) = query.teacher_id {
            select = select.filter(Column::TeacherId.eq(teacher_id));
        }

        // 搜索条件
        if let Some(ref search) = query.search
            && !search.trim().is_empty()
        {
            select = select.filter(Column::Name.contains(search.trim()));
        }

        // 排序
        select = select.order_by_desc(Column::CreatedAt);

        // 分页查询
        let paginator = select.paginate(&self.db, size);
        let total = paginator
            .num_items()
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询班级总数失败: {e}")))?;

        let pages = paginator
            .num_pages()
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询班级页数失败: {e}")))?;

        let classes = paginator
            .fetch_page(page - 1)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询班级列表失败: {e}")))?;

        Ok(ClassListResponse {
            items: classes.into_iter().map(|m| m.into_class()).collect(),
            pagination: PaginationInfo {
                page: page as i64,
                size: size as i64,
                total: total as i64,
                pages: pages as i64,
            },
        })
    }

    /// 更新班级信息
    pub async fn update_class_impl(
        &self,
        class_id: i64,
        update: UpdateClassRequest,
    ) -> Result<Option<Class>> {
        // 先检查班级是否存在
        let existing = self.get_class_by_id_impl(class_id).await?;
        if existing.is_none() {
            return Ok(None);
        }

        let now = chrono::Utc::now().timestamp();

        let mut model = ActiveModel {
            id: Set(class_id),
            updated_at: Set(now),
            ..Default::default()
        };

        if let Some(name) = update.name {
            model.name = Set(name);
        }

        if let Some(description) = update.description {
            model.description = Set(Some(description));
        }

        model
            .update(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("更新班级失败: {e}")))?;

        self.get_class_by_id_impl(class_id).await
    }

    /// 删除班级
    pub async fn delete_class_impl(&self, class_id: i64) -> Result<bool> {
        let result = Classes::delete_by_id(class_id)
            .exec(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("删除班级失败: {e}")))?;

        Ok(result.rows_affected > 0)
    }
}
