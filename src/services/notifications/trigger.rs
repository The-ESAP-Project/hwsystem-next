//! 通知触发辅助模块
//!
//! 提供异步发送通知的函数，不阻塞主业务流程。

use std::sync::Arc;
use tracing::{error, info};

use crate::models::class_users::requests::ClassUserQuery;
use crate::models::notifications::{
    entities::{NotificationType, ReferenceType},
    requests::CreateNotificationRequest,
};
use crate::services::websocket::push_notification_to_users;
use crate::storage::Storage;

/// 批量发送通知（异步，不阻塞）
///
/// 1. 批量创建通知到数据库
/// 2. 通过 WebSocket 推送给在线用户
/// 3. 错误只记录日志，不影响调用方
pub async fn send_notifications(
    storage: Arc<dyn Storage>,
    user_ids: Vec<i64>,
    notification_type: NotificationType,
    title: String,
    content: Option<String>,
    reference_type: Option<ReferenceType>,
    reference_id: Option<i64>,
) {
    if user_ids.is_empty() {
        return;
    }

    let requests: Vec<CreateNotificationRequest> = user_ids
        .iter()
        .map(|&user_id| CreateNotificationRequest {
            user_id,
            notification_type: notification_type.to_string(),
            title: title.clone(),
            content: content.clone(),
            reference_type: reference_type.as_ref().map(|r| r.to_string()),
            reference_id,
        })
        .collect();

    match storage.create_notifications_batch(requests).await {
        Ok(notifications) => {
            info!(
                "Created {} notifications of type {}",
                notifications.len(),
                notification_type
            );

            // WebSocket 推送
            if let Some(first) = notifications.into_iter().next() {
                push_notification_to_users(&user_ids, first);
            }
        }
        Err(e) => {
            error!("Failed to create notifications: {}", e);
        }
    }
}

/// 发送单个通知
pub async fn send_notification(
    storage: Arc<dyn Storage>,
    user_id: i64,
    notification_type: NotificationType,
    title: String,
    content: Option<String>,
    reference_type: Option<ReferenceType>,
    reference_id: Option<i64>,
) {
    send_notifications(
        storage,
        vec![user_id],
        notification_type,
        title,
        content,
        reference_type,
        reference_id,
    )
    .await;
}

/// 获取班级所有学生的 user_id 列表（排除教师角色）
pub async fn get_class_student_ids(storage: &Arc<dyn Storage>, class_id: i64) -> Vec<i64> {
    use crate::models::class_users::entities::ClassUserRole;

    let query = ClassUserQuery {
        page: Some(1),
        size: Some(10000),
        search: None,
        role: None,
    };

    match storage
        .list_class_users_with_pagination(class_id, query)
        .await
    {
        Ok(response) => response
            .items
            .into_iter()
            .filter(|cu| cu.role != ClassUserRole::Teacher)
            .map(|cu| cu.user_id)
            .collect(),
        Err(e) => {
            error!("Failed to get class students for class {}: {}", class_id, e);
            vec![]
        }
    }
}
