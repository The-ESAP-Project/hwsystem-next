//! 作业存储操作

use std::collections::HashMap;

use super::SeaOrmStorage;
use crate::entity::class_users::{Column as ClassUserColumn, Entity as ClassUsers};
use crate::entity::grades::{Column as GradeColumn, Entity as Grades};
use crate::entity::homework_files::{
    ActiveModel as HomeworkFileActiveModel, Column as HomeworkFileColumn, Entity as HomeworkFiles,
};
use crate::entity::homeworks::{ActiveModel, Column, Entity as Homeworks};
use crate::entity::submissions::{Column as SubmissionColumn, Entity as Submissions};
use crate::errors::{HWSystemError, Result};
use crate::models::{
    PaginationInfo,
    homeworks::{
        entities::Homework,
        requests::{CreateHomeworkRequest, HomeworkListQuery, UpdateHomeworkRequest},
        responses::{
            HomeworkCreator, HomeworkListItem, HomeworkListResponse, HomeworkStatsSummary,
            MySubmissionSummary,
        },
    },
};
use crate::utils::escape_like_pattern;
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
            deadline: Set(req.deadline.map(|dt| dt.timestamp())),
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
        if let Some(tokens) = req.attachments {
            self.set_homework_files_impl(result.id, tokens, created_by)
                .await?;
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
        current_user_id: Option<i64>,
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
            let escaped = escape_like_pattern(search.trim());
            select = select.filter(Column::Title.contains(&escaped));
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

        let homeworks: Vec<Homework> = paginator
            .fetch_page(page - 1)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询作业列表失败: {e}")))?
            .into_iter()
            .map(|m| m.into_homework())
            .collect();

        // 收集所有 created_by ID 并去重
        let creator_ids: Vec<i64> = homeworks
            .iter()
            .map(|h| h.created_by)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        // 查询创建者信息
        let mut creator_map: HashMap<i64, HomeworkCreator> = HashMap::new();
        for creator_id in creator_ids {
            if let Ok(Some(user)) = self.get_user_by_id_impl(creator_id).await {
                creator_map.insert(
                    creator_id,
                    HomeworkCreator {
                        id: user.id,
                        username: user.username,
                        display_name: user.display_name,
                        avatar_url: user.avatar_url,
                    },
                );
            }
        }

        // 查询当前用户的提交状态（如果提供了 current_user_id）
        let mut my_submission_map: HashMap<i64, MySubmissionSummary> = HashMap::new();
        if let Some(user_id) = current_user_id {
            let homework_ids: Vec<i64> = homeworks.iter().map(|h| h.id).collect();
            if !homework_ids.is_empty() {
                // 查询该用户对这些作业的所有提交
                let submissions = Submissions::find()
                    .filter(SubmissionColumn::HomeworkId.is_in(homework_ids))
                    .filter(SubmissionColumn::CreatorId.eq(user_id))
                    .order_by_desc(SubmissionColumn::Version)
                    .all(&self.db)
                    .await
                    .map_err(|e| {
                        HWSystemError::database_operation(format!("查询用户提交失败: {e}"))
                    })?;

                // 按 homework_id 聚合，取最新版本
                for sub in submissions {
                    my_submission_map.entry(sub.homework_id).or_insert_with(|| {
                        MySubmissionSummary {
                            id: sub.id,
                            version: sub.version,
                            status: sub.status.clone(),
                            is_late: sub.is_late,
                            score: None, // 稍后填充
                        }
                    });
                }

                // 批量查询评分信息
                if !my_submission_map.is_empty() {
                    let submission_ids: Vec<i64> =
                        my_submission_map.values().map(|s| s.id).collect();
                    let grades = Grades::find()
                        .filter(GradeColumn::SubmissionId.is_in(submission_ids))
                        .all(&self.db)
                        .await
                        .map_err(|e| {
                            HWSystemError::database_operation(format!("查询评分失败: {e}"))
                        })?;

                    // 建立 submission_id -> score 的映射
                    let grade_map: HashMap<i64, f64> = grades
                        .into_iter()
                        .map(|g| (g.submission_id, g.score))
                        .collect();

                    // 填充 score 并更新状态为 graded
                    for summary in my_submission_map.values_mut() {
                        if let Some(score) = grade_map.get(&summary.id) {
                            summary.score = Some(*score);
                            summary.status = "graded".to_string();
                        }
                    }
                }
            }
        }

        // 查询统计信息（如果 include_stats=true）
        let mut stats_map: HashMap<i64, HomeworkStatsSummary> = HashMap::new();
        if query.include_stats.unwrap_or(false) && !homeworks.is_empty() {
            let homework_ids: Vec<i64> = homeworks.iter().map(|h| h.id).collect();

            // 获取每个作业所属班级的需要提交作业的人数（学生和课代表，排除教师）
            for hw in &homeworks {
                let total_students = ClassUsers::find()
                    .filter(ClassUserColumn::ClassId.eq(hw.class_id))
                    .filter(ClassUserColumn::Role.is_in(["student", "class_representative"]))
                    .count(&self.db)
                    .await
                    .map_err(|e| {
                        HWSystemError::database_operation(format!("查询班级学生数失败: {e}"))
                    })? as i64;

                stats_map.insert(
                    hw.id,
                    HomeworkStatsSummary {
                        total_students,
                        submitted_count: 0,
                        graded_count: 0,
                    },
                );
            }

            // 查询每个作业的提交人数（按 creator_id 去重）
            let submissions = Submissions::find()
                .filter(SubmissionColumn::HomeworkId.is_in(homework_ids.clone()))
                .all(&self.db)
                .await
                .map_err(|e| HWSystemError::database_operation(format!("查询作业提交失败: {e}")))?;

            // 按 homework_id 聚合，统计唯一提交者
            let mut hw_submitters: HashMap<i64, std::collections::HashSet<i64>> = HashMap::new();
            let mut submission_ids: Vec<i64> = Vec::new();
            for sub in &submissions {
                hw_submitters
                    .entry(sub.homework_id)
                    .or_default()
                    .insert(sub.creator_id);
                submission_ids.push(sub.id);
            }

            for (hw_id, submitters) in hw_submitters {
                if let Some(stats) = stats_map.get_mut(&hw_id) {
                    stats.submitted_count = submitters.len() as i64;
                }
            }

            // 查询已评分的提交数
            if !submission_ids.is_empty() {
                let grades = Grades::find()
                    .filter(GradeColumn::SubmissionId.is_in(submission_ids))
                    .all(&self.db)
                    .await
                    .map_err(|e| HWSystemError::database_operation(format!("查询评分失败: {e}")))?;

                // 建立 submission_id -> homework_id 的映射
                let sub_to_hw: HashMap<i64, i64> =
                    submissions.iter().map(|s| (s.id, s.homework_id)).collect();

                // 按 homework 聚合已评分的唯一用户
                let mut hw_graded_users: HashMap<i64, std::collections::HashSet<i64>> =
                    HashMap::new();
                let sub_to_creator: HashMap<i64, i64> =
                    submissions.iter().map(|s| (s.id, s.creator_id)).collect();

                for grade in grades {
                    if let (Some(&hw_id), Some(&creator_id)) = (
                        sub_to_hw.get(&grade.submission_id),
                        sub_to_creator.get(&grade.submission_id),
                    ) {
                        hw_graded_users.entry(hw_id).or_default().insert(creator_id);
                    }
                }

                for (hw_id, graded_users) in hw_graded_users {
                    if let Some(stats) = stats_map.get_mut(&hw_id) {
                        stats.graded_count = graded_users.len() as i64;
                    }
                }
            }
        }

        // 构造带 creator 和 my_submission 的作业列表
        let items: Vec<HomeworkListItem> = homeworks
            .into_iter()
            .map(|homework| {
                let creator = creator_map.get(&homework.created_by).cloned();
                let my_submission = my_submission_map.get(&homework.id).cloned();
                let stats_summary = stats_map.get(&homework.id).cloned();
                HomeworkListItem {
                    homework,
                    creator,
                    my_submission,
                    stats_summary,
                }
            })
            .collect();

        Ok(HomeworkListResponse {
            items,
            pagination: PaginationInfo {
                page: page as i64,
                page_size: size as i64,
                total: total as i64,
                total_pages: pages as i64,
            },
        })
    }

    /// 更新作业
    pub async fn update_homework_impl(
        &self,
        homework_id: i64,
        update: UpdateHomeworkRequest,
        user_id: i64,
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
            model.deadline = Set(Some(deadline.timestamp()));
        }

        if let Some(allow_late) = update.allow_late {
            model.allow_late = Set(allow_late);
        }

        model
            .update(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("更新作业失败: {e}")))?;

        // 处理附件
        if let Some(tokens) = update.attachments {
            self.set_homework_files_impl(homework_id, tokens, user_id)
                .await?;
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

    /// 设置作业附件（通过 download_token，带所有权校验）
    pub async fn set_homework_files_impl(
        &self,
        homework_id: i64,
        tokens: Vec<String>,
        user_id: i64,
    ) -> Result<()> {
        // 先删除旧的关联
        HomeworkFiles::delete_many()
            .filter(HomeworkFileColumn::HomeworkId.eq(homework_id))
            .exec(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("删除旧附件关联失败: {e}")))?;

        // 通过 token 查找文件并校验所有权
        for token in tokens {
            let file = self
                .get_file_by_token_impl(&token)
                .await?
                .ok_or_else(|| HWSystemError::not_found(format!("文件不存在: {token}")))?;

            // 校验文件所有权
            if file.user_id != Some(user_id) {
                return Err(HWSystemError::authorization(format!(
                    "无权使用此文件: {token}"
                )));
            }

            let model = HomeworkFileActiveModel {
                homework_id: Set(homework_id),
                file_id: Set(file.id),
            };

            model
                .insert(&self.db)
                .await
                .map_err(|e| HWSystemError::database_operation(format!("创建附件关联失败: {e}")))?;

            // 增加文件引用计数
            self.increment_file_citation_impl(file.id).await?;
        }

        Ok(())
    }
}
