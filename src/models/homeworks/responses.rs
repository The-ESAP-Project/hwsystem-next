use crate::models::common::pagination::PaginationInfo;
use crate::models::homeworks::entities::Homework;
use serde::Serialize;
use ts_rs::TS;

#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/homework.ts")]
pub struct HomeworkCreator {
    pub id: i64,
    pub username: String,
    pub profile_name: Option<String>,
}

#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/homework.ts")]
pub struct HomeworkResponse {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub content: Option<String>,
    pub deadline: Option<String>,
    pub max_score: f64,
    pub allow_late_submission: bool,
    pub attachments: Vec<String>,
    pub submission_count: i32,
    pub status: String,
    pub created_by: HomeworkCreator,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/homework.ts")]
pub struct HomeworkListResponse {
    pub items: Vec<Homework>,
    pub pagination: PaginationInfo,
}
