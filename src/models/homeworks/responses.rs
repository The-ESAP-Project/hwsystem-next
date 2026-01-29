use crate::models::common::pagination::PaginationInfo;
use crate::models::common::serialization::serialize_i64_as_string;
use crate::models::files::responses::FileInfo;
use crate::models::homeworks::entities::Homework;
use serde::Serialize;
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/homework.ts")]
pub struct HomeworkCreator {
    #[serde(serialize_with = "serialize_i64_as_string")]
    #[ts(type = "string")]
    pub id: i64,
    pub username: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/homework.ts")]
pub struct HomeworkResponse {
    #[serde(serialize_with = "serialize_i64_as_string")]
    #[ts(type = "string")]
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub content: Option<String>,
    pub deadline: Option<String>,
    pub max_score: f64,
    pub allow_late_submission: bool,
    pub attachments: Vec<String>,
    pub submission_count: i32,
    pub status: String,
    pub created_by: HomeworkCreator,
    pub created_at: String,
    pub updated_at: String,
}

/// 带创建者信息的作业（用于列表，旧版兼容）
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/homework.ts")]
pub struct HomeworkWithCreator {
    #[serde(flatten)]
    pub homework: Homework,
    pub creator: Option<HomeworkCreator>,
}

/// 我的提交摘要（用于作业列表显示提交状态）
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/homework.ts")]
pub struct MySubmissionSummary {
    #[serde(serialize_with = "serialize_i64_as_string")]
    #[ts(type = "string")]
    pub id: i64,
    pub version: i32,
    pub status: String,
    pub is_late: bool,
    pub score: Option<f64>,
}

/// 作业统计摘要（用于教师视角列表显示）
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/homework.ts")]
pub struct HomeworkStatsSummary {
    /// 班级学生总数
    #[serde(serialize_with = "serialize_i64_as_string")]
    #[ts(type = "string")]
    pub total_students: i64,
    /// 已提交人数
    #[serde(serialize_with = "serialize_i64_as_string")]
    #[ts(type = "string")]
    pub submitted_count: i64,
    /// 已评分人数
    #[serde(serialize_with = "serialize_i64_as_string")]
    #[ts(type = "string")]
    pub graded_count: i64,
}

/// 作业列表项（包含创建者和我的提交状态）
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/homework.ts")]
pub struct HomeworkListItem {
    #[serde(flatten)]
    pub homework: Homework,
    pub creator: Option<HomeworkCreator>,
    /// 当前用户的最新提交（仅学生视角有值）
    pub my_submission: Option<MySubmissionSummary>,
    /// 作业统计摘要（仅教师/管理员视角且请求 include_stats=true 时有值）
    pub stats_summary: Option<HomeworkStatsSummary>,
}

/// 作业详情（包含附件和创建者）
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/homework.ts")]
pub struct HomeworkDetail {
    #[serde(flatten)]
    pub homework: Homework,
    pub attachments: Vec<FileInfo>,
    pub creator: Option<HomeworkCreator>,
}

#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/homework.ts")]
pub struct HomeworkListResponse {
    pub items: Vec<HomeworkListItem>,
    pub pagination: PaginationInfo,
}

/// 学生作业统计响应
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/homework.ts")]
pub struct MyHomeworkStatsResponse {
    /// 待完成（未提交）
    #[serde(serialize_with = "serialize_i64_as_string")]
    #[ts(type = "string")]
    pub pending: i64,
    /// 已提交待批改
    #[serde(serialize_with = "serialize_i64_as_string")]
    #[ts(type = "string")]
    pub submitted: i64,
    /// 已批改
    #[serde(serialize_with = "serialize_i64_as_string")]
    #[ts(type = "string")]
    pub graded: i64,
    /// 总数
    #[serde(serialize_with = "serialize_i64_as_string")]
    #[ts(type = "string")]
    pub total: i64,
}

/// 教师作业统计响应
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/homework.ts")]
pub struct TeacherHomeworkStatsResponse {
    /// 作业总数
    #[serde(serialize_with = "serialize_i64_as_string")]
    #[ts(type = "string")]
    pub total_homeworks: i64,
    /// 待批改提交数
    #[serde(serialize_with = "serialize_i64_as_string")]
    #[ts(type = "string")]
    pub pending_review: i64,
    /// 总提交数
    #[serde(serialize_with = "serialize_i64_as_string")]
    #[ts(type = "string")]
    pub total_submissions: i64,
    /// 已批改数
    #[serde(serialize_with = "serialize_i64_as_string")]
    #[ts(type = "string")]
    pub graded_submissions: i64,
}

/// 跨班级作业列表响应
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/homework.ts")]
pub struct AllHomeworksResponse {
    pub items: Vec<HomeworkListItem>,
    pub pagination: PaginationInfo,
    /// 服务器时间（ISO 8601），用于前端统一时间判断
    pub server_time: String,
}
