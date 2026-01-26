use crate::models::common::PaginationQuery;
use serde::Deserialize;
use ts_rs::TS;

/// 创建提交请求
#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/submission.ts")]
pub struct CreateSubmissionRequest {
    pub homework_id: i64,
    pub content: String,
    pub attachments: Option<Vec<String>>,
}

/// 更新提交请求
#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/submission.ts")]
pub struct UpdateSubmissionRequest {
    pub content: Option<String>,
    pub attachments: Option<Vec<String>>,
}

/// 提交列表查询参数
#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/submission.ts")]
pub struct SubmissionListParams {
    #[serde(flatten)]
    #[ts(flatten)]
    pub pagination: PaginationQuery,
    pub homework_id: Option<i64>,
    pub creator_id: Option<i64>,
    pub status: Option<String>,
}

/// 提交列表存储层查询参数
#[derive(Debug, Clone, Deserialize)]
pub struct SubmissionListQuery {
    pub page: Option<i64>,
    pub size: Option<i64>,
    pub homework_id: Option<i64>,
    pub creator_id: Option<i64>,
    pub status: Option<String>,
}

/// 提交概览分页查询参数
#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/submission.ts")]
pub struct SubmissionSummaryQuery {
    pub page: Option<i64>,
    pub size: Option<i64>,
    /// 筛选是否已批改：true=已批改，false=待批改，None=全部
    pub graded: Option<bool>,
}
