use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// 提交状态
#[derive(Debug, Clone, Serialize, PartialEq, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../frontend/src/types/generated/submission.ts")]
pub enum SubmissionStatus {
    Pending, // 待批改
    Graded,  // 已批改
    Late,    // 迟交
}

impl SubmissionStatus {
    pub const PENDING: &'static str = "pending";
    pub const GRADED: &'static str = "graded";
    pub const LATE: &'static str = "late";
}

impl<'de> Deserialize<'de> for SubmissionStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "pending" => Ok(SubmissionStatus::Pending),
            "graded" => Ok(SubmissionStatus::Graded),
            "late" => Ok(SubmissionStatus::Late),
            _ => Err(serde::de::Error::custom(format!(
                "无效的提交状态: '{s}'. 支持的状态: pending, graded, late"
            ))),
        }
    }
}

impl std::fmt::Display for SubmissionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SubmissionStatus::Pending => write!(f, "pending"),
            SubmissionStatus::Graded => write!(f, "graded"),
            SubmissionStatus::Late => write!(f, "late"),
        }
    }
}

/// 提交实体
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/submission.ts")]
pub struct Submission {
    pub id: i64,
    pub homework_id: i64,
    pub creator_id: i64,
    pub content: String,
    pub attachments: Option<String>,
    pub submitted_at: chrono::DateTime<chrono::Utc>,
}
