use serde::Serialize;
use ts_rs::TS;

use crate::models::PaginationInfo;

use super::entities::Grade;

/// 评分者信息
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/grade.ts")]
pub struct Grader {
    pub id: i64,
    pub username: String,
    pub display_name: Option<String>,
}

/// 评分响应
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/grade.ts")]
pub struct GradeResponse {
    pub id: i64,
    pub submission_id: i64,
    pub grader: Grader,
    pub score: f64,
    pub comment: Option<String>,
    pub graded_at: String,
}

/// 评分列表响应
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/grade.ts")]
pub struct GradeListResponse {
    pub items: Vec<Grade>,
    pub pagination: PaginationInfo,
}
