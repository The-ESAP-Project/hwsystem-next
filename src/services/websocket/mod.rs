/*!
 * WebSocket 实时通知服务
 *
 * 此服务用于建立 WebSocket 连接，实时推送通知给用户。
 *
 * ## 使用方法
 *
 * 客户端通过以下 URL 连接：
 * ```
 * ws://host/api/v1/ws?token=<access_token>
 * ```
 *
 * ## 消息格式
 *
 * ### 服务端推送
 * ```json
 * {
 *     "type": "notification",
 *     "payload": {
 *         "id": "uuid",
 *         "type": "homework_created",
 *         "title": "新作业发布",
 *         "content": "《数据结构》作业已发布",
 *         "reference_type": "homework",
 *         "reference_id": "uuid",
 *         "created_at": "2026-01-24T12:00:00Z"
 *     }
 * }
 * ```
 *
 * ### 心跳
 * ```json
 * {"type": "ping"}
 * {"type": "pong"}
 * ```
 */

use actix_ws::Message;
use dashmap::DashMap;
use futures_util::StreamExt;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tracing::{debug, info, warn};

use crate::models::notifications::entities::Notification;

/// 全局连接管理器
static CONNECTION_MANAGER: Lazy<ConnectionManager> = Lazy::new(ConnectionManager::new);

/// WebSocket 消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsMessage {
    /// 通知消息
    Notification { payload: NotificationPayload },
    /// 心跳请求
    Ping,
    /// 心跳响应
    Pong,
    /// 连接成功
    Connected { user_id: i64 },
    /// 错误消息
    Error { message: String },
}

/// 通知载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPayload {
    pub id: i64,
    pub notification_type: String,
    pub title: String,
    pub content: Option<String>,
    pub reference_type: Option<String>,
    pub reference_id: Option<i64>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<Notification> for NotificationPayload {
    fn from(n: Notification) -> Self {
        Self {
            id: n.id,
            notification_type: n.notification_type.to_string(),
            title: n.title,
            content: n.content,
            reference_type: n.reference_type.map(|r| r.to_string()),
            reference_id: n.reference_id,
            created_at: n.created_at,
        }
    }
}

/// 连接管理器
pub struct ConnectionManager {
    /// 用户 ID -> 广播发送器
    connections: DashMap<i64, broadcast::Sender<WsMessage>>,
}

impl ConnectionManager {
    fn new() -> Self {
        Self {
            connections: DashMap::new(),
        }
    }

    /// 获取全局实例
    pub fn get() -> &'static Self {
        &CONNECTION_MANAGER
    }

    /// 注册用户连接
    pub fn register(&self, user_id: i64) -> broadcast::Receiver<WsMessage> {
        let entry = self.connections.entry(user_id).or_insert_with(|| {
            let (tx, _) = broadcast::channel(100);
            tx
        });
        entry.subscribe()
    }

    /// 移除用户连接
    pub fn unregister(&self, user_id: i64) {
        // 只有当没有订阅者时才移除
        if let Some(entry) = self.connections.get(&user_id)
            && entry.receiver_count() == 0
        {
            self.connections.remove(&user_id);
        }
    }

    /// 向指定用户发送通知
    pub fn send_to_user(&self, user_id: i64, message: WsMessage) -> bool {
        if let Some(sender) = self.connections.get(&user_id) {
            sender.send(message).is_ok()
        } else {
            false
        }
    }

    /// 向多个用户发送通知
    pub fn send_to_users(&self, user_ids: &[i64], message: WsMessage) {
        for &user_id in user_ids {
            self.send_to_user(user_id, message.clone());
        }
    }

    /// 推送通知给用户
    pub fn push_notification(&self, user_id: i64, notification: Notification) {
        let message = WsMessage::Notification {
            payload: NotificationPayload::from(notification),
        };
        self.send_to_user(user_id, message);
    }

    /// 获取在线用户数
    pub fn online_count(&self) -> usize {
        self.connections
            .iter()
            .filter(|entry| entry.receiver_count() > 0)
            .count()
    }

    /// 检查用户是否在线
    pub fn is_online(&self, user_id: i64) -> bool {
        self.connections
            .get(&user_id)
            .is_some_and(|s| s.receiver_count() > 0)
    }
}

/// WebSocket 服务
pub struct WebSocketService;

impl WebSocketService {
    /// 处理 WebSocket 连接
    pub async fn handle_connection(
        user_id: i64,
        mut session: actix_ws::Session,
        mut stream: actix_ws::MessageStream,
    ) {
        info!("WebSocket connected for user: {}", user_id);

        // 注册连接
        let mut rx = ConnectionManager::get().register(user_id);

        // 发送连接成功消息
        let connected_msg = WsMessage::Connected { user_id };
        if let Ok(json) = serde_json::to_string(&connected_msg) {
            let _ = session.text(json).await;
        }

        // 心跳间隔
        let heartbeat_interval = std::time::Duration::from_secs(30);
        let mut heartbeat = tokio::time::interval(heartbeat_interval);

        loop {
            tokio::select! {
                // 处理来自客户端的消息
                msg = stream.next() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                                match ws_msg {
                                    WsMessage::Ping => {
                                        let pong = serde_json::to_string(&WsMessage::Pong).unwrap();
                                        if session.text(pong).await.is_err() {
                                            break;
                                        }
                                    }
                                    _ => {
                                        debug!("Received message from user {}: {:?}", user_id, ws_msg);
                                    }
                                }
                            }
                        }
                        Some(Ok(Message::Ping(data))) => {
                            if session.pong(&data).await.is_err() {
                                break;
                            }
                        }
                        Some(Ok(Message::Close(_))) | None => {
                            info!("WebSocket closed for user: {}", user_id);
                            break;
                        }
                        Some(Err(e)) => {
                            warn!("WebSocket error for user {}: {:?}", user_id, e);
                            break;
                        }
                        _ => {}
                    }
                }

                // 处理来自服务器的推送消息
                msg = rx.recv() => {
                    match msg {
                        Ok(ws_msg) => {
                            if let Ok(json) = serde_json::to_string(&ws_msg)
                                && session.text(json).await.is_err() {
                                    break;
                                }
                        }
                        Err(broadcast::error::RecvError::Lagged(n)) => {
                            warn!("WebSocket for user {} lagged by {} messages", user_id, n);
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            break;
                        }
                    }
                }

                // 心跳
                _ = heartbeat.tick() => {
                    if session.ping(b"").await.is_err() {
                        break;
                    }
                }
            }
        }

        // 清理连接
        ConnectionManager::get().unregister(user_id);
        info!("WebSocket disconnected for user: {}", user_id);
    }
}

/// 辅助函数：向用户推送通知
pub fn push_notification_to_user(user_id: i64, notification: Notification) {
    ConnectionManager::get().push_notification(user_id, notification);
}

/// 辅助函数：向多个用户推送通知
pub fn push_notification_to_users(user_ids: &[i64], notification: Notification) {
    let manager = ConnectionManager::get();
    let message = WsMessage::Notification {
        payload: NotificationPayload::from(notification),
    };
    manager.send_to_users(user_ids, message);
}

/// 辅助函数：检查用户是否在线
pub fn is_user_online(user_id: i64) -> bool {
    ConnectionManager::get().is_online(user_id)
}

/// 辅助函数：获取在线用户数
pub fn get_online_count() -> usize {
    ConnectionManager::get().online_count()
}
