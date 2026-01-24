use serde::Serialize;
use ts_rs::TS;

/// 作业统计响应
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/homework.ts")]
pub struct HomeworkStatsResponse {
    pub homework_id: i64,
    pub total_students: i64,
    pub submitted_count: i64,
    pub graded_count: i64,
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
    pub count: i64,
}

/// 未提交学生
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/homework.ts")]
pub struct UnsubmittedStudent {
    pub id: i64,
    pub username: String,
    pub display_name: Option<String>,
}
