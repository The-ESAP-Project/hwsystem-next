use super::entities::{UserRole, UserStatus};
use crate::models::common::PaginationQuery;
use serde::Deserialize;
use ts_rs::TS;

// 用户查询参数（来自HTTP请求）
#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/user.ts")]
pub struct UserListParams {
    #[serde(flatten)]
    #[ts(flatten)]
    pub pagination: PaginationQuery,
    pub role: Option<UserRole>,
    pub status: Option<UserStatus>,
    pub search: Option<String>,
}

// 用户创建请求
#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/user.ts")]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub role: UserRole,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}

// 用户更新请求
#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/user.ts")]
pub struct UpdateUserRequest {
    pub email: Option<String>,
    pub password: Option<String>,
    pub role: Option<UserRole>,
    pub status: Option<UserStatus>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}

// 用户列表查询参数（用于存储层）
#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/user.ts")]
pub struct UserListQuery {
    pub page: Option<i64>,
    pub size: Option<i64>,
    pub role: Option<UserRole>,
    pub status: Option<UserStatus>,
    pub search: Option<String>,
}

// 用户导出参数
#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/user.ts")]
pub struct UserExportParams {
    #[serde(default = "default_export_format")]
    pub format: String,
    pub role: Option<UserRole>,
    pub status: Option<UserStatus>,
    pub search: Option<String>,
}

fn default_export_format() -> String {
    "csv".to_string()
}

// 导入模板参数
#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/user.ts")]
pub struct ImportTemplateParams {
    #[serde(default = "default_export_format")]
    pub format: String,
}
