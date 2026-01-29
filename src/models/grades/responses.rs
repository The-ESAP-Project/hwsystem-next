use serde::Serialize;
use ts_rs::TS;

use crate::models::PaginationInfo;
use crate::models::common::serialization::serialize_i64_as_string;

use super::entities::Grade;

/// 评分者信息
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/grade.ts")]
pub struct Grader {
    #[serde(serialize_with = "serialize_i64_as_string")]
    #[ts(type = "string")]
    pub id: i64,
    pub username: String,
    pub display_name: Option<String>,
}

/// 评分响应
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/grade.ts")]
pub struct GradeResponse {
    #[serde(serialize_with = "serialize_i64_as_string")]
    #[ts(type = "string")]
    pub id: i64,
    #[serde(serialize_with = "serialize_i64_as_string")]
    #[ts(type = "string")]
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
