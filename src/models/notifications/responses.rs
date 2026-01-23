use serde::Serialize;
use ts_rs::TS;

use super::entities::Notification;
use crate::models::common::pagination::PaginationInfo;

/// 通知列表响应
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/notification.ts")]
pub struct NotificationListResponse {
    pub notifications: Vec<Notification>,
    pub pagination: PaginationInfo,
}

/// 未读通知数量响应
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/notification.ts")]
pub struct UnreadCountResponse {
    pub count: i64,
}

/// 标记全部已读响应
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/notification.ts")]
pub struct MarkAllReadResponse {
    pub marked_count: i64,
}
