use super::entities::Class;
use crate::models::common::PaginationInfo;
use serde::Serialize;
use ts_rs::TS;

/// 教师简要信息
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/class.ts")]
pub struct TeacherInfo {
    pub id: i64,
    pub username: String,
    pub display_name: Option<String>,
}

/// 班级详情（包含教师信息和成员数量）
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/class.ts")]
pub struct ClassDetail {
    #[serde(flatten)]
    pub class: Class,
    pub teacher: TeacherInfo,
    pub member_count: i64,
}

// 班级列表响应
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/class.ts")]
pub struct ClassListResponse {
    pub pagination: PaginationInfo,
    pub items: Vec<Class>,
}

// 班级详情列表响应
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/class.ts")]
pub struct ClassDetailListResponse {
    pub pagination: PaginationInfo,
    pub items: Vec<ClassDetail>,
}
