//! 作业附件关联实体

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "homework_files")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub homework_id: i64,
    #[sea_orm(primary_key, auto_increment = false)]
    pub file_id: i64,
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
        belongs_to = "super::files::Entity",
        from = "Column::FileId",
        to = "super::files::Column::Id"
    )]
    File,
}

impl Related<super::homeworks::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Homework.def()
    }
}

impl Related<super::files::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::File.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
