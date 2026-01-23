//! 提交实体

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "submissions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub homework_id: i64,
    pub creator_id: i64,
    pub version: i32,
    #[sea_orm(column_type = "Text", nullable)]
    pub content: Option<String>,
    pub status: String,
    pub is_late: bool,
    pub submitted_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::homeworks::Entity",
        from = "Column::HomeworkId",
        to = "super::homeworks::Column::Id"
    )]
    Homework,
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::CreatorId",
        to = "super::users::Column::Id"
    )]
    Creator,
    #[sea_orm(has_one = "super::grades::Entity")]
    Grade,
    #[sea_orm(has_many = "super::submission_files::Entity")]
    SubmissionFiles,
}

impl Related<super::homeworks::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Homework.def()
    }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Creator.def()
    }
}

impl Related<super::grades::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Grade.def()
    }
}

impl Related<super::submission_files::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SubmissionFiles.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

// 从数据库模型转换为业务模型
impl Model {
    pub fn into_submission(self) -> crate::models::submissions::entities::Submission {
        use crate::models::submissions::entities::{Submission, SubmissionStatus};
        use chrono::{DateTime, Utc};

        Submission {
            id: self.id,
            homework_id: self.homework_id,
            creator_id: self.creator_id,
            version: self.version,
            content: self.content,
            status: self
                .status
                .parse::<SubmissionStatus>()
                .unwrap_or(SubmissionStatus::Pending),
            is_late: self.is_late,
            submitted_at: DateTime::<Utc>::from_timestamp(self.submitted_at, 0).unwrap_or_default(),
        }
    }
}
