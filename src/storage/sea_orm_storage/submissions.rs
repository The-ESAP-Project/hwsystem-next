//! 提交存储操作

use super::SeaOrmStorage;
use crate::entity::submission_files::{
    ActiveModel as SubmissionFileActiveModel, Column as SubmissionFileColumn,
    Entity as SubmissionFiles,
};
use crate::entity::submissions::{ActiveModel, Column, Entity as Submissions};
use crate::errors::{HWSystemError, Result};
use crate::models::{
    PaginationInfo,
    submissions::{
        entities::{Submission, SubmissionStatus},
        requests::{CreateSubmissionRequest, SubmissionListQuery},
        responses::SubmissionListResponse,
    },
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};

impl SeaOrmStorage {
    /// 创建提交（自动计算版本号）
    pub async fn create_submission_impl(
        &self,
        creator_id: i64,
        req: CreateSubmissionRequest,
    ) -> Result<Submission> {
        let now = chrono::Utc::now().timestamp();

        // 查询当前最大版本号
        let max_version = Submissions::find()
            .filter(Column::HomeworkId.eq(req.homework_id))
            .filter(Column::CreatorId.eq(creator_id))
            .select_only()
            .column_as(Column::Version.max(), "max_version")
            .into_tuple::<Option<i32>>()
            .one(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询最大版本号失败: {e}")))?
            .flatten()
            .unwrap_or(0);

        let version = max_version + 1;

        // 检查是否迟交
        let homework = self.get_homework_by_id_impl(req.homework_id).await?;
        let is_late = if let Some(hw) = homework {
            if let Some(deadline) = hw.deadline {
                chrono::Utc::now() > deadline
            } else {
                false
            }
        } else {
            false
        };

        let status = if is_late {
            SubmissionStatus::Late.to_string()
        } else {
            SubmissionStatus::Pending.to_string()
        };

        let model = ActiveModel {
            homework_id: Set(req.homework_id),
            creator_id: Set(creator_id),
            version: Set(version),
            content: Set(Some(req.content)),
            status: Set(status),
            is_late: Set(is_late),
            submitted_at: Set(now),
            ..Default::default()
        };

        let result = model
            .insert(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("创建提交失败: {e}")))?;

        // 处理附件
        if let Some(attachments) = req.attachments {
            let file_ids: Vec<i64> = attachments
                .iter()
                .filter_map(|s| s.parse::<i64>().ok())
                .collect();
            self.set_submission_files_impl(result.id, file_ids).await?;
        }

        Ok(result.into_submission())
    }

    /// 通过 ID 获取提交
    pub async fn get_submission_by_id_impl(
        &self,
        submission_id: i64,
    ) -> Result<Option<Submission>> {
        let result = Submissions::find_by_id(submission_id)
            .one(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询提交失败: {e}")))?;

        Ok(result.map(|m| m.into_submission()))
    }

    /// 获取学生某作业的最新提交
    pub async fn get_latest_submission_impl(
        &self,
        homework_id: i64,
        creator_id: i64,
    ) -> Result<Option<Submission>> {
        let result = Submissions::find()
            .filter(Column::HomeworkId.eq(homework_id))
            .filter(Column::CreatorId.eq(creator_id))
            .order_by_desc(Column::Version)
            .one(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询最新提交失败: {e}")))?;

        Ok(result.map(|m| m.into_submission()))
    }

    /// 获取学生某作业的提交历史
    pub async fn list_user_submissions_impl(
        &self,
        homework_id: i64,
        creator_id: i64,
    ) -> Result<Vec<Submission>> {
        let results = Submissions::find()
            .filter(Column::HomeworkId.eq(homework_id))
            .filter(Column::CreatorId.eq(creator_id))
            .order_by_desc(Column::Version)
            .all(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询提交历史失败: {e}")))?;

        Ok(results.into_iter().map(|m| m.into_submission()).collect())
    }

    /// 列出提交（分页）
    pub async fn list_submissions_with_pagination_impl(
        &self,
        query: SubmissionListQuery,
    ) -> Result<SubmissionListResponse> {
        let page = query.page.unwrap_or(1).max(1) as u64;
        let size = query.size.unwrap_or(10).clamp(1, 100) as u64;

        let mut select = Submissions::find();

        // 作业筛选
        if let Some(homework_id) = query.homework_id {
            select = select.filter(Column::HomeworkId.eq(homework_id));
        }

        // 提交者筛选
        if let Some(creator_id) = query.creator_id {
            select = select.filter(Column::CreatorId.eq(creator_id));
        }

        // 状态筛选
        if let Some(ref status) = query.status {
            select = select.filter(Column::Status.eq(status));
        }

        // 排序
        select = select.order_by_desc(Column::SubmittedAt);

        // 分页查询
        let paginator = select.paginate(&self.db, size);
        let total = paginator
            .num_items()
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询提交总数失败: {e}")))?;

        let pages = paginator
            .num_pages()
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询提交页数失败: {e}")))?;

        let submissions = paginator
            .fetch_page(page - 1)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询提交列表失败: {e}")))?;

        Ok(SubmissionListResponse {
            items: submissions
                .into_iter()
                .map(|m| m.into_submission())
                .collect(),
            pagination: PaginationInfo {
                page: page as i64,
                size: size as i64,
                total: total as i64,
                pages: pages as i64,
            },
        })
    }

    /// 删除提交（撤回）
    pub async fn delete_submission_impl(&self, submission_id: i64) -> Result<bool> {
        // 先删除附件关联
        SubmissionFiles::delete_many()
            .filter(SubmissionFileColumn::SubmissionId.eq(submission_id))
            .exec(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("删除提交附件关联失败: {e}")))?;

        let result = Submissions::delete_by_id(submission_id)
            .exec(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("删除提交失败: {e}")))?;

        Ok(result.rows_affected > 0)
    }

    /// 更新提交状态
    pub async fn update_submission_status_impl(
        &self,
        submission_id: i64,
        status: &str,
    ) -> Result<bool> {
        let result = Submissions::update_many()
            .col_expr(
                Column::Status,
                sea_orm::sea_query::Expr::value(status.to_string()),
            )
            .filter(Column::Id.eq(submission_id))
            .exec(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("更新提交状态失败: {e}")))?;

        Ok(result.rows_affected > 0)
    }

    /// 获取提交附件 ID 列表
    pub async fn get_submission_file_ids_impl(&self, submission_id: i64) -> Result<Vec<i64>> {
        let results = SubmissionFiles::find()
            .filter(SubmissionFileColumn::SubmissionId.eq(submission_id))
            .all(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询提交附件失败: {e}")))?;

        Ok(results.into_iter().map(|m| m.file_id).collect())
    }

    /// 设置提交附件
    pub async fn set_submission_files_impl(
        &self,
        submission_id: i64,
        file_ids: Vec<i64>,
    ) -> Result<()> {
        // 先删除旧的关联
        SubmissionFiles::delete_many()
            .filter(SubmissionFileColumn::SubmissionId.eq(submission_id))
            .exec(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("删除旧附件关联失败: {e}")))?;

        // 创建新的关联
        for file_id in file_ids {
            let model = SubmissionFileActiveModel {
                submission_id: Set(submission_id),
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
