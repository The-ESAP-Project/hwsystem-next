use crate::models::common::pagination::PaginationQuery;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use ts_rs::TS;

/// 创建作业请求
#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/homework.ts")]
pub struct CreateHomeworkRequest {
    pub class_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub max_score: Option<f64>,
    pub deadline: Option<DateTime<Utc>>, // ISO 8601 格式，如 "2026-01-24T12:00:00Z"
    pub allow_late: Option<bool>,
    pub attachments: Option<Vec<String>>, // download_token 列表
}

/// 更新作业请求
#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/homework.ts")]
pub struct UpdateHomeworkRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub max_score: Option<f64>,
    pub deadline: Option<DateTime<Utc>>, // ISO 8601 格式
    pub allow_late: Option<bool>,
    pub attachments: Option<Vec<String>>, // download_token 列表
}

/// 作业列表查询参数（HTTP 请求）
#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/homework.ts")]
pub struct HomeworkListParams {
    #[serde(flatten)]
    #[ts(flatten)]
    pub pagination: PaginationQuery,
    pub class_id: Option<i64>,
    pub created_by: Option<i64>,
    pub search: Option<String>,
}

/// 作业列表查询参数（存储层）
#[derive(Debug, Clone, Deserialize)]
pub struct HomeworkListQuery {
    pub page: Option<i64>,
    pub size: Option<i64>,
    pub class_id: Option<i64>,
    pub created_by: Option<i64>,
    pub search: Option<String>,
}
