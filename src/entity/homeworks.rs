//! 作业实体

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "homeworks")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub class_id: i64,
    pub title: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub description: Option<String>,
    pub max_score: f64,
    pub deadline: Option<i64>,
    pub allow_late: bool,
    pub created_by: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::classes::Entity",
        from = "Column::ClassId",
        to = "super::classes::Column::Id"
    )]
    Class,
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::CreatedBy",
        to = "super::users::Column::Id"
    )]
    Creator,
    #[sea_orm(has_many = "super::submissions::Entity")]
    Submissions,
    #[sea_orm(has_many = "super::homework_files::Entity")]
    HomeworkFiles,
}

impl Related<super::classes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Class.def()
    }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Creator.def()
    }
}

impl Related<super::submissions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Submissions.def()
    }
}

impl Related<super::homework_files::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::HomeworkFiles.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

// 从数据库模型转换为业务模型
impl Model {
    pub fn into_homework(self) -> crate::models::homeworks::entities::Homework {
        use crate::models::homeworks::entities::Homework;
        use chrono::{DateTime, Utc};

        Homework {
            id: self.id,
            class_id: self.class_id,
            title: self.title,
            description: self.description,
            max_score: self.max_score,
            deadline: self
                .deadline
                .and_then(|ts| DateTime::<Utc>::from_timestamp(ts, 0)),
            allow_late: self.allow_late,
            created_by: self.created_by,
            created_at: DateTime::<Utc>::from_timestamp(self.created_at, 0).unwrap_or_default(),
            updated_at: DateTime::<Utc>::from_timestamp(self.updated_at, 0).unwrap_or_default(),
        }
    }
}
