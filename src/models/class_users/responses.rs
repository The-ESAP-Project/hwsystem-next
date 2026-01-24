use serde::Serialize;
use ts_rs::TS;

use crate::models::{
    PaginationInfo,
    class_users::entities::{ClassUser, ClassUserRole},
};

/// 用户简要信息
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/class-user.ts")]
pub struct UserInfo {
    pub id: i64,
    pub username: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}

/// 班级成员详情（包含用户信息）
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/class-user.ts")]
pub struct ClassUserDetail {
    pub id: i64,
    pub class_id: i64,
    pub user_id: i64,
    pub role: ClassUserRole,
    pub joined_at: chrono::DateTime<chrono::Utc>,
    pub user: UserInfo,
}

/// 班级学生列表响应
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/class-user.ts")]
pub struct ClassUserListResponse {
    pub pagination: PaginationInfo,
    pub items: Vec<ClassUser>,
}

/// 班级成员详情列表响应
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/class-user.ts")]
pub struct ClassUserDetailListResponse {
    pub pagination: PaginationInfo,
    pub items: Vec<ClassUserDetail>,
}
