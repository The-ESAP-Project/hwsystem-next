use serde::Serialize;
use ts_rs::TS;

/// 文件信息（用于附件列表展示）
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/file.ts")]
pub struct FileInfo {
    /// 下载令牌
    pub download_token: String,
    /// 原始文件名
    pub original_name: String,
    /// 文件大小(字节)
    pub file_size: i64,
    /// 文件类型 (MIME)
    pub file_type: String,
}

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
