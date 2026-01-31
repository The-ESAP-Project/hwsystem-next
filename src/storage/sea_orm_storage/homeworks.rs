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
use crate::entity::users::{Column as UserColumn, Entity as Users};
use crate::errors::{HWSystemError, Result};
use crate::models::{
    PaginationInfo,
    homeworks::{
        entities::{DeadlineFilter, Homework, HomeworkUserStatus},
        requests::{
            AllHomeworksQuery, CreateHomeworkRequest, HomeworkListQuery, UpdateHomeworkRequest,
        },
        responses::{
            AllHomeworksResponse, HomeworkCreator, HomeworkListItem, HomeworkListResponse,
            HomeworkStatsSummary, MySubmissionSummary,
        },
    },
};
use crate::utils::escape_like_pattern;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, EntityTrait, ExprTrait,
    FromQueryResult, JoinType, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set,
    TransactionTrait,
    sea_query::{Expr, Query},
};

impl SeaOrmStorage {
    /// 创建作业（使用事务保护）
    pub async fn create_homework_impl(
        &self,
        created_by: i64,
        req: CreateHomeworkRequest,
    ) -> Result<Homework> {
        let txn = self
            .db
            .begin()
            .await
            .map_err(|e| HWSystemError::database_operation(format!("开启事务失败: {e}")))?;

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
            .insert(&txn)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("创建作业失败: {e}")))?;

        // 处理附件
        if let Some(tokens) = req.attachments {
            self.set_homework_files_txn(&txn, result.id, tokens, created_by)
                .await?;
        }

        txn.commit()
            .await
            .map_err(|e| HWSystemError::database_operation(format!("提交事务失败: {e}")))?;

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
        let (page, page_size) = query.pagination.normalized();

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
        let paginator = select.paginate(&self.db, page_size);
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

        // 批量查询所有创建者
        let creator_map: HashMap<i64, HomeworkCreator> = if creator_ids.is_empty() {
            HashMap::new()
        } else {
            let users = Users::find()
                .filter(UserColumn::Id.is_in(creator_ids))
                .all(&self.db)
                .await
                .map_err(|e| {
                    HWSystemError::database_operation(format!("查询创建者信息失败: {e}"))
                })?;

            users
                .into_iter()
                .map(|user| {
                    (
                        user.id,
                        HomeworkCreator {
                            id: user.id,
                            username: user.username,
                            display_name: user.display_name,
                            avatar_url: user.avatar_url,
                        },
                    )
                })
                .collect()
        };

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

            // 批量查询每个班级的学生数（按 class_id 去重，单次 GROUP BY）
            let unique_class_ids: Vec<i64> = homeworks
                .iter()
                .map(|hw| hw.class_id)
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();

            let class_student_counts: HashMap<i64, i64> = if unique_class_ids.is_empty() {
                HashMap::new()
            } else {
                #[derive(FromQueryResult)]
                struct ClassCount {
                    class_id: i64,
                    count: i64,
                }

                let counts: Vec<ClassCount> = ClassUsers::find()
                    .filter(ClassUserColumn::ClassId.is_in(unique_class_ids))
                    .filter(ClassUserColumn::Role.is_in(["student", "class_representative"]))
                    .select_only()
                    .column(ClassUserColumn::ClassId)
                    .column_as(ClassUserColumn::Id.count(), "count")
                    .group_by(ClassUserColumn::ClassId)
                    .into_model::<ClassCount>()
                    .all(&self.db)
                    .await
                    .map_err(|e| {
                        HWSystemError::database_operation(format!("查询班级学生数失败: {e}"))
                    })?;

                counts.into_iter().map(|c| (c.class_id, c.count)).collect()
            };

            for hw in &homeworks {
                stats_map.insert(
                    hw.id,
                    HomeworkStatsSummary {
                        total_students: class_student_counts
                            .get(&hw.class_id)
                            .copied()
                            .unwrap_or(0),
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
                page_size: page_size as i64,
                total: total as i64,
                total_pages: pages as i64,
            },
        })
    }

    /// 更新作业（使用事务保护）
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

        let txn = self
            .db
            .begin()
            .await
            .map_err(|e| HWSystemError::database_operation(format!("开启事务失败: {e}")))?;

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
            .update(&txn)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("更新作业失败: {e}")))?;

        // 处理附件
        if let Some(tokens) = update.attachments {
            self.set_homework_files_txn(&txn, homework_id, tokens, user_id)
                .await?;
        }

        txn.commit()
            .await
            .map_err(|e| HWSystemError::database_operation(format!("提交事务失败: {e}")))?;

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
        self.set_homework_files_txn(&self.db, homework_id, tokens, user_id)
            .await
    }

    /// 设置作业附件（事务版本）
    pub async fn set_homework_files_txn<C: ConnectionTrait>(
        &self,
        conn: &C,
        homework_id: i64,
        tokens: Vec<String>,
        user_id: i64,
    ) -> Result<()> {
        // 先删除旧的关联
        HomeworkFiles::delete_many()
            .filter(HomeworkFileColumn::HomeworkId.eq(homework_id))
            .exec(conn)
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
                .insert(conn)
                .await
                .map_err(|e| HWSystemError::database_operation(format!("创建附件关联失败: {e}")))?;

            // 增加文件引用计数
            self.increment_file_citation_txn(conn, file.id).await?;
        }

        Ok(())
    }

    /// 获取学生作业统计（跨所有加入的班级）
    /// 返回 (pending, submitted, graded, total)
    pub async fn get_my_homework_stats_impl(&self, user_id: i64) -> Result<(i64, i64, i64, i64)> {
        // 1. 获取用户加入的所有班级
        let class_users = ClassUsers::find()
            .filter(ClassUserColumn::UserId.eq(user_id))
            .all(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询用户班级失败: {e}")))?;

        let class_ids: Vec<i64> = class_users.iter().map(|cu| cu.class_id).collect();
        if class_ids.is_empty() {
            return Ok((0, 0, 0, 0));
        }

        // 2. 获取这些班级的所有作业
        let homeworks = Homeworks::find()
            .filter(Column::ClassId.is_in(class_ids))
            .all(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询作业失败: {e}")))?;

        let total = homeworks.len() as i64;
        if total == 0 {
            return Ok((0, 0, 0, 0));
        }

        let homework_ids: Vec<i64> = homeworks.iter().map(|h| h.id).collect();

        // 3. 获取用户对这些作业的提交（取每个作业的最新版本）
        let submissions = Submissions::find()
            .filter(SubmissionColumn::HomeworkId.is_in(homework_ids.clone()))
            .filter(SubmissionColumn::CreatorId.eq(user_id))
            .order_by_desc(SubmissionColumn::Version)
            .all(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询提交失败: {e}")))?;

        // 按 homework_id 聚合，取最新版本
        let mut latest_submissions: HashMap<i64, i64> = HashMap::new(); // homework_id -> submission_id
        for sub in &submissions {
            latest_submissions.entry(sub.homework_id).or_insert(sub.id);
        }

        let submitted_homework_ids: std::collections::HashSet<i64> =
            latest_submissions.keys().cloned().collect();
        let pending = total - submitted_homework_ids.len() as i64;

        // 4. 查询这些提交的评分状态
        let submission_ids: Vec<i64> = latest_submissions.values().cloned().collect();
        let mut graded = 0i64;

        if !submission_ids.is_empty() {
            let grades = Grades::find()
                .filter(GradeColumn::SubmissionId.is_in(submission_ids))
                .all(&self.db)
                .await
                .map_err(|e| HWSystemError::database_operation(format!("查询评分失败: {e}")))?;

            graded = grades.len() as i64;
        }

        let submitted = submitted_homework_ids.len() as i64 - graded;

        Ok((pending, submitted, graded, total))
    }

    /// 获取教师作业统计（跨所有管理的班级）
    /// 返回 (total_homeworks, pending_review, total_submissions, graded_submissions)
    pub async fn get_teacher_homework_stats_impl(
        &self,
        user_id: i64,
    ) -> Result<(i64, i64, i64, i64)> {
        // 1. 获取教师管理的所有班级（作为 teacher 角色或班级创建者）
        let class_users = ClassUsers::find()
            .filter(ClassUserColumn::UserId.eq(user_id))
            .filter(ClassUserColumn::Role.eq("teacher"))
            .all(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询教师班级失败: {e}")))?;

        let mut class_ids: std::collections::HashSet<i64> =
            class_users.iter().map(|cu| cu.class_id).collect();

        // 也查询作为班级创建者（teacher_id）的班级
        use crate::entity::classes::{Column as ClassColumn, Entity as Classes};
        let owned_classes = Classes::find()
            .filter(ClassColumn::TeacherId.eq(user_id))
            .all(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询创建的班级失败: {e}")))?;

        for class in owned_classes {
            class_ids.insert(class.id);
        }

        if class_ids.is_empty() {
            return Ok((0, 0, 0, 0));
        }

        let class_ids_vec: Vec<i64> = class_ids.into_iter().collect();

        // 2. 获取这些班级的所有作业
        let homeworks = Homeworks::find()
            .filter(Column::ClassId.is_in(class_ids_vec))
            .all(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询作业失败: {e}")))?;

        let total_homeworks = homeworks.len() as i64;
        if total_homeworks == 0 {
            return Ok((0, 0, 0, 0));
        }

        let homework_ids: Vec<i64> = homeworks.iter().map(|h| h.id).collect();

        // 3. 获取这些作业的所有提交（按学生去重，取最新版本）
        let submissions = Submissions::find()
            .filter(SubmissionColumn::HomeworkId.is_in(homework_ids))
            .order_by_desc(SubmissionColumn::Version)
            .all(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询提交失败: {e}")))?;

        // 按 (homework_id, creator_id) 聚合，取最新版本
        let mut latest_submissions: HashMap<(i64, i64), i64> = HashMap::new();
        for sub in &submissions {
            latest_submissions
                .entry((sub.homework_id, sub.creator_id))
                .or_insert(sub.id);
        }

        let total_submissions = latest_submissions.len() as i64;

        // 4. 查询这些提交的评分状态
        let submission_ids: Vec<i64> = latest_submissions.values().cloned().collect();
        let mut graded_submissions = 0i64;

        if !submission_ids.is_empty() {
            let grades = Grades::find()
                .filter(GradeColumn::SubmissionId.is_in(submission_ids))
                .all(&self.db)
                .await
                .map_err(|e| HWSystemError::database_operation(format!("查询评分失败: {e}")))?;

            graded_submissions = grades.len() as i64;
        }

        let pending_review = total_submissions - graded_submissions;

        Ok((
            total_homeworks,
            pending_review,
            total_submissions,
            graded_submissions,
        ))
    }

    /// 构建状态过滤条件（将状态过滤下推到 SQL 层）
    ///
    /// 通过 EXISTS/NOT EXISTS 子查询实现：
    /// - Pending: 没有提交
    /// - Submitted: 有提交但没评分
    /// - Graded: 有提交且有评分
    fn build_status_filter(
        select: sea_orm::Select<Homeworks>,
        status: HomeworkUserStatus,
        user_id: i64,
    ) -> sea_orm::Select<Homeworks> {
        // 构建子查询：用户在该作业下是否有提交
        let submission_exists_subquery = Query::select()
            .expr(Expr::val(1))
            .from(Submissions)
            .cond_where(
                Condition::all()
                    .add(
                        Expr::col((Submissions, SubmissionColumn::HomeworkId))
                            .equals((Homeworks, Column::Id)),
                    )
                    .add(Expr::col((Submissions, SubmissionColumn::CreatorId)).eq(user_id)),
            )
            .to_owned();

        // 构建子查询：用户在该作业下是否有评分（需要 JOIN submissions）
        let grade_exists_subquery = Query::select()
            .expr(Expr::val(1))
            .from(Grades)
            .join(
                JoinType::InnerJoin,
                Submissions,
                Expr::col((Grades, GradeColumn::SubmissionId))
                    .equals((Submissions, SubmissionColumn::Id)),
            )
            .cond_where(
                Condition::all()
                    .add(
                        Expr::col((Submissions, SubmissionColumn::HomeworkId))
                            .equals((Homeworks, Column::Id)),
                    )
                    .add(Expr::col((Submissions, SubmissionColumn::CreatorId)).eq(user_id)),
            )
            .to_owned();

        match status {
            HomeworkUserStatus::Pending => {
                // 没有提交：NOT EXISTS submission
                select.filter(Expr::exists(submission_exists_subquery).not())
            }
            HomeworkUserStatus::Submitted => {
                // 有提交但没评分：EXISTS submission AND NOT EXISTS grade
                select
                    .filter(Expr::exists(submission_exists_subquery))
                    .filter(Expr::exists(grade_exists_subquery).not())
            }
            HomeworkUserStatus::Graded => {
                // 有提交且有评分：EXISTS grade
                select.filter(Expr::exists(grade_exists_subquery))
            }
        }
    }

    /// 列出用户所有班级的作业（跨班级）
    pub async fn list_all_homeworks_impl(
        &self,
        user_id: i64,
        is_teacher: bool,
        query: AllHomeworksQuery,
    ) -> Result<AllHomeworksResponse> {
        let (page, page_size) = query.pagination.normalized();
        let now = chrono::Utc::now();
        let now_ts = now.timestamp();

        // 1. 获取用户相关的班级
        let class_ids: Vec<i64> = if is_teacher {
            // 教师：获取管理的班级
            let class_users = ClassUsers::find()
                .filter(ClassUserColumn::UserId.eq(user_id))
                .filter(ClassUserColumn::Role.eq("teacher"))
                .all(&self.db)
                .await
                .map_err(|e| HWSystemError::database_operation(format!("查询教师班级失败: {e}")))?;

            let mut ids: std::collections::HashSet<i64> =
                class_users.iter().map(|cu| cu.class_id).collect();

            // 也查询作为班级创建者的班级
            use crate::entity::classes::{Column as ClassColumn, Entity as Classes};
            let owned_classes = Classes::find()
                .filter(ClassColumn::TeacherId.eq(user_id))
                .all(&self.db)
                .await
                .map_err(|e| {
                    HWSystemError::database_operation(format!("查询创建的班级失败: {e}"))
                })?;

            for class in owned_classes {
                ids.insert(class.id);
            }

            ids.into_iter().collect()
        } else {
            // 学生：获取加入的班级
            let class_users = ClassUsers::find()
                .filter(ClassUserColumn::UserId.eq(user_id))
                .all(&self.db)
                .await
                .map_err(|e| HWSystemError::database_operation(format!("查询用户班级失败: {e}")))?;

            class_users.iter().map(|cu| cu.class_id).collect()
        };

        if class_ids.is_empty() {
            return Ok(AllHomeworksResponse {
                items: vec![],
                pagination: PaginationInfo {
                    page: page as i64,
                    page_size: page_size as i64,
                    total: 0,
                    total_pages: 0,
                },
                server_time: now.to_rfc3339(),
            });
        }

        // 2. 构建基础查询
        let mut select = Homeworks::find().filter(Column::ClassId.is_in(class_ids.clone()));

        // 截止日期过滤
        match query.deadline_filter.unwrap_or_default() {
            DeadlineFilter::Active => {
                // 未过期：deadline 为空或 deadline > now
                select = select.filter(Column::Deadline.is_null().or(Column::Deadline.gt(now_ts)));
            }
            DeadlineFilter::Expired => {
                // 已过期：deadline 不为空且 deadline <= now
                select = select.filter(
                    Column::Deadline
                        .is_not_null()
                        .and(Column::Deadline.lte(now_ts)),
                );
            }
            DeadlineFilter::All => {
                // 不过滤
            }
        }

        // 搜索条件
        if let Some(ref search) = query.search
            && !search.trim().is_empty()
        {
            let escaped = escape_like_pattern(search.trim());
            select = select.filter(Column::Title.contains(&escaped));
        }

        // 排序
        select = select.order_by_desc(Column::CreatedAt);

        // 3. 应用状态过滤（下推到 SQL 层）
        if let Some(status) = query.status {
            select = Self::build_status_filter(select, status, user_id);
        }

        // 4. 数据库分页
        let paginator = select.clone().paginate(&self.db, page_size);
        let total = paginator
            .num_items()
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询作业总数失败: {e}")))?
            as i64;

        if total == 0 {
            return Ok(AllHomeworksResponse {
                items: vec![],
                pagination: PaginationInfo {
                    page: page as i64,
                    page_size: page_size as i64,
                    total: 0,
                    total_pages: 0,
                },
                server_time: now.to_rfc3339(),
            });
        }

        let total_pages = ((total as f64) / (page_size as f64)).ceil() as i64;
        let paged_homeworks: Vec<crate::entity::homeworks::Model> = paginator
            .fetch_page(page - 1)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询作业列表失败: {e}")))?;

        // 5. 转换为业务模型
        let ordered_homeworks: Vec<Homework> = paged_homeworks
            .into_iter()
            .map(|m| m.into_homework())
            .collect();

        let homework_ids: Vec<i64> = ordered_homeworks.iter().map(|h| h.id).collect();

        // 6. 查询用户的提交状态（仅针对分页后的作业）
        let submissions = Submissions::find()
            .filter(SubmissionColumn::HomeworkId.is_in(homework_ids.clone()))
            .filter(SubmissionColumn::CreatorId.eq(user_id))
            .order_by_desc(SubmissionColumn::Version)
            .all(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询用户提交失败: {e}")))?;

        // 按 homework_id 聚合，取最新版本
        let mut my_submission_map: HashMap<i64, (i64, i32, String, bool)> = HashMap::new(); // homework_id -> (submission_id, version, status, is_late)
        for sub in &submissions {
            my_submission_map.entry(sub.homework_id).or_insert((
                sub.id,
                sub.version,
                sub.status.clone(),
                sub.is_late,
            ));
        }

        // 查询评分信息
        let submission_ids: Vec<i64> = my_submission_map
            .values()
            .map(|(id, _, _, _)| *id)
            .collect();
        let mut grade_map: HashMap<i64, f64> = HashMap::new(); // submission_id -> score
        if !submission_ids.is_empty() {
            let grades = Grades::find()
                .filter(GradeColumn::SubmissionId.is_in(submission_ids))
                .all(&self.db)
                .await
                .map_err(|e| HWSystemError::database_operation(format!("查询评分失败: {e}")))?;

            for grade in grades {
                grade_map.insert(grade.submission_id, grade.score);
            }
        }

        // 7. 查询创建者信息
        let creator_ids: Vec<i64> = ordered_homeworks
            .iter()
            .map(|h| h.created_by)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        let mut creator_map: HashMap<i64, HomeworkCreator> = HashMap::new();
        if !creator_ids.is_empty() {
            let users = Users::find()
                .filter(UserColumn::Id.is_in(creator_ids))
                .all(&self.db)
                .await
                .map_err(|e| {
                    HWSystemError::database_operation(format!("查询创建者信息失败: {e}"))
                })?;

            for user in users {
                creator_map.insert(
                    user.id,
                    HomeworkCreator {
                        id: user.id,
                        username: user.username,
                        display_name: user.display_name,
                        avatar_url: user.avatar_url,
                    },
                );
            }
        }

        // 8. 查询统计信息（如果 include_stats=true）
        let mut stats_map: HashMap<i64, HomeworkStatsSummary> = HashMap::new();
        if query.include_stats.unwrap_or(false) && !ordered_homeworks.is_empty() {
            // 批量查询每个班级的学生数（按 class_id 去重，单次 GROUP BY）
            let unique_class_ids: Vec<i64> = ordered_homeworks
                .iter()
                .map(|hw| hw.class_id)
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();

            let class_student_counts: HashMap<i64, i64> = if unique_class_ids.is_empty() {
                HashMap::new()
            } else {
                #[derive(FromQueryResult)]
                struct ClassCount {
                    class_id: i64,
                    count: i64,
                }

                let counts: Vec<ClassCount> = ClassUsers::find()
                    .filter(ClassUserColumn::ClassId.is_in(unique_class_ids))
                    .filter(ClassUserColumn::Role.is_in(["student", "class_representative"]))
                    .select_only()
                    .column(ClassUserColumn::ClassId)
                    .column_as(ClassUserColumn::Id.count(), "count")
                    .group_by(ClassUserColumn::ClassId)
                    .into_model::<ClassCount>()
                    .all(&self.db)
                    .await
                    .map_err(|e| {
                        HWSystemError::database_operation(format!("查询班级学生数失败: {e}"))
                    })?;

                counts.into_iter().map(|c| (c.class_id, c.count)).collect()
            };

            for hw in &ordered_homeworks {
                stats_map.insert(
                    hw.id,
                    HomeworkStatsSummary {
                        total_students: class_student_counts
                            .get(&hw.class_id)
                            .copied()
                            .unwrap_or(0),
                        submitted_count: 0,
                        graded_count: 0,
                    },
                );
            }

            // 查询提交统计
            let hw_ids: Vec<i64> = ordered_homeworks.iter().map(|h| h.id).collect();
            let all_submissions = Submissions::find()
                .filter(SubmissionColumn::HomeworkId.is_in(hw_ids.clone()))
                .all(&self.db)
                .await
                .map_err(|e| HWSystemError::database_operation(format!("查询作业提交失败: {e}")))?;

            let mut hw_submitters: HashMap<i64, std::collections::HashSet<i64>> = HashMap::new();
            let mut all_sub_ids: Vec<i64> = Vec::new();
            for sub in &all_submissions {
                hw_submitters
                    .entry(sub.homework_id)
                    .or_default()
                    .insert(sub.creator_id);
                all_sub_ids.push(sub.id);
            }

            for (hw_id, submitters) in &hw_submitters {
                if let Some(stats) = stats_map.get_mut(hw_id) {
                    stats.submitted_count = submitters.len() as i64;
                }
            }

            // 查询评分统计
            if !all_sub_ids.is_empty() {
                let all_grades = Grades::find()
                    .filter(GradeColumn::SubmissionId.is_in(all_sub_ids))
                    .all(&self.db)
                    .await
                    .map_err(|e| HWSystemError::database_operation(format!("查询评分失败: {e}")))?;

                let sub_to_hw: HashMap<i64, i64> = all_submissions
                    .iter()
                    .map(|s| (s.id, s.homework_id))
                    .collect();
                let sub_to_creator: HashMap<i64, i64> = all_submissions
                    .iter()
                    .map(|s| (s.id, s.creator_id))
                    .collect();

                let mut hw_graded_users: HashMap<i64, std::collections::HashSet<i64>> =
                    HashMap::new();
                for grade in all_grades {
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

        // 9. 构造响应
        let items: Vec<HomeworkListItem> = ordered_homeworks
            .into_iter()
            .map(|homework| {
                let creator = creator_map.get(&homework.created_by).cloned();
                let my_submission =
                    my_submission_map
                        .get(&homework.id)
                        .map(|(id, version, status, is_late)| {
                            let score = grade_map.get(id).copied();
                            let final_status = if score.is_some() {
                                "graded".to_string()
                            } else {
                                status.clone()
                            };
                            MySubmissionSummary {
                                id: *id,
                                version: *version,
                                status: final_status,
                                is_late: *is_late,
                                score,
                            }
                        });
                let stats_summary = stats_map.get(&homework.id).cloned();
                HomeworkListItem {
                    homework,
                    creator,
                    my_submission,
                    stats_summary,
                }
            })
            .collect();

        Ok(AllHomeworksResponse {
            items,
            pagination: PaginationInfo {
                page: page as i64,
                page_size: page_size as i64,
                total,
                total_pages,
            },
            server_time: now.to_rfc3339(),
        })
    }

    /// 获取班级所有作业（不分页，用于内部统计/导出）
    pub async fn list_all_homeworks_by_class_impl(&self, class_id: i64) -> Result<Vec<Homework>> {
        let result = Homeworks::find()
            .filter(Column::ClassId.eq(class_id))
            .order_by_desc(Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询班级作业失败: {e}")))?;

        Ok(result.into_iter().map(|m| m.into_homework()).collect())
    }
}
