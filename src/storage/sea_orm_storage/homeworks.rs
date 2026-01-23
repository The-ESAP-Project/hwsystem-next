//! 作业存储操作

use super::SeaOrmStorage;
use crate::entity::homework_files::{
    ActiveModel as HomeworkFileActiveModel, Column as HomeworkFileColumn, Entity as HomeworkFiles,
};
use crate::entity::homeworks::{ActiveModel, Column, Entity as Homeworks};
use crate::errors::{HWSystemError, Result};
use crate::models::{
    PaginationInfo,
    homeworks::{
        entities::Homework,
        requests::{CreateHomeworkRequest, HomeworkListQuery, UpdateHomeworkRequest},
        responses::HomeworkListResponse,
    },
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Set,
};

impl SeaOrmStorage {
    /// 创建作业
    pub async fn create_homework_impl(
        &self,
        created_by: i64,
        req: CreateHomeworkRequest,
    ) -> Result<Homework> {
        let now = chrono::Utc::now().timestamp();

        let model = ActiveModel {
            class_id: Set(req.class_id),
            title: Set(req.title),
            description: Set(req.description),
            max_score: Set(req.max_score.unwrap_or(100.0)),
            deadline: Set(req.deadline),
            allow_late: Set(req.allow_late.unwrap_or(false)),
            created_by: Set(created_by),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        let result = model
            .insert(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("创建作业失败: {e}")))?;

        // 处理附件
        if let Some(file_ids) = req.attachments {
            self.set_homework_files_impl(result.id, file_ids).await?;
        }

        Ok(result.into_homework())
    }

    /// 通过 ID 获取作业
    pub async fn get_homework_by_id_impl(&self, homework_id: i64) -> Result<Option<Homework>> {
        let result = Homeworks::find_by_id(homework_id)
            .one(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询作业失败: {e}")))?;

        Ok(result.map(|m| m.into_homework()))
    }

    /// 分页列出作业
    pub async fn list_homeworks_with_pagination_impl(
        &self,
        query: HomeworkListQuery,
    ) -> Result<HomeworkListResponse> {
        let page = query.page.unwrap_or(1).max(1) as u64;
        let size = query.size.unwrap_or(10).clamp(1, 100) as u64;

        let mut select = Homeworks::find();

        // 班级筛选
        if let Some(class_id) = query.class_id {
            select = select.filter(Column::ClassId.eq(class_id));
        }

        // 创建者筛选
        if let Some(created_by) = query.created_by {
            select = select.filter(Column::CreatedBy.eq(created_by));
        }

        // 搜索条件（按标题搜索）
        if let Some(ref search) = query.search
            && !search.trim().is_empty()
        {
            select = select.filter(Column::Title.contains(search.trim()));
        }

        // 排序
        select = select.order_by_desc(Column::CreatedAt);

        // 分页查询
        let paginator = select.paginate(&self.db, size);
        let total = paginator
            .num_items()
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询作业总数失败: {e}")))?;

        let pages = paginator
            .num_pages()
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询作业页数失败: {e}")))?;

        let homeworks = paginator
            .fetch_page(page - 1)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询作业列表失败: {e}")))?;

        Ok(HomeworkListResponse {
            items: homeworks.into_iter().map(|m| m.into_homework()).collect(),
            pagination: PaginationInfo {
                page: page as i64,
                size: size as i64,
                total: total as i64,
                pages: pages as i64,
            },
        })
    }

    /// 更新作业
    pub async fn update_homework_impl(
        &self,
        homework_id: i64,
        update: UpdateHomeworkRequest,
    ) -> Result<Option<Homework>> {
        // 先检查作业是否存在
        let existing = self.get_homework_by_id_impl(homework_id).await?;
        if existing.is_none() {
            return Ok(None);
        }

        let now = chrono::Utc::now().timestamp();

        let mut model = ActiveModel {
            id: Set(homework_id),
            updated_at: Set(now),
            ..Default::default()
        };

        if let Some(title) = update.title {
            model.title = Set(title);
        }

        if let Some(description) = update.description {
            model.description = Set(Some(description));
        }

        if let Some(max_score) = update.max_score {
            model.max_score = Set(max_score);
        }

        if let Some(deadline) = update.deadline {
            model.deadline = Set(Some(deadline));
        }

        if let Some(allow_late) = update.allow_late {
            model.allow_late = Set(allow_late);
        }

        model
            .update(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("更新作业失败: {e}")))?;

        // 处理附件
        if let Some(file_ids) = update.attachments {
            self.set_homework_files_impl(homework_id, file_ids).await?;
        }

        self.get_homework_by_id_impl(homework_id).await
    }

    /// 删除作业
    pub async fn delete_homework_impl(&self, homework_id: i64) -> Result<bool> {
        // 先删除附件关联
        HomeworkFiles::delete_many()
            .filter(HomeworkFileColumn::HomeworkId.eq(homework_id))
            .exec(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("删除作业附件关联失败: {e}")))?;

        let result = Homeworks::delete_by_id(homework_id)
            .exec(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("删除作业失败: {e}")))?;

        Ok(result.rows_affected > 0)
    }

    /// 获取作业附件 ID 列表
    pub async fn get_homework_file_ids_impl(&self, homework_id: i64) -> Result<Vec<i64>> {
        let results = HomeworkFiles::find()
            .filter(HomeworkFileColumn::HomeworkId.eq(homework_id))
            .all(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询作业附件失败: {e}")))?;

        Ok(results.into_iter().map(|m| m.file_id).collect())
    }

    /// 设置作业附件
    pub async fn set_homework_files_impl(
        &self,
        homework_id: i64,
        file_ids: Vec<i64>,
    ) -> Result<()> {
        // 先删除旧的关联
        HomeworkFiles::delete_many()
            .filter(HomeworkFileColumn::HomeworkId.eq(homework_id))
            .exec(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("删除旧附件关联失败: {e}")))?;

        // 创建新的关联
        for file_id in file_ids {
            let model = HomeworkFileActiveModel {
                homework_id: Set(homework_id),
                file_id: Set(file_id),
            };

            model
                .insert(&self.db)
                .await
                .map_err(|e| HWSystemError::database_operation(format!("创建附件关联失败: {e}")))?;

            // 增加文件引用计数
            self.increment_file_citation_impl(file_id).await?;
        }

        Ok(())
    }
}
