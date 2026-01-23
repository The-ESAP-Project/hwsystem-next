//! 用户实体

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(unique)]
    pub username: String,
    #[sea_orm(unique)]
    pub email: String,
    pub password_hash: String,
    pub display_name: Option<String>,
    pub role: String,
    pub status: String,
    pub avatar_url: Option<String>,
    pub last_login: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::classes::Entity")]
    Classes,
    #[sea_orm(has_many = "super::class_users::Entity")]
    ClassUsers,
    #[sea_orm(has_many = "super::homeworks::Entity")]
    Homeworks,
    #[sea_orm(has_many = "super::submissions::Entity")]
    Submissions,
    #[sea_orm(has_many = "super::grades::Entity")]
    Grades,
    #[sea_orm(has_many = "super::files::Entity")]
    Files,
    #[sea_orm(has_many = "super::notifications::Entity")]
    Notifications,
}

impl Related<super::classes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Classes.def()
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

impl Related<super::submissions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Submissions.def()
    }
}

impl Related<super::grades::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Grades.def()
    }
}

impl Related<super::files::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Files.def()
    }
}

impl Related<super::notifications::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Notifications.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

// 从数据库模型转换为业务模型
impl Model {
    pub fn into_user(self) -> crate::models::users::entities::User {
        use crate::models::users::entities::{User, UserRole, UserStatus};
        use chrono::{DateTime, Utc};

        User {
            id: self.id,
            username: self.username,
            email: self.email,
            password_hash: self.password_hash,
            role: self.role.parse::<UserRole>().unwrap_or(UserRole::User),
            status: self
                .status
                .parse::<UserStatus>()
                .unwrap_or(UserStatus::Active),
            display_name: self.display_name,
            avatar_url: self.avatar_url,
            last_login: self
                .last_login
                .and_then(|ts| DateTime::<Utc>::from_timestamp(ts, 0)),
            created_at: DateTime::<Utc>::from_timestamp(self.created_at, 0).unwrap_or_default(),
            updated_at: DateTime::<Utc>::from_timestamp(self.updated_at, 0).unwrap_or_default(),
        }
    }
}
