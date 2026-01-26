use serde::Serialize;
use ts_rs::TS;

use crate::models::PaginationInfo;
use crate::models::files::responses::FileInfo;

/// 提交者信息
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/submission.ts")]
pub struct SubmissionCreator {
    pub id: i64,
    pub username: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}

/// 提交关联的作业信息
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/submission.ts")]
pub struct SubmissionHomeworkInfo {
    pub id: i64,
    pub title: String,
    pub max_score: f64,
    pub deadline: Option<String>,
}

/// 提交响应
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/submission.ts")]
pub struct SubmissionResponse {
    pub id: i64,
    pub homework_id: i64,
    pub creator: SubmissionCreator,
    pub content: String,
    pub attachments: Vec<FileInfo>,
    pub status: String,
    pub submitted_at: String,
    pub grade: Option<SubmissionGradeInfo>,
    pub version: i32,
    pub is_late: bool,
    pub homework: Option<SubmissionHomeworkInfo>,
}

/// 提交中的评分信息
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/submission.ts")]
pub struct SubmissionGradeInfo {
    pub id: i64,
    pub score: f64,
    pub comment: Option<String>,
    pub graded_at: String,
}

/// 提交列表项（包含提交者信息）
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/submission.ts")]
pub struct SubmissionListItem {
    pub id: i64,
    pub homework_id: i64,
    pub creator_id: i64,
    pub creator: SubmissionCreator,
    pub version: i32,
    pub content: Option<String>,
    pub status: String,
    pub is_late: bool,
    pub submitted_at: String,
}

/// 提交列表响应
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/submission.ts")]
pub struct SubmissionListResponse {
    pub items: Vec<SubmissionListItem>,
    pub pagination: PaginationInfo,
}

/// 用户提交历史项（包含评分信息）
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/submission.ts")]
pub struct UserSubmissionHistoryItem {
    pub id: i64,
    pub homework_id: i64,
    pub version: i32,
    pub content: Option<String>,
    pub status: String,
    pub is_late: bool,
    pub submitted_at: String,
    pub attachments: Vec<FileInfo>,
    pub grade: Option<SubmissionGradeInfo>,
}

/// 用户提交历史响应（无分页）
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/submission.ts")]
pub struct UserSubmissionHistoryResponse {
    pub items: Vec<UserSubmissionHistoryItem>,
}

// ============ 提交概览相关（按学生聚合）============

/// 最新提交信息（概览用）
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/submission.ts")]
pub struct LatestSubmissionInfo {
    pub id: i64,
    pub version: i32,
    pub status: String,
    pub is_late: bool,
    pub submitted_at: String,
}

/// 提交概览项（按学生聚合）
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/submission.ts")]
pub struct SubmissionSummaryItem {
    pub creator: SubmissionCreator,
    pub latest_submission: LatestSubmissionInfo,
    pub grade: Option<SubmissionGradeInfo>,
    pub total_versions: i32,
}

/// 提交概览响应
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/submission.ts")]
pub struct SubmissionSummaryResponse {
    pub items: Vec<SubmissionSummaryItem>,
    pub pagination: PaginationInfo,
}
