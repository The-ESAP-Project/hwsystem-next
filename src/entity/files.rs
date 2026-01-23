//! 文件实体

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "files")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub user_id: Option<i64>,
    pub original_name: String,
    pub stored_name: String,
    pub file_type: String,
    pub file_size: i64,
    pub file_path: String,
    #[sea_orm(unique)]
    pub download_token: String,
    pub citation_count: i32,
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
    #[sea_orm(has_many = "super::homework_files::Entity")]
    HomeworkFiles,
    #[sea_orm(has_many = "super::submission_files::Entity")]
    SubmissionFiles,
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::homework_files::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::HomeworkFiles.def()
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
    pub fn into_file(self) -> crate::models::files::entities::File {
        use crate::models::files::entities::File;
        use chrono::{DateTime, Utc};

        File {
            id: self.id,
            user_id: self.user_id,
            original_name: self.original_name,
            stored_name: self.stored_name,
            file_type: self.file_type,
            file_size: self.file_size,
            file_path: self.file_path,
            download_token: self.download_token,
            citation_count: self.citation_count,
            created_at: DateTime::<Utc>::from_timestamp(self.created_at, 0).unwrap_or_default(),
        }
    }
}
