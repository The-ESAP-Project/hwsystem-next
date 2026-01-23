// 公共模块
pub mod common;

// 登录模块
pub mod auth;

// 业务模块
pub mod users;

// 文件模块
pub mod files;

// 班级模块
pub mod classes;

// 班级成员模块
pub mod class_users;

// 作业模块
pub mod homeworks;

// 提交模块
pub mod submissions;

// 评分模块
pub mod grades;

// 系统模块
pub mod system;

// 重新导出通用类型
pub use common::{ApiResponse, ErrorCode, PaginationInfo};

// 系统模块
#[derive(Clone, Debug)]
pub struct AppStartTime {
    pub start_datetime: chrono::DateTime<chrono::Utc>,
}
