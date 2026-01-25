use super::entities::User;
use crate::models::common::PaginationInfo;
use serde::Serialize;
use ts_rs::TS;

// 用户响应
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/user.ts")]
pub struct UserResponse {
    pub user: User,
}

// 用户列表响应
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/user.ts")]
pub struct UserListResponse {
    pub items: Vec<User>,
    pub pagination: PaginationInfo,
}

// 导入行错误
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/user.ts")]
pub struct ImportRowError {
    pub row: usize,
    pub field: String,
    pub message: String,
}

// 用户导入响应
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/user.ts")]
pub struct UserImportResponse {
    pub total: usize,
    pub success: usize,
    pub skipped: usize,
    pub failed: usize,
    pub errors: Vec<ImportRowError>,
}
