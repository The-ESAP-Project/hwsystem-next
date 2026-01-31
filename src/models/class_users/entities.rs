use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::models::common::serialization;

// 用户角色
#[derive(Debug, Clone, Serialize, PartialEq, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../frontend/src/types/generated/class-user.ts")]
pub enum ClassUserRole {
    Student,             // 学生
    ClassRepresentative, // 课代表
    Teacher,             // 教师
}

impl ClassUserRole {
    pub const STUDENT: &'static str = "student";
    pub const TEACHER: &'static str = "teacher";
    pub const CLASSREPRESENTATIVE: &'static str = "class_representative";

    pub fn class_teacher_roles() -> &'static [&'static ClassUserRole] {
        &[&Self::Teacher]
    }
    pub fn class_representative_roles() -> &'static [&'static ClassUserRole] {
        &[&Self::ClassRepresentative, &Self::Teacher]
    }
    pub fn all_roles() -> &'static [&'static ClassUserRole] {
        &[&Self::Student, &Self::ClassRepresentative, &Self::Teacher]
    }
}

impl<'de> Deserialize<'de> for ClassUserRole {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "student" => Ok(ClassUserRole::Student),
            "class_representative" => Ok(ClassUserRole::ClassRepresentative),
            "teacher" => Ok(ClassUserRole::Teacher),
            _ => Err(serde::de::Error::custom(format!(
                "无效的班级用户角色: '{s}'. 支持的角色: student, class_representative, teacher"
            ))),
        }
    }
}

impl std::fmt::Display for ClassUserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClassUserRole::Student => write!(f, "student"),
            ClassUserRole::ClassRepresentative => write!(f, "class_representative"),
            ClassUserRole::Teacher => write!(f, "teacher"),
        }
    }
}

impl std::str::FromStr for ClassUserRole {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "student" => Ok(ClassUserRole::Student),
            "class_representative" => Ok(ClassUserRole::ClassRepresentative),
            "teacher" => Ok(ClassUserRole::Teacher),
            _ => Err(format!("Invalid class user role: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/class-user.ts")]
pub struct ClassUser {
    #[serde(with = "serialization::i64_as_string")]
    #[ts(type = "string")]
    pub id: i64,
    #[serde(with = "serialization::i64_as_string")]
    #[ts(type = "string")]
    pub class_id: i64,
    #[serde(with = "serialization::i64_as_string")]
    #[ts(type = "string")]
    pub user_id: i64,
    pub role: ClassUserRole,
    pub joined_at: chrono::DateTime<chrono::Utc>,
}
