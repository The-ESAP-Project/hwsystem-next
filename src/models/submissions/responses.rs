use serde::Serialize;
use ts_rs::TS;

use crate::models::PaginationInfo;
use crate::models::files::responses::FileInfo;

use super::entities::Submission;

/// 提交者信息
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/submission.ts")]
pub struct SubmissionCreator {
    pub id: i64,
    pub username: String,
    pub display_name: Option<String>,
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
}

/// 提交中的评分信息
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/submission.ts")]
pub struct SubmissionGradeInfo {
    pub score: f64,
    pub comment: Option<String>,
    pub graded_at: String,
}

/// 提交列表响应
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/submission.ts")]
pub struct SubmissionListResponse {
    pub items: Vec<Submission>,
    pub pagination: PaginationInfo,
}
