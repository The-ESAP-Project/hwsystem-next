use serde::Deserialize;
use ts_rs::TS;

/// 更新配置请求
#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/system.ts")]
pub struct UpdateSettingRequest {
    pub value: String,
}

/// 批量更新配置请求
#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/system.ts")]
pub struct BatchUpdateSettingsRequest {
    pub settings: Vec<UpdateSettingItem>,
}

#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/system.ts")]
pub struct UpdateSettingItem {
    pub key: String,
    pub value: String,
}

/// 审计日志查询参数
#[derive(Debug, Clone, Deserialize, Default, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/system.ts")]
pub struct SettingAuditQuery {
    pub key: Option<String>,
    pub page: Option<i64>,
    pub size: Option<i64>,
}

/// WebSocket 连接查询参数
#[derive(Debug, Deserialize)]
pub struct WsQuery {
    pub token: String,
}
