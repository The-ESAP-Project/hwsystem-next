use serde::Serialize;
use ts_rs::TS;

use super::entities::{SettingAudit, SystemSetting};
use crate::models::common::PaginationInfo;
use crate::models::common::serialization::serialize_u64_as_string;

#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/system.ts")]
pub struct SystemSettingsResponse {
    pub system_name: String, // 系统名称
    #[serde(serialize_with = "serialize_u64_as_string")]
    #[ts(type = "string")]
    pub max_file_size: u64, // 单文件最大字节数
    pub allowed_file_types: Vec<String>, // 允许的文件类型
    pub environment: String, // 运行环境
    pub log_level: String,   // 日志级别
}

/// WebSocket 状态响应
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/system.ts")]
pub struct WebSocketStatusResponse {
    pub online_users: usize,
    pub status: String,
}

/// 管理员配置列表响应
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/system.ts")]
pub struct AdminSettingsListResponse {
    pub settings: Vec<SystemSetting>,
}

/// 单个配置响应
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/system.ts")]
pub struct SettingResponse {
    pub setting: SystemSetting,
}

/// 审计日志列表响应
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/system.ts")]
pub struct SettingAuditListResponse {
    pub audits: Vec<SettingAudit>,
    pub pagination: PaginationInfo,
}
