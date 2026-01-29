use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::models::common::serialization::{
    serialize_i64_as_string, serialize_option_i64_as_string,
};

/// 配置值类型
#[derive(Debug, Clone, Serialize, PartialEq, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../frontend/src/types/generated/system.ts")]
pub enum SettingValueType {
    String,
    Integer,
    Boolean,
    JsonArray,
}

impl<'de> Deserialize<'de> for SettingValueType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "string" => Ok(SettingValueType::String),
            "integer" => Ok(SettingValueType::Integer),
            "boolean" => Ok(SettingValueType::Boolean),
            "json_array" => Ok(SettingValueType::JsonArray),
            _ => Err(serde::de::Error::custom(format!(
                "无效的配置值类型: '{s}'. 支持的类型: string, integer, boolean, json_array"
            ))),
        }
    }
}

impl std::fmt::Display for SettingValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SettingValueType::String => write!(f, "string"),
            SettingValueType::Integer => write!(f, "integer"),
            SettingValueType::Boolean => write!(f, "boolean"),
            SettingValueType::JsonArray => write!(f, "json_array"),
        }
    }
}

impl std::str::FromStr for SettingValueType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "string" => Ok(SettingValueType::String),
            "integer" => Ok(SettingValueType::Integer),
            "boolean" => Ok(SettingValueType::Boolean),
            "json_array" => Ok(SettingValueType::JsonArray),
            _ => Err(format!("Invalid setting value type: {s}")),
        }
    }
}

/// 已知配置键
#[derive(Debug, Clone, PartialEq)]
pub enum KnownSettingKey {
    SystemName,
    AccessTokenExpiry,
    RefreshTokenExpiry,
    RefreshTokenRememberMeExpiry,
    UploadMaxSize,
    UploadAllowedTypes,
    CorsAllowedOrigins,
    CorsMaxAge,
}

impl KnownSettingKey {
    pub fn as_str(&self) -> &'static str {
        match self {
            KnownSettingKey::SystemName => "app.system_name",
            KnownSettingKey::AccessTokenExpiry => "jwt.access_token_expiry",
            KnownSettingKey::RefreshTokenExpiry => "jwt.refresh_token_expiry",
            KnownSettingKey::RefreshTokenRememberMeExpiry => "jwt.refresh_token_remember_me_expiry",
            KnownSettingKey::UploadMaxSize => "upload.max_size",
            KnownSettingKey::UploadAllowedTypes => "upload.allowed_types",
            KnownSettingKey::CorsAllowedOrigins => "cors.allowed_origins",
            KnownSettingKey::CorsMaxAge => "cors.max_age",
        }
    }

    pub fn value_type(&self) -> SettingValueType {
        match self {
            KnownSettingKey::SystemName => SettingValueType::String,
            KnownSettingKey::AccessTokenExpiry => SettingValueType::Integer,
            KnownSettingKey::RefreshTokenExpiry => SettingValueType::Integer,
            KnownSettingKey::RefreshTokenRememberMeExpiry => SettingValueType::Integer,
            KnownSettingKey::UploadMaxSize => SettingValueType::Integer,
            KnownSettingKey::UploadAllowedTypes => SettingValueType::JsonArray,
            KnownSettingKey::CorsAllowedOrigins => SettingValueType::JsonArray,
            KnownSettingKey::CorsMaxAge => SettingValueType::Integer,
        }
    }

    pub fn all() -> Vec<Self> {
        vec![
            KnownSettingKey::SystemName,
            KnownSettingKey::AccessTokenExpiry,
            KnownSettingKey::RefreshTokenExpiry,
            KnownSettingKey::RefreshTokenRememberMeExpiry,
            KnownSettingKey::UploadMaxSize,
            KnownSettingKey::UploadAllowedTypes,
            KnownSettingKey::CorsAllowedOrigins,
            KnownSettingKey::CorsMaxAge,
        ]
    }
}

impl std::str::FromStr for KnownSettingKey {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "app.system_name" => Ok(KnownSettingKey::SystemName),
            "jwt.access_token_expiry" => Ok(KnownSettingKey::AccessTokenExpiry),
            "jwt.refresh_token_expiry" => Ok(KnownSettingKey::RefreshTokenExpiry),
            "jwt.refresh_token_remember_me_expiry" => {
                Ok(KnownSettingKey::RefreshTokenRememberMeExpiry)
            }
            "upload.max_size" => Ok(KnownSettingKey::UploadMaxSize),
            "upload.allowed_types" => Ok(KnownSettingKey::UploadAllowedTypes),
            "cors.allowed_origins" => Ok(KnownSettingKey::CorsAllowedOrigins),
            "cors.max_age" => Ok(KnownSettingKey::CorsMaxAge),
            _ => Err(format!("Unknown setting key: {s}")),
        }
    }
}

/// 系统设置实体
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/system.ts")]
pub struct SystemSetting {
    pub key: String,
    pub value: String,
    pub value_type: SettingValueType,
    pub description: Option<String>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    #[serde(serialize_with = "serialize_option_i64_as_string")]
    #[ts(type = "string | null")]
    pub updated_by: Option<i64>,
}

/// 设置审计日志实体
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/system.ts")]
pub struct SettingAudit {
    #[serde(serialize_with = "serialize_i64_as_string")]
    #[ts(type = "string")]
    pub id: i64,
    pub setting_key: String,
    pub old_value: Option<String>,
    pub new_value: String,
    #[serde(serialize_with = "serialize_i64_as_string")]
    #[ts(type = "string")]
    pub changed_by: i64,
    pub changed_at: chrono::DateTime<chrono::Utc>,
    pub ip_address: Option<String>,
}
