//! 通知实体

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "notifications")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub user_id: i64,
    #[sea_orm(column_name = "type")]
    pub notification_type: String,
    pub title: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub content: Option<String>,
    pub reference_type: Option<String>,
    pub reference_id: Option<i64>,
    pub is_read: bool,
    pub created_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::Id"
    )]
    User,
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

// 从数据库模型转换为业务模型
impl Model {
    pub fn into_notification(self) -> crate::models::notifications::entities::Notification {
        use crate::models::notifications::entities::{
            Notification, NotificationType, ReferenceType,
        };
        use chrono::{DateTime, Utc};

        Notification {
            id: self.id,
            user_id: self.user_id,
            notification_type: self
                .notification_type
                .parse::<NotificationType>()
                .unwrap_or(NotificationType::HomeworkCreated),
            title: self.title,
            content: self.content,
            reference_type: self
                .reference_type
                .and_then(|s| s.parse::<ReferenceType>().ok()),
            reference_id: self.reference_id,
            is_read: self.is_read,
            created_at: DateTime::<Utc>::from_timestamp(self.created_at, 0).unwrap_or_default(),
        }
    }
}
