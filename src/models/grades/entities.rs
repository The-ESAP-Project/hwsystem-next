use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::models::common::serialization::serialize_i64_as_string;

/// 评分实体
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/grade.ts")]
pub struct Grade {
    #[serde(serialize_with = "serialize_i64_as_string")]
    #[ts(type = "string")]
    pub id: i64,
    #[serde(serialize_with = "serialize_i64_as_string")]
    #[ts(type = "string")]
    pub submission_id: i64,
    #[serde(serialize_with = "serialize_i64_as_string")]
    #[ts(type = "string")]
    pub grader_id: i64,
    pub score: f64,
    pub comment: Option<String>,
    pub graded_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
