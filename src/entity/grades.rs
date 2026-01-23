//! 评分实体

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "grades")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(unique)]
    pub submission_id: i64,
    pub grader_id: i64,
    pub score: f64,
    #[sea_orm(column_type = "Text", nullable)]
    pub comment: Option<String>,
    pub graded_at: i64,
    pub updated_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::submissions::Entity",
        from = "Column::SubmissionId",
        to = "super::submissions::Column::Id"
    )]
    Submission,
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::GraderId",
        to = "super::users::Column::Id"
    )]
    Grader,
}

impl Related<super::submissions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Submission.def()
    }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Grader.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

// 从数据库模型转换为业务模型
impl Model {
    pub fn into_grade(self) -> crate::models::grades::entities::Grade {
        use crate::models::grades::entities::Grade;
        use chrono::{DateTime, Utc};

        Grade {
            id: self.id,
            submission_id: self.submission_id,
            grader_id: self.grader_id,
            score: self.score,
            comment: self.comment,
            graded_at: DateTime::<Utc>::from_timestamp(self.graded_at, 0).unwrap_or_default(),
            updated_at: DateTime::<Utc>::from_timestamp(self.updated_at, 0).unwrap_or_default(),
        }
    }
}
