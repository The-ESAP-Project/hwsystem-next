//! 提交附件关联实体

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "submission_files")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub submission_id: i64,
    #[sea_orm(primary_key, auto_increment = false)]
    pub file_id: i64,
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
        belongs_to = "super::files::Entity",
        from = "Column::FileId",
        to = "super::files::Column::Id"
    )]
    File,
}

impl Related<super::submissions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Submission.def()
    }
}

impl Related<super::files::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::File.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
