//! 班级实体

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "classes")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub name: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub description: Option<String>,
    pub teacher_id: i64,
    #[sea_orm(unique)]
    pub invite_code: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::TeacherId",
        to = "super::users::Column::Id"
    )]
    Teacher,
    #[sea_orm(has_many = "super::class_users::Entity")]
    ClassUsers,
    #[sea_orm(has_many = "super::homeworks::Entity")]
    Homeworks,
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Teacher.def()
    }
}

impl Related<super::class_users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ClassUsers.def()
    }
}

impl Related<super::homeworks::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Homeworks.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

// 从数据库模型转换为业务模型
impl Model {
    pub fn into_class(self) -> crate::models::classes::entities::Class {
        use crate::models::classes::entities::Class;
        use chrono::{DateTime, Utc};

        Class {
            id: self.id,
            name: self.name,
            description: self.description,
            teacher_id: self.teacher_id,
            invite_code: self.invite_code,
            created_at: DateTime::<Utc>::from_timestamp(self.created_at, 0).unwrap_or_default(),
            updated_at: DateTime::<Utc>::from_timestamp(self.updated_at, 0).unwrap_or_default(),
        }
    }
}
