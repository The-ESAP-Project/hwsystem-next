//! 评分存储操作

use std::collections::HashMap;

use super::SeaOrmStorage;
use crate::entity::grades::{ActiveModel, Column, Entity as Grades};
use crate::entity::submissions::Column as SubmissionColumn;
use crate::errors::{HWSystemError, Result};
use crate::models::{
    PaginationInfo,
    grades::{
        entities::Grade,
        requests::{CreateGradeRequest, GradeListQuery, UpdateGradeRequest},
        responses::GradeListResponse,
    },
    submissions::entities::SubmissionStatus,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, JoinType, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, RelationTrait, Set, TransactionTrait,
};

impl SeaOrmStorage {
    /// 创建评分（使用事务保护）
    pub async fn create_grade_impl(
        &self,
        grader_id: i64,
        req: CreateGradeRequest,
    ) -> Result<Grade> {
        let txn = self
            .db
            .begin()
            .await
            .map_err(|e| HWSystemError::database_operation(format!("开启事务失败: {e}")))?;

        let now = chrono::Utc::now().timestamp();

        let model = ActiveModel {
            submission_id: Set(req.submission_id),
            grader_id: Set(grader_id),
            score: Set(req.score),
            comment: Set(req.comment),
            graded_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        let result = model
            .insert(&txn)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("创建评分失败: {e}")))?;

        // 更新提交状态为已评分
        self.update_submission_status_txn(&txn, req.submission_id, SubmissionStatus::GRADED)
            .await?;

        txn.commit()
            .await
            .map_err(|e| HWSystemError::database_operation(format!("提交事务失败: {e}")))?;

        Ok(result.into_grade())
    }

    /// 通过 ID 获取评分
    pub async fn get_grade_by_id_impl(&self, grade_id: i64) -> Result<Option<Grade>> {
        let result = Grades::find_by_id(grade_id)
            .one(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询评分失败: {e}")))?;

        Ok(result.map(|m| m.into_grade()))
    }

    /// 通过提交 ID 获取评分
    pub async fn get_grade_by_submission_id_impl(
        &self,
        submission_id: i64,
    ) -> Result<Option<Grade>> {
        let result = Grades::find()
            .filter(Column::SubmissionId.eq(submission_id))
            .one(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询评分失败: {e}")))?;

        Ok(result.map(|m| m.into_grade()))
    }

    /// 更新评分
    pub async fn update_grade_impl(
        &self,
        grade_id: i64,
        update: UpdateGradeRequest,
    ) -> Result<Option<Grade>> {
        let now = chrono::Utc::now().timestamp();

        let mut model = ActiveModel {
            id: Set(grade_id),
            updated_at: Set(now),
            ..Default::default()
        };

        if let Some(score) = update.score {
            model.score = Set(score);
        }

        if let Some(comment) = update.comment {
            model.comment = Set(Some(comment));
        }

        match model.update(&self.db).await {
            Ok(updated) => Ok(Some(updated.into_grade())),
            Err(e) => {
                // SeaORM 的 RecordNotUpdated 错误表示记录不存在
                if e.to_string().contains("RecordNotUpdated") {
                    Ok(None)
                } else {
                    Err(HWSystemError::database_operation(format!(
                        "更新评分失败: {e}"
                    )))
                }
            }
        }
    }

    /// 列出评分（分页）
    pub async fn list_grades_with_pagination_impl(
        &self,
        query: GradeListQuery,
    ) -> Result<GradeListResponse> {
        let (page, page_size) = query.pagination.normalized();

        let mut select = Grades::find();

        // 如果指定了 homework_id，需要 join submissions 表
        if let Some(homework_id) = query.homework_id {
            select = select
                .join(
                    JoinType::InnerJoin,
                    crate::entity::grades::Relation::Submission.def(),
                )
                .filter(SubmissionColumn::HomeworkId.eq(homework_id));
        }

        // 提交筛选
        if let Some(submission_id) = query.submission_id {
            select = select.filter(Column::SubmissionId.eq(submission_id));
        }

        // 评分者筛选
        if let Some(grader_id) = query.grader_id {
            select = select.filter(Column::GraderId.eq(grader_id));
        }

        // 排序
        select = select.order_by_desc(Column::GradedAt);

        // 分页查询
        let paginator = select.paginate(&self.db, page_size);
        let total = paginator
            .num_items()
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询评分总数失败: {e}")))?;

        let pages = paginator
            .num_pages()
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询评分页数失败: {e}")))?;

        let grades = paginator
            .fetch_page(page - 1)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询评分列表失败: {e}")))?;

        Ok(GradeListResponse {
            items: grades.into_iter().map(|m| m.into_grade()).collect(),
            pagination: PaginationInfo {
                page: page as i64,
                page_size: page_size as i64,
                total: total as i64,
                total_pages: pages as i64,
            },
        })
    }

    /// 批量获取评分（通过提交ID列表）
    pub async fn get_grades_by_submission_ids_impl(
        &self,
        submission_ids: &[i64],
    ) -> Result<HashMap<i64, Grade>> {
        if submission_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let grades = Grades::find()
            .filter(Column::SubmissionId.is_in(submission_ids.to_vec()))
            .all(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("批量查询评分失败: {e}")))?;

        Ok(grades
            .into_iter()
            .map(|g| (g.submission_id, g.into_grade()))
            .collect())
    }
}
