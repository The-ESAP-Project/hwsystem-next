use serde::Serialize;
use ts_rs::TS;

use super::entities::Notification;
use crate::models::common::pagination::PaginationInfo;
use crate::models::common::serialization::serialize_i64_as_string;

/// 通知列表响应
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/notification.ts")]
pub struct NotificationListResponse {
    pub items: Vec<Notification>,
    pub pagination: PaginationInfo,
}

/// 未读通知数量响应
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/notification.ts")]
pub struct UnreadCountResponse {
    #[serde(serialize_with = "serialize_i64_as_string")]
    #[ts(type = "string")]
    pub unread_count: i64,
}

/// 标记全部已读响应
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/notification.ts")]
pub struct MarkAllReadResponse {
    #[serde(serialize_with = "serialize_i64_as_string")]
    #[ts(type = "string")]
    pub marked_count: i64,
}
