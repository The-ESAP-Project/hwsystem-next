use crate::models::common::PaginationQuery;
use crate::models::common::serialization::{
    deserialize_option_string_to_i64, deserialize_string_to_i64,
};
use serde::Deserialize;
use ts_rs::TS;

/// 创建评分请求
#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/grade.ts")]
pub struct CreateGradeRequest {
    #[serde(deserialize_with = "deserialize_string_to_i64")]
    #[ts(type = "string")]
    pub submission_id: i64,
    pub score: f64,
    pub comment: Option<String>,
}

/// 更新评分请求
#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/grade.ts")]
pub struct UpdateGradeRequest {
    pub score: Option<f64>,
    pub comment: Option<String>,
}

/// 评分列表查询参数
#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/grade.ts")]
pub struct GradeListQuery {
    #[serde(flatten)]
    #[ts(flatten)]
    pub pagination: PaginationQuery,
    #[serde(default, deserialize_with = "deserialize_option_string_to_i64")]
    #[ts(type = "string | null")]
    pub submission_id: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_i64")]
    #[ts(type = "string | null")]
    pub grader_id: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_option_string_to_i64")]
    #[ts(type = "string | null")]
    pub homework_id: Option<i64>,
}
