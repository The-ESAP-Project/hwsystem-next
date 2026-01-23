use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// 评分实体
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/grade.ts")]
pub struct Grade {
    pub id: i64,
    pub submission_id: i64,
    pub grader_id: i64,
    pub score: f64,
    pub comment: Option<String>,
    pub graded_at: chrono::DateTime<chrono::Utc>,
}
