//! 班级用户关联实体

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "class_users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub class_id: i64,
    pub user_id: i64,
    pub role: String,
    pub joined_at: i64,
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
        from = "Column::UserId",
        to = "super::users::Column::Id"
    )]
    User,
}

impl Related<super::classes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Class.def()
    }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

// 从数据库模型转换为业务模型
impl Model {
    pub fn into_class_user(self) -> crate::models::class_users::entities::ClassUser {
        use crate::models::class_users::entities::{ClassUser, ClassUserRole};
        use chrono::{DateTime, Utc};

        ClassUser {
            id: self.id,
            class_id: self.class_id,
            user_id: self.user_id,
            role: self
                .role
                .parse::<ClassUserRole>()
                .unwrap_or(ClassUserRole::Student),
            joined_at: DateTime::<Utc>::from_timestamp(self.joined_at, 0).unwrap_or_default(),
        }
    }
}
