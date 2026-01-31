use std::collections::HashMap;
use std::sync::Arc;

use crate::models::{
    class_users::{
        entities::{ClassUser, ClassUserRole},
        requests::{ClassUserListQuery, UpdateClassUserRequest},
        responses::ClassUserListResponse,
    },
    classes::{
        entities::Class,
        requests::{ClassListQuery, CreateClassRequest, UpdateClassRequest},
        responses::ClassListResponse,
    },
    files::entities::File,
    grades::{
        entities::Grade,
        requests::{CreateGradeRequest, GradeListQuery, UpdateGradeRequest},
        responses::GradeListResponse,
    },
    homeworks::{
        entities::Homework,
        requests::{
            AllHomeworksQuery, CreateHomeworkRequest, HomeworkListQuery, UpdateHomeworkRequest,
        },
        responses::{AllHomeworksResponse, HomeworkListResponse},
    },
    notifications::{
        entities::Notification,
        requests::{CreateNotificationRequest, NotificationListQuery},
        responses::NotificationListResponse,
    },
    submissions::{
        entities::Submission,
        requests::{CreateSubmissionRequest, SubmissionListQuery},
        responses::{
            SubmissionListItem, SubmissionListResponse, SubmissionResponse,
            SubmissionSummaryResponse, UserSubmissionHistoryItem,
        },
    },
    system::{
        entities::SystemSetting, requests::SettingAuditQuery, responses::SettingAuditListResponse,
    },
    users::{
        entities::{User, UserRole, UserStatus},
        requests::{CreateUserRequest, UpdateUserRequest, UserListQuery},
        responses::{UserListResponse, UserStatsResponse},
    },
};

use crate::errors::Result;

pub mod sea_orm_storage;

#[async_trait::async_trait]
pub trait Storage: Send + Sync {
    // ============================================
    // 用户管理方法
    // ============================================

    /// 创建用户
    async fn create_user(&self, user: CreateUserRequest) -> Result<User>;
    /// 通过ID获取用户信息
    async fn get_user_by_id(&self, id: i64) -> Result<Option<User>>;
    /// 通过用户名获取用户信息
    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>>;
    /// 通过邮箱获取用户信息
    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>>;
    /// 通过用户名或邮箱获取用户信息
    async fn get_user_by_username_or_email(&self, identifier: &str) -> Result<Option<User>>;
    /// 列出用户
    async fn list_users_with_pagination(&self, query: UserListQuery) -> Result<UserListResponse>;
    /// 更新用户信息
    async fn update_user(&self, id: i64, update: UpdateUserRequest) -> Result<Option<User>>;
    /// 删除用户
    async fn delete_user(&self, id: i64) -> Result<bool>;
    /// 更新用户最后登录时间
    async fn update_last_login(&self, id: i64) -> Result<bool>;
    /// 统计用户数量
    async fn count_users(&self) -> Result<u64>;
    /// 批量检查用户名是否已存在
    async fn check_usernames_exist(&self, usernames: &[String]) -> Result<Vec<String>>;
    /// 批量检查邮箱是否已存在
    async fn check_emails_exist(&self, emails: &[String]) -> Result<Vec<String>>;
    /// 列出所有用户（用于导出，限制数量）
    async fn list_all_users_for_export(&self, limit: u64) -> Result<Vec<User>>;
    /// 列出用户（用于导出，支持筛选）
    async fn list_users_for_export_filtered(
        &self,
        limit: u64,
        role: Option<UserRole>,
        status: Option<UserStatus>,
        search: Option<&str>,
    ) -> Result<Vec<User>>;
    /// 获取用户综合统计（合并学生和教师视角）
    async fn get_user_stats(&self, user_id: i64, role: UserRole) -> Result<UserStatsResponse>;
    /// 批量获取用户信息
    async fn get_users_by_ids(&self, ids: &[i64]) -> Result<HashMap<i64, User>>;

    // ============================================
    // 文件管理方法
    // ============================================

    /// 上传文件
    async fn upload_file(
        &self,
        original_name: &str,
        stored_name: &str,
        file_size: &i64,
        file_type: &str,
        user_id: i64,
    ) -> Result<File>;
    /// 通过唯一 token 获取文件信息
    async fn get_file_by_token(&self, token: &str) -> Result<Option<File>>;
    /// 通过 ID 获取文件信息
    async fn get_file_by_id(&self, id: i64) -> Result<Option<File>>;
    /// 增加文件引用计数
    async fn increment_file_citation(&self, file_id: i64) -> Result<bool>;
    /// 减少文件引用计数
    async fn decrement_file_citation(&self, file_id: i64) -> Result<bool>;
    /// 删除文件（仅当引用计数为 0 时删除物理文件）
    async fn delete_file(&self, token: &str, user_id: i64) -> Result<bool>;

    // ============================================
    // 班级管理方法
    // ============================================

    /// 创建班级
    async fn create_class(&self, class: CreateClassRequest) -> Result<Class>;
    /// 通过ID获取班级信息
    async fn get_class_by_id(&self, class_id: i64) -> Result<Option<Class>>;
    /// 通过邀请码获取班级信息
    async fn get_class_by_code(&self, invite_code: &str) -> Result<Option<Class>>;
    /// 列出班级
    async fn list_classes_with_pagination(
        &self,
        query: ClassListQuery,
    ) -> Result<ClassListResponse>;
    /// 更新班级信息
    async fn update_class(
        &self,
        class_id: i64,
        update: UpdateClassRequest,
    ) -> Result<Option<Class>>;
    /// 删除班级
    async fn delete_class(&self, class_id: i64) -> Result<bool>;

    // ============================================
    // 班级成员管理方法
    // ============================================

    /// 获取班级成员数量
    async fn count_class_members(&self, class_id: i64) -> Result<i64>;
    /// 学生加入班级
    async fn join_class(
        &self,
        user_id: i64,
        class_id: i64,
        role: ClassUserRole,
    ) -> Result<ClassUser>;
    /// 学生离开/踢出班级
    async fn leave_class(&self, user_id: i64, class_id: i64) -> Result<bool>;
    /// 更新班级用户信息（通过 user_id 和 class_id）
    async fn update_class_user(
        &self,
        class_id: i64,
        user_id: i64,
        update_data: UpdateClassUserRequest,
    ) -> Result<Option<ClassUser>>;
    /// 列出班级用户
    async fn list_class_users_with_pagination(
        &self,
        class_id: i64,
        query: ClassUserListQuery,
    ) -> Result<ClassUserListResponse>;
    /// 列出用户所在的班级
    async fn list_user_classes_with_pagination(
        &self,
        user_id: i64,
        query: ClassListQuery,
    ) -> Result<ClassListResponse>;
    /// 获取用户在班级中的信息
    async fn get_class_user_by_user_id_and_class_id(
        &self,
        user_id: i64,
        class_id: i64,
    ) -> Result<Option<ClassUser>>;
    /// 通过班级用户 ID 获取班级用户信息
    async fn get_class_user_by_id(&self, class_user_id: i64) -> Result<Option<ClassUser>>;
    /// 根据班级ID和邀请码获取班级及用户信息
    async fn get_class_and_class_user_by_class_id_and_code(
        &self,
        class_id: i64,
        invite_code: &str,
        user_id: i64,
    ) -> Result<(Option<Class>, Option<ClassUser>)>;
    /// 获取班级所有成员（不分页，用于内部统计/导出）
    async fn list_all_class_users(&self, class_id: i64) -> Result<Vec<ClassUser>>;

    // ============================================
    // 作业管理方法
    // ============================================

    /// 创建作业
    async fn create_homework(
        &self,
        created_by: i64,
        req: CreateHomeworkRequest,
    ) -> Result<Homework>;
    /// 通过 ID 获取作业
    async fn get_homework_by_id(&self, homework_id: i64) -> Result<Option<Homework>>;
    /// 列出作业
    /// - current_user_id: 当前用户 ID，如果提供则查询该用户对这些作业的提交状态
    async fn list_homeworks_with_pagination(
        &self,
        query: HomeworkListQuery,
        current_user_id: Option<i64>,
    ) -> Result<HomeworkListResponse>;
    /// 更新作业
    async fn update_homework(
        &self,
        homework_id: i64,
        update: UpdateHomeworkRequest,
        user_id: i64,
    ) -> Result<Option<Homework>>;
    /// 删除作业
    async fn delete_homework(&self, homework_id: i64) -> Result<bool>;
    /// 获取作业附件 ID 列表
    async fn get_homework_file_ids(&self, homework_id: i64) -> Result<Vec<i64>>;
    /// 设置作业附件（通过 download_token，带所有权校验）
    async fn set_homework_files(
        &self,
        homework_id: i64,
        tokens: Vec<String>,
        user_id: i64,
    ) -> Result<()>;
    /// 获取学生作业统计（跨所有加入的班级）
    async fn get_my_homework_stats(&self, user_id: i64) -> Result<(i64, i64, i64, i64)>;
    /// 获取教师作业统计（跨所有管理的班级）
    async fn get_teacher_homework_stats(&self, user_id: i64) -> Result<(i64, i64, i64, i64)>;
    /// 列出用户所有班级的作业（跨班级）
    /// - user_id: 当前用户 ID
    /// - is_teacher: 是否为教师视角
    async fn list_all_homeworks(
        &self,
        user_id: i64,
        is_teacher: bool,
        query: AllHomeworksQuery,
    ) -> Result<AllHomeworksResponse>;
    /// 获取班级所有作业（不分页，用于内部统计/导出）
    async fn list_all_homeworks_by_class(&self, class_id: i64) -> Result<Vec<Homework>>;

    // ============================================
    // 提交管理方法
    // ============================================

    /// 创建提交（自动计算版本号）
    async fn create_submission(
        &self,
        creator_id: i64,
        req: CreateSubmissionRequest,
    ) -> Result<Submission>;
    /// 通过 ID 获取提交
    async fn get_submission_by_id(&self, submission_id: i64) -> Result<Option<Submission>>;
    /// 通过 ID 获取提交详情（完整响应，包含 creator、attachments、grade）
    async fn get_submission_response(
        &self,
        submission_id: i64,
    ) -> Result<Option<SubmissionResponse>>;
    /// 获取学生某作业的最新提交
    async fn get_latest_submission(
        &self,
        homework_id: i64,
        creator_id: i64,
    ) -> Result<Option<Submission>>;
    /// 获取学生某作业的提交历史（包含评分和附件）
    async fn list_user_submissions(
        &self,
        homework_id: i64,
        creator_id: i64,
    ) -> Result<Vec<UserSubmissionHistoryItem>>;
    /// 列出作业的所有提交（分页）
    async fn list_submissions_with_pagination(
        &self,
        query: SubmissionListQuery,
    ) -> Result<SubmissionListResponse>;
    /// 获取作业所有提交（不分页，用于内部统计/导出）
    async fn list_all_submissions_by_homework(
        &self,
        homework_id: i64,
    ) -> Result<Vec<SubmissionListItem>>;
    /// 批量获取多个作业的所有提交（不分页，用于导出）
    async fn list_all_submissions_by_homework_ids(
        &self,
        homework_ids: &[i64],
    ) -> Result<Vec<SubmissionListItem>>;
    /// 删除提交（撤回）
    async fn delete_submission(&self, submission_id: i64) -> Result<bool>;
    /// 更新提交状态
    async fn update_submission_status(&self, submission_id: i64, status: &str) -> Result<bool>;
    /// 获取提交附件 ID 列表
    async fn get_submission_file_ids(&self, submission_id: i64) -> Result<Vec<i64>>;
    /// 设置提交附件（通过 download_token，带所有权校验）
    async fn set_submission_files(
        &self,
        submission_id: i64,
        tokens: Vec<String>,
        user_id: i64,
    ) -> Result<()>;
    /// 获取作业提交概览（按学生聚合）
    /// - `include_grades`: 是否包含成绩信息（课代表不可见成绩）
    /// - `graded`: 筛选是否已批改，true=已批改，false=待批改，None=全部
    async fn get_submission_summary(
        &self,
        homework_id: i64,
        page: i64,
        page_size: i64,
        include_grades: bool,
        graded: Option<bool>,
    ) -> Result<SubmissionSummaryResponse>;
    /// 获取某学生某作业的所有提交版本（教师视角，包含评分和附件）
    /// - `include_grades`: 是否包含成绩信息（课代表不可见成绩）
    async fn list_user_submissions_for_teacher(
        &self,
        homework_id: i64,
        user_id: i64,
        include_grades: bool,
    ) -> Result<Vec<UserSubmissionHistoryItem>>;

    // ============================================
    // 评分管理方法
    // ============================================

    /// 创建评分
    async fn create_grade(&self, grader_id: i64, req: CreateGradeRequest) -> Result<Grade>;
    /// 通过 ID 获取评分
    async fn get_grade_by_id(&self, grade_id: i64) -> Result<Option<Grade>>;
    /// 通过提交 ID 获取评分
    async fn get_grade_by_submission_id(&self, submission_id: i64) -> Result<Option<Grade>>;
    /// 更新评分
    async fn update_grade(
        &self,
        grade_id: i64,
        update: UpdateGradeRequest,
    ) -> Result<Option<Grade>>;
    /// 列出评分（分页）
    async fn list_grades_with_pagination(&self, query: GradeListQuery)
    -> Result<GradeListResponse>;
    /// 批量获取评分（通过提交ID列表），返回 submission_id -> Grade 的映射
    async fn get_grades_by_submission_ids(
        &self,
        submission_ids: &[i64],
    ) -> Result<HashMap<i64, Grade>>;

    // ============================================
    // 通知管理方法
    // ============================================

    /// 创建通知
    async fn create_notification(&self, req: CreateNotificationRequest) -> Result<Notification>;
    /// 批量创建通知（用于群发）
    async fn create_notifications_batch(
        &self,
        reqs: Vec<CreateNotificationRequest>,
    ) -> Result<Vec<Notification>>;
    /// 通过 ID 获取通知
    async fn get_notification_by_id(&self, notification_id: i64) -> Result<Option<Notification>>;
    /// 列出用户的通知（分页）
    async fn list_notifications_with_pagination(
        &self,
        user_id: i64,
        query: NotificationListQuery,
    ) -> Result<NotificationListResponse>;
    /// 获取用户未读通知数量
    async fn get_unread_notification_count(&self, user_id: i64) -> Result<i64>;
    /// 标记通知为已读
    async fn mark_notification_as_read(&self, notification_id: i64) -> Result<bool>;
    /// 标记用户所有通知为已读
    async fn mark_all_notifications_as_read(&self, user_id: i64) -> Result<i64>;
    /// 删除通知
    async fn delete_notification(&self, notification_id: i64) -> Result<bool>;

    // ============================================
    // 系统设置管理方法
    // ============================================

    /// 获取所有设置
    async fn list_all_settings(&self) -> Result<Vec<SystemSetting>>;
    /// 通过 key 获取设置
    async fn get_setting_by_key(&self, key: &str) -> Result<Option<SystemSetting>>;
    /// 更新设置
    async fn update_setting(
        &self,
        key: &str,
        value: &str,
        user_id: i64,
        ip_address: Option<String>,
    ) -> Result<SystemSetting>;
    /// 批量更新设置
    async fn batch_update_settings(
        &self,
        updates: Vec<(String, String)>,
        user_id: i64,
        ip_address: Option<String>,
    ) -> Result<Vec<SystemSetting>>;
    /// 获取审计日志
    async fn list_setting_audits(
        &self,
        query: SettingAuditQuery,
    ) -> Result<SettingAuditListResponse>;
}

pub async fn create_storage() -> Result<Arc<dyn Storage>> {
    let storage = sea_orm_storage::SeaOrmStorage::new_async().await?;
    Ok(Arc::new(storage))
}
