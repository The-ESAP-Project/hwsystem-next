use serde::Deserialize;
use ts_rs::TS;

use crate::models::common::pagination::PaginationQuery;

/// 通知列表查询参数
#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/notification.ts")]
pub struct NotificationListQuery {
    /// 是否只显示未读
    pub unread_only: Option<bool>,
    /// 分页参数
    #[serde(flatten)]
    pub pagination: PaginationQuery,
}

/// 创建通知请求
#[derive(Debug, Deserialize)]
pub struct CreateNotificationRequest {
    pub user_id: i64,
    pub notification_type: String,
    pub title: String,
    pub content: Option<String>,
    pub reference_type: Option<String>,
    pub reference_id: Option<i64>,
}
