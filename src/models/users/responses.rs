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

/// 用户统计响应（合并学生和教师视角）
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/user.ts")]
pub struct UserStatsResponse {
    /// 班级数量
    pub class_count: i64,
    /// 学生总数（教师视角）
    pub total_students: i64,
    /// 待完成作业（学生视角：未提交）
    pub homework_pending: i64,
    /// 已提交作业（学生视角：已提交待批改）
    pub homework_submitted: i64,
    /// 已批改作业（学生视角：已批改）
    pub homework_graded: i64,
    /// 待批改数（教师视角：待批改的提交数）
    pub pending_review: i64,
    /// 服务器时间（ISO 8601）
    pub server_time: String,
}
