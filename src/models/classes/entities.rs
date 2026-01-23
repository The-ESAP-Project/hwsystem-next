use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/class.ts")]
pub struct Class {
    // 班级ID
    pub id: i64,
    // 班级名称
    pub name: String,
    // 班级描述
    pub description: Option<String>,
    // 教师ID
    pub teacher_id: i64,
    // 邀请码
    pub invite_code: String,
    // 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    // 更新时间
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
