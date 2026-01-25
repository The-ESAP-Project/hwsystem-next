use crate::models::common::PaginationQuery;
use serde::Deserialize;
use ts_rs::TS;

// 班级查询参数（来自HTTP请求）
#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/class.ts")]
pub struct ClassQueryParams {
    #[serde(flatten)]
    #[ts(flatten)]
    pub pagination: PaginationQuery,
    pub search: Option<String>,
}

// 创建班级请求
//
// # teacher_id 字段说明
// - **教师创建**：可选字段，不填写则自动使用当前登录教师的 ID
// - **管理员创建**：必填字段，用于指定负责该班级的教师
//
// # 权限验证
// - 教师：如果指定 teacher_id，必须等于自己的 ID
// - 管理员：必须指定 teacher_id，且该用户必须是教师角色
#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/class.ts")]
pub struct CreateClassRequest {
    pub teacher_id: Option<i64>,
    pub name: String,
    pub description: Option<String>,
}

// 更新班级请求
#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/class.ts")]
pub struct UpdateClassRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    #[ts(skip)]
    pub _teacher_id: Option<i64>, // TODO: 未来计划实现班级转让
}

// 班级列表查询参数（用于存储层）
#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/class.ts")]
pub struct ClassListQuery {
    pub page: Option<i64>,
    pub size: Option<i64>,
    pub teacher_id: Option<i64>,
    pub search: Option<String>,
}
