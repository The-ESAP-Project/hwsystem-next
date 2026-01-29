use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::models::common::serialization::{
    serialize_i64_as_string, serialize_option_i64_as_string,
};

/// 通知类型
#[derive(Debug, Clone, Serialize, PartialEq, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../frontend/src/types/generated/notification.ts")]
pub enum NotificationType {
    // 作业相关
    HomeworkCreated,  // 新作业发布
    HomeworkUpdated,  // 作业更新
    HomeworkDeadline, // 作业即将截止

    // 提交相关
    SubmissionReceived, // 收到新提交（通知教师）

    // 评分相关
    GradeReceived, // 收到评分（通知学生）
    GradeUpdated,  // 评分修改（通知学生）

    // 班级相关
    ClassJoined,      // 加入班级
    ClassRoleChanged, // 班级角色变更
}

impl NotificationType {
    pub const HOMEWORK_CREATED: &'static str = "homework_created";
    pub const HOMEWORK_UPDATED: &'static str = "homework_updated";
    pub const HOMEWORK_DEADLINE: &'static str = "homework_deadline";
    pub const SUBMISSION_RECEIVED: &'static str = "submission_received";
    pub const GRADE_RECEIVED: &'static str = "grade_received";
    pub const GRADE_UPDATED: &'static str = "grade_updated";
    pub const CLASS_JOINED: &'static str = "class_joined";
    pub const CLASS_ROLE_CHANGED: &'static str = "class_role_changed";
}

impl<'de> Deserialize<'de> for NotificationType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

impl std::fmt::Display for NotificationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NotificationType::HomeworkCreated => write!(f, "{}", Self::HOMEWORK_CREATED),
            NotificationType::HomeworkUpdated => write!(f, "{}", Self::HOMEWORK_UPDATED),
            NotificationType::HomeworkDeadline => write!(f, "{}", Self::HOMEWORK_DEADLINE),
            NotificationType::SubmissionReceived => write!(f, "{}", Self::SUBMISSION_RECEIVED),
            NotificationType::GradeReceived => write!(f, "{}", Self::GRADE_RECEIVED),
            NotificationType::GradeUpdated => write!(f, "{}", Self::GRADE_UPDATED),
            NotificationType::ClassJoined => write!(f, "{}", Self::CLASS_JOINED),
            NotificationType::ClassRoleChanged => write!(f, "{}", Self::CLASS_ROLE_CHANGED),
        }
    }
}

impl std::str::FromStr for NotificationType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "homework_created" => Ok(NotificationType::HomeworkCreated),
            "homework_updated" => Ok(NotificationType::HomeworkUpdated),
            "homework_deadline" => Ok(NotificationType::HomeworkDeadline),
            "submission_received" => Ok(NotificationType::SubmissionReceived),
            "grade_received" => Ok(NotificationType::GradeReceived),
            "grade_updated" => Ok(NotificationType::GradeUpdated),
            "class_joined" => Ok(NotificationType::ClassJoined),
            "class_role_changed" => Ok(NotificationType::ClassRoleChanged),
            _ => Err(format!("Invalid notification type: {s}")),
        }
    }
}

/// 引用类型
#[derive(Debug, Clone, Serialize, PartialEq, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../frontend/src/types/generated/notification.ts")]
pub enum ReferenceType {
    Homework,
    Submission,
    Grade,
    Class,
}

impl<'de> Deserialize<'de> for ReferenceType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

impl std::fmt::Display for ReferenceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReferenceType::Homework => write!(f, "homework"),
            ReferenceType::Submission => write!(f, "submission"),
            ReferenceType::Grade => write!(f, "grade"),
            ReferenceType::Class => write!(f, "class"),
        }
    }
}

impl std::str::FromStr for ReferenceType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "homework" => Ok(ReferenceType::Homework),
            "submission" => Ok(ReferenceType::Submission),
            "grade" => Ok(ReferenceType::Grade),
            "class" => Ok(ReferenceType::Class),
            _ => Err(format!("Invalid reference type: {s}")),
        }
    }
}

/// 通知实体
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/notification.ts")]
pub struct Notification {
    #[serde(serialize_with = "serialize_i64_as_string")]
    #[ts(type = "string")]
    pub id: i64,
    #[serde(serialize_with = "serialize_i64_as_string")]
    #[ts(type = "string")]
    pub user_id: i64,
    pub notification_type: NotificationType,
    pub title: String,
    pub content: Option<String>,
    pub reference_type: Option<ReferenceType>,
    #[serde(serialize_with = "serialize_option_i64_as_string")]
    #[ts(type = "string | null")]
    pub reference_id: Option<i64>,
    pub is_read: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
