use serde::Serialize;
use ts_rs::TS;

use crate::models::common::serialization::serialize_i64_as_string;

/// 作业统计响应
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/homework.ts")]
pub struct HomeworkStatsResponse {
    #[serde(serialize_with = "serialize_i64_as_string")]
    #[ts(type = "string")]
    pub homework_id: i64,
    #[serde(serialize_with = "serialize_i64_as_string")]
    #[ts(type = "string")]
    pub total_students: i64,
    #[serde(serialize_with = "serialize_i64_as_string")]
    #[ts(type = "string")]
    pub submitted_count: i64,
    #[serde(serialize_with = "serialize_i64_as_string")]
    #[ts(type = "string")]
    pub graded_count: i64,
    #[serde(serialize_with = "serialize_i64_as_string")]
    #[ts(type = "string")]
    pub late_count: i64,
    pub submission_rate: f64,
    pub score_stats: Option<ScoreStats>,
    pub score_distribution: Vec<ScoreRange>,
    pub unsubmitted_students: Vec<UnsubmittedStudent>,
}

/// 分数统计
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/homework.ts")]
pub struct ScoreStats {
    pub average: f64,
    pub max: f64,
    pub min: f64,
}

/// 分数区间
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/homework.ts")]
pub struct ScoreRange {
    pub range: String,
    #[serde(serialize_with = "serialize_i64_as_string")]
    #[ts(type = "string")]
    pub count: i64,
}

/// 未提交学生
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/homework.ts")]
pub struct UnsubmittedStudent {
    #[serde(serialize_with = "serialize_i64_as_string")]
    #[ts(type = "string")]
    pub id: i64,
    pub username: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}
