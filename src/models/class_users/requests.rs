use crate::models::{class_users::entities::ClassUserRole, common::PaginationQuery};
use serde::Deserialize;
use ts_rs::TS;

// 加入班级请求
#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/class-user.ts")]
pub struct JoinClassRequest {
    pub invite_code: String,
}

#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/class-user.ts")]
pub struct UpdateClassUserRequest {
    pub role: Option<ClassUserRole>, // 更新用户角色
}

#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/class-user.ts")]
pub struct ClassUserListParams {
    #[serde(flatten)]
    #[ts(flatten)]
    pub pagination: PaginationQuery,
    pub search: Option<String>,
    pub role: Option<ClassUserRole>,
}

// 班级列表查询参数（用于存储层）
#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/class-user.ts")]
pub struct ClassUserQuery {
    pub page: Option<i64>,
    pub size: Option<i64>,
    pub search: Option<String>,
    pub role: Option<ClassUserRole>,
}
