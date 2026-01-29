use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::models::common::serialization::serialize_i64_as_string;

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/class.ts")]
pub struct Class {
    // 班级ID
    #[serde(serialize_with = "serialize_i64_as_string")]
    #[ts(type = "string")]
    pub id: i64,
    // 班级名称
    pub name: String,
    // 班级描述
    pub description: Option<String>,
    // 教师ID
    #[serde(serialize_with = "serialize_i64_as_string")]
    #[ts(type = "string")]
    pub teacher_id: i64,
    // 邀请码
    pub invite_code: String,
    // 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    // 更新时间
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
