use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::models::common::serialization::{
    serialize_i64_as_string, serialize_option_i64_as_string,
};

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/file.ts")]
pub struct File {
    // 文件的唯一标识符
    #[serde(serialize_with = "serialize_i64_as_string")]
    #[ts(type = "string")]
    pub id: i64,
    // 上传者用户ID
    #[serde(serialize_with = "serialize_option_i64_as_string")]
    #[ts(type = "string | null")]
    pub user_id: Option<i64>,
    // 原始文件名
    pub original_name: String,
    // 存储文件名（防冲突）
    pub stored_name: String,
    // 文件类型（MIME）
    pub file_type: String,
    // 文件大小（以字节为单位）
    #[serde(serialize_with = "serialize_i64_as_string")]
    #[ts(type = "string")]
    pub file_size: i64,
    // 文件存储路径
    pub file_path: String,
    // 下载令牌
    pub download_token: String,
    // 引用计数
    pub citation_count: i32,
    // 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
}
