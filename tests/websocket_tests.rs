//! WebSocket 服务单元测试

use rust_hwsystem_next::services::websocket::{
    ConnectionManager, WsMessage, get_online_count, is_user_online,
};

#[test]
fn test_connection_manager_register() {
    let manager = ConnectionManager::get();

    // 注册用户
    let _rx = manager.register(1);
    assert!(manager.is_online(1));
}

#[test]
fn test_connection_manager_online_count() {
    let manager = ConnectionManager::get();

    // 注册多个用户
    let _rx1 = manager.register(100);
    let _rx2 = manager.register(101);
    let _rx3 = manager.register(102);

    // 检查在线数量
    let count = manager.online_count();
    assert!(count >= 3);
}

#[test]
fn test_send_to_user() {
    let manager = ConnectionManager::get();

    // 注册用户
    let mut rx = manager.register(200);

    // 发送消息
    let message = WsMessage::Ping;
    let sent = manager.send_to_user(200, message);
    assert!(sent);

    // 接收消息
    let received = rx.try_recv();
    assert!(received.is_ok());
}

#[test]
fn test_send_to_offline_user() {
    let manager = ConnectionManager::get();

    // 发送给不存在的用户
    let message = WsMessage::Pong;
    let sent = manager.send_to_user(999999, message);
    assert!(!sent);
}

#[test]
fn test_ws_message_serialization() {
    // 测试消息序列化
    let ping = WsMessage::Ping;
    let json = serde_json::to_string(&ping).unwrap();
    assert!(json.contains("ping"));

    let pong = WsMessage::Pong;
    let json = serde_json::to_string(&pong).unwrap();
    assert!(json.contains("pong"));

    let connected = WsMessage::Connected { user_id: 123 };
    let json = serde_json::to_string(&connected).unwrap();
    assert!(json.contains("connected"));
    assert!(json.contains("123"));

    let error = WsMessage::Error {
        message: "Test error".to_string(),
    };
    let json = serde_json::to_string(&error).unwrap();
    assert!(json.contains("error"));
    assert!(json.contains("Test error"));
}

#[test]
fn test_helper_functions() {
    // 测试辅助函数
    let _ = get_online_count();
    let _ = is_user_online(12345);
}
