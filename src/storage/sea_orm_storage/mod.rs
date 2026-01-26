//! SeaORM 存储实现
//!
//! 统一的数据库存储层，支持 SQLite、PostgreSQL 和 MySQL。

mod class_users;
mod classes;
mod files;
mod grades;
mod homeworks;
mod notifications;
mod submissions;
mod system_settings;
mod users;

use crate::config::AppConfig;
use crate::errors::{HWSystemError, Result};
use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::time::Duration;
use tracing::info;

/// SeaORM 存储实现
#[derive(Clone)]
pub struct SeaOrmStorage {
    pub(crate) db: DatabaseConnection,
}

impl SeaOrmStorage {
    /// 创建新的 SeaORM 存储实例
    pub async fn new_async() -> Result<Self> {
        let config = AppConfig::get();
        let db_url = Self::build_database_url(&config.database.url)?;

        // 根据数据库类型选择连接方式
        let db = if db_url.starts_with("sqlite://") {
            Self::connect_sqlite(&db_url, config).await?
        } else {
            Self::connect_generic(&db_url, config).await?
        };

        // 运行迁移
        Migrator::up(&db, None)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("数据库迁移失败: {e}")))?;

        info!("SeaORM 存储初始化完成，数据库: {}", db_url);

        Ok(Self { db })
    }

    /// SQLite 专用连接（WAL + pragma 优化）
    async fn connect_sqlite(url: &str, config: &AppConfig) -> Result<DatabaseConnection> {
        use sea_orm::SqlxSqliteConnector;
        use sea_orm::sqlx::sqlite::{
            SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous,
        };
        use std::str::FromStr;

        let opt = SqliteConnectOptions::from_str(url)
            .map_err(|e| HWSystemError::database_config(format!("SQLite URL 解析失败: {e}")))?
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal)
            .busy_timeout(Duration::from_secs(5))
            .pragma("cache_size", "-64000")
            .pragma("temp_store", "memory")
            .pragma("mmap_size", "536870912")
            .pragma("wal_autocheckpoint", "1000");

        let pool = SqlitePoolOptions::new()
            .max_connections(config.database.pool_size)
            .min_connections(1)
            .test_before_acquire(true)
            .acquire_timeout(Duration::from_secs(config.database.timeout))
            .idle_timeout(Duration::from_secs(300))
            .connect_with(opt)
            .await
            .map_err(|e| HWSystemError::database_connection(format!("SQLite 连接失败: {e}")))?;

        Ok(SqlxSqliteConnector::from_sqlx_sqlite_pool(pool))
    }

    /// 通用连接（PostgreSQL、MySQL 等）
    async fn connect_generic(url: &str, config: &AppConfig) -> Result<DatabaseConnection> {
        let mut opt = ConnectOptions::new(url);
        opt.max_connections(config.database.pool_size)
            .min_connections(5)
            .connect_timeout(Duration::from_secs(config.database.timeout))
            .acquire_timeout(Duration::from_secs(config.database.timeout))
            .idle_timeout(Duration::from_secs(600))
            .max_lifetime(Duration::from_secs(1800))
            .sqlx_logging(false)
            .sqlx_logging_level(tracing::log::LevelFilter::Debug);

        Database::connect(opt)
            .await
            .map_err(|e| HWSystemError::database_connection(format!("无法连接到数据库: {e}")))
    }

    /// 从 URL 自动推断数据库类型并构建连接 URL
    fn build_database_url(url: &str) -> Result<String> {
        if url.starts_with("sqlite://") {
            Ok(url.to_string())
        } else if url.ends_with(".db") || url.ends_with(".sqlite") || url == ":memory:" {
            Ok(format!("sqlite://{}?mode=rwc", url))
        } else if url.starts_with("postgres://")
            || url.starts_with("postgresql://")
            || url.starts_with("mysql://")
            || url.starts_with("mariadb://")
        {
            Ok(url.to_string())
        } else {
            Err(HWSystemError::database_config(format!(
                "无法从 URL 推断数据库类型: {url}. 支持: sqlite://, postgres://, mysql://, 或 .db/.sqlite 文件路径"
            )))
        }
    }
}

// Storage trait 实现
use crate::models::{
    class_users::{
        entities::{ClassUser, ClassUserRole},
        requests::{ClassUserQuery, UpdateClassUserRequest},
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
        requests::{CreateHomeworkRequest, HomeworkListQuery, UpdateHomeworkRequest},
        responses::HomeworkListResponse,
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
            SubmissionListResponse, SubmissionResponse, SubmissionSummaryResponse,
            UserSubmissionHistoryItem,
        },
    },
    users::{
        entities::{User, UserRole, UserStatus},
        requests::{CreateUserRequest, UpdateUserRequest, UserListQuery},
        responses::UserListResponse,
    },
};
use crate::storage::Storage;
use async_trait::async_trait;

#[async_trait]
impl Storage for SeaOrmStorage {
    // ============================================
    // 用户模块
    // ============================================

    async fn create_user(&self, user: CreateUserRequest) -> Result<User> {
        self.create_user_impl(user).await
    }

    async fn get_user_by_id(&self, id: i64) -> Result<Option<User>> {
        self.get_user_by_id_impl(id).await
    }

    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        self.get_user_by_username_impl(username).await
    }

    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        self.get_user_by_email_impl(email).await
    }

    async fn get_user_by_username_or_email(&self, identifier: &str) -> Result<Option<User>> {
        self.get_user_by_username_or_email_impl(identifier).await
    }

    async fn list_users_with_pagination(&self, query: UserListQuery) -> Result<UserListResponse> {
        self.list_users_with_pagination_impl(query).await
    }

    async fn update_last_login(&self, id: i64) -> Result<bool> {
        self.update_last_login_impl(id).await
    }

    async fn update_user(&self, id: i64, update: UpdateUserRequest) -> Result<Option<User>> {
        self.update_user_impl(id, update).await
    }

    async fn delete_user(&self, id: i64) -> Result<bool> {
        self.delete_user_impl(id).await
    }

    async fn count_users(&self) -> Result<u64> {
        self.count_users_impl().await
    }

    async fn check_usernames_exist(&self, usernames: &[String]) -> Result<Vec<String>> {
        self.check_usernames_exist_impl(usernames).await
    }

    async fn check_emails_exist(&self, emails: &[String]) -> Result<Vec<String>> {
        self.check_emails_exist_impl(emails).await
    }

    async fn list_all_users_for_export(&self, limit: u64) -> Result<Vec<User>> {
        self.list_all_users_for_export_impl(limit).await
    }

    async fn list_users_for_export_filtered(
        &self,
        limit: u64,
        role: Option<UserRole>,
        status: Option<UserStatus>,
        search: Option<&str>,
    ) -> Result<Vec<User>> {
        self.list_users_for_export_filtered_impl(limit, role, status, search)
            .await
    }

    // ============================================
    // 文件模块
    // ============================================

    async fn upload_file(
        &self,
        original_name: &str,
        stored_name: &str,
        file_size: &i64,
        file_type: &str,
        user_id: i64,
    ) -> Result<File> {
        self.upload_file_impl(original_name, stored_name, file_size, file_type, user_id)
            .await
    }

    async fn get_file_by_token(&self, token: &str) -> Result<Option<File>> {
        self.get_file_by_token_impl(token).await
    }

    async fn get_file_by_id(&self, id: i64) -> Result<Option<File>> {
        self.get_file_by_id_impl(id).await
    }

    async fn increment_file_citation(&self, file_id: i64) -> Result<bool> {
        self.increment_file_citation_impl(file_id).await
    }

    async fn decrement_file_citation(&self, file_id: i64) -> Result<bool> {
        self.decrement_file_citation_impl(file_id).await
    }

    // ============================================
    // 班级模块
    // ============================================

    async fn create_class(&self, class: CreateClassRequest) -> Result<Class> {
        self.create_class_impl(class).await
    }

    async fn get_class_by_id(&self, class_id: i64) -> Result<Option<Class>> {
        self.get_class_by_id_impl(class_id).await
    }

    async fn get_class_by_code(&self, invite_code: &str) -> Result<Option<Class>> {
        self.get_class_by_code_impl(invite_code).await
    }

    async fn list_classes_with_pagination(
        &self,
        query: ClassListQuery,
    ) -> Result<ClassListResponse> {
        self.list_classes_with_pagination_impl(query).await
    }

    async fn update_class(
        &self,
        class_id: i64,
        update: UpdateClassRequest,
    ) -> Result<Option<Class>> {
        self.update_class_impl(class_id, update).await
    }

    async fn delete_class(&self, class_id: i64) -> Result<bool> {
        self.delete_class_impl(class_id).await
    }

    // ============================================
    // 班级用户模块
    // ============================================

    async fn count_class_members(&self, class_id: i64) -> Result<i64> {
        self.count_class_members_impl(class_id).await
    }

    async fn join_class(
        &self,
        user_id: i64,
        class_id: i64,
        role: ClassUserRole,
    ) -> Result<ClassUser> {
        self.join_class_impl(user_id, class_id, role).await
    }

    async fn leave_class(&self, user_id: i64, class_id: i64) -> Result<bool> {
        self.leave_class_impl(user_id, class_id).await
    }

    async fn update_class_user(
        &self,
        class_id: i64,
        user_id: i64,
        update_data: UpdateClassUserRequest,
    ) -> Result<Option<ClassUser>> {
        self.update_class_user_impl(class_id, user_id, update_data)
            .await
    }

    async fn list_class_users_with_pagination(
        &self,
        class_id: i64,
        query: ClassUserQuery,
    ) -> Result<ClassUserListResponse> {
        self.list_class_users_with_pagination_impl(class_id, query)
            .await
    }

    async fn list_user_classes_with_pagination(
        &self,
        user_id: i64,
        query: ClassListQuery,
    ) -> Result<ClassListResponse> {
        self.list_user_classes_with_pagination_impl(user_id, query)
            .await
    }

    async fn get_class_user_by_user_id_and_class_id(
        &self,
        user_id: i64,
        class_id: i64,
    ) -> Result<Option<ClassUser>> {
        self.get_class_user_by_user_id_and_class_id_impl(user_id, class_id)
            .await
    }

    async fn get_class_user_by_id(&self, class_user_id: i64) -> Result<Option<ClassUser>> {
        self.get_class_user_by_id_impl(class_user_id).await
    }

    async fn get_class_and_class_user_by_class_id_and_code(
        &self,
        class_id: i64,
        invite_code: &str,
        user_id: i64,
    ) -> Result<(Option<Class>, Option<ClassUser>)> {
        self.get_class_and_class_user_by_class_id_and_code_impl(class_id, invite_code, user_id)
            .await
    }

    // ============================================
    // 作业模块
    // ============================================

    async fn create_homework(
        &self,
        created_by: i64,
        req: CreateHomeworkRequest,
    ) -> Result<Homework> {
        self.create_homework_impl(created_by, req).await
    }

    async fn get_homework_by_id(&self, homework_id: i64) -> Result<Option<Homework>> {
        self.get_homework_by_id_impl(homework_id).await
    }

    async fn list_homeworks_with_pagination(
        &self,
        query: HomeworkListQuery,
        current_user_id: Option<i64>,
    ) -> Result<HomeworkListResponse> {
        self.list_homeworks_with_pagination_impl(query, current_user_id)
            .await
    }

    async fn update_homework(
        &self,
        homework_id: i64,
        update: UpdateHomeworkRequest,
        user_id: i64,
    ) -> Result<Option<Homework>> {
        self.update_homework_impl(homework_id, update, user_id)
            .await
    }

    async fn delete_homework(&self, homework_id: i64) -> Result<bool> {
        self.delete_homework_impl(homework_id).await
    }

    async fn get_homework_file_ids(&self, homework_id: i64) -> Result<Vec<i64>> {
        self.get_homework_file_ids_impl(homework_id).await
    }

    async fn set_homework_files(
        &self,
        homework_id: i64,
        tokens: Vec<String>,
        user_id: i64,
    ) -> Result<()> {
        self.set_homework_files_impl(homework_id, tokens, user_id)
            .await
    }

    async fn get_my_homework_stats(&self, user_id: i64) -> Result<(i64, i64, i64, i64)> {
        self.get_my_homework_stats_impl(user_id).await
    }

    async fn get_teacher_homework_stats(&self, user_id: i64) -> Result<(i64, i64, i64, i64)> {
        self.get_teacher_homework_stats_impl(user_id).await
    }

    // ============================================
    // 提交模块
    // ============================================

    async fn create_submission(
        &self,
        creator_id: i64,
        req: CreateSubmissionRequest,
    ) -> Result<Submission> {
        self.create_submission_impl(creator_id, req).await
    }

    async fn get_submission_by_id(&self, submission_id: i64) -> Result<Option<Submission>> {
        self.get_submission_by_id_impl(submission_id).await
    }

    async fn get_submission_response(
        &self,
        submission_id: i64,
    ) -> Result<Option<SubmissionResponse>> {
        self.get_submission_response_impl(submission_id).await
    }

    async fn get_latest_submission(
        &self,
        homework_id: i64,
        creator_id: i64,
    ) -> Result<Option<Submission>> {
        self.get_latest_submission_impl(homework_id, creator_id)
            .await
    }

    async fn list_user_submissions(
        &self,
        homework_id: i64,
        creator_id: i64,
    ) -> Result<Vec<UserSubmissionHistoryItem>> {
        self.list_user_submissions_impl(homework_id, creator_id)
            .await
    }

    async fn list_submissions_with_pagination(
        &self,
        query: SubmissionListQuery,
    ) -> Result<SubmissionListResponse> {
        self.list_submissions_with_pagination_impl(query).await
    }

    async fn delete_submission(&self, submission_id: i64) -> Result<bool> {
        self.delete_submission_impl(submission_id).await
    }

    async fn update_submission_status(&self, submission_id: i64, status: &str) -> Result<bool> {
        self.update_submission_status_impl(submission_id, status)
            .await
    }

    async fn get_submission_file_ids(&self, submission_id: i64) -> Result<Vec<i64>> {
        self.get_submission_file_ids_impl(submission_id).await
    }

    async fn set_submission_files(
        &self,
        submission_id: i64,
        tokens: Vec<String>,
        user_id: i64,
    ) -> Result<()> {
        self.set_submission_files_impl(submission_id, tokens, user_id)
            .await
    }

    async fn get_submission_summary(
        &self,
        homework_id: i64,
        page: i64,
        size: i64,
        include_grades: bool,
        graded: Option<bool>,
    ) -> Result<SubmissionSummaryResponse> {
        self.get_submission_summary_impl(homework_id, page, size, include_grades, graded)
            .await
    }

    async fn list_user_submissions_for_teacher(
        &self,
        homework_id: i64,
        user_id: i64,
        include_grades: bool,
    ) -> Result<Vec<UserSubmissionHistoryItem>> {
        self.list_user_submissions_for_teacher_impl(homework_id, user_id, include_grades)
            .await
    }

    // ============================================
    // 评分模块
    // ============================================

    async fn create_grade(&self, grader_id: i64, req: CreateGradeRequest) -> Result<Grade> {
        self.create_grade_impl(grader_id, req).await
    }

    async fn get_grade_by_id(&self, grade_id: i64) -> Result<Option<Grade>> {
        self.get_grade_by_id_impl(grade_id).await
    }

    async fn get_grade_by_submission_id(&self, submission_id: i64) -> Result<Option<Grade>> {
        self.get_grade_by_submission_id_impl(submission_id).await
    }

    async fn update_grade(
        &self,
        grade_id: i64,
        update: UpdateGradeRequest,
    ) -> Result<Option<Grade>> {
        self.update_grade_impl(grade_id, update).await
    }

    async fn list_grades_with_pagination(
        &self,
        query: GradeListQuery,
    ) -> Result<GradeListResponse> {
        self.list_grades_with_pagination_impl(query).await
    }

    // ============================================
    // 通知模块
    // ============================================

    async fn create_notification(&self, req: CreateNotificationRequest) -> Result<Notification> {
        self.create_notification_impl(req).await
    }

    async fn create_notifications_batch(
        &self,
        reqs: Vec<CreateNotificationRequest>,
    ) -> Result<Vec<Notification>> {
        self.create_notifications_batch_impl(reqs).await
    }

    async fn get_notification_by_id(&self, notification_id: i64) -> Result<Option<Notification>> {
        self.get_notification_by_id_impl(notification_id).await
    }

    async fn list_notifications_with_pagination(
        &self,
        user_id: i64,
        query: NotificationListQuery,
    ) -> Result<NotificationListResponse> {
        self.list_notifications_with_pagination_impl(user_id, query)
            .await
    }

    async fn get_unread_notification_count(&self, user_id: i64) -> Result<i64> {
        self.get_unread_notification_count_impl(user_id).await
    }

    async fn mark_notification_as_read(&self, notification_id: i64) -> Result<bool> {
        self.mark_notification_as_read_impl(notification_id).await
    }

    async fn mark_all_notifications_as_read(&self, user_id: i64) -> Result<i64> {
        self.mark_all_notifications_as_read_impl(user_id).await
    }

    async fn delete_notification(&self, notification_id: i64) -> Result<bool> {
        self.delete_notification_impl(notification_id).await
    }

    // ============================================
    // 系统设置模块
    // ============================================

    async fn list_all_settings(
        &self,
    ) -> Result<Vec<crate::models::system::entities::SystemSetting>> {
        self.list_all_settings_impl().await
    }

    async fn get_setting_by_key(
        &self,
        key: &str,
    ) -> Result<Option<crate::models::system::entities::SystemSetting>> {
        self.get_setting_by_key_impl(key).await
    }

    async fn update_setting(
        &self,
        key: &str,
        value: &str,
        user_id: i64,
        ip_address: Option<String>,
    ) -> Result<crate::models::system::entities::SystemSetting> {
        self.update_setting_impl(key, value, user_id, ip_address)
            .await
    }

    async fn batch_update_settings(
        &self,
        updates: Vec<(String, String)>,
        user_id: i64,
        ip_address: Option<String>,
    ) -> Result<Vec<crate::models::system::entities::SystemSetting>> {
        self.batch_update_settings_impl(updates, user_id, ip_address)
            .await
    }

    async fn list_setting_audits(
        &self,
        query: crate::models::system::requests::SettingAuditQuery,
    ) -> Result<crate::models::system::responses::SettingAuditListResponse> {
        self.list_setting_audits_impl(query).await
    }
}
