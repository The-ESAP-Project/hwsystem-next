use serde::Serialize;
use ts_rs::TS;

/// FileAttachment
#[derive(Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/file.ts")]
pub struct FileUploadResponse {
    /// 下载令牌
    pub download_token: String,
    /// 原始文件名
    pub file_name: String,
    /// 文件大小(字节)
    pub size: i64,
    /// 文件类型
    pub content_type: String,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
}
