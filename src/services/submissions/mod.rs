pub mod create;
pub mod delete;
pub mod detail;
pub mod grade;
pub mod history;
pub mod list;
pub mod summary;

use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};
use std::sync::Arc;

use crate::models::submissions::requests::{CreateSubmissionRequest, SubmissionListQuery};
use crate::models::users::entities::UserRole;
use crate::storage::Storage;

pub struct SubmissionService {
    storage: Option<Arc<dyn Storage>>,
}

impl SubmissionService {
    pub fn new_lazy() -> Self {
        Self { storage: None }
    }

    /// 创建提交
    pub async fn create_submission(
        &self,
        request: &HttpRequest,
        creator_id: i64,
        creator_role: UserRole,
        req: CreateSubmissionRequest,
    ) -> ActixResult<HttpResponse> {
        create::create_submission(self, request, creator_id, creator_role, req).await
    }

    /// 获取提交详情
    pub async fn get_submission(
        &self,
        request: &HttpRequest,
        submission_id: i64,
    ) -> ActixResult<HttpResponse> {
        detail::get_submission(self, request, submission_id).await
    }

    /// 获取最新提交
    pub async fn get_latest_submission(
        &self,
        request: &HttpRequest,
        homework_id: i64,
        creator_id: i64,
    ) -> ActixResult<HttpResponse> {
        detail::get_latest_submission(self, request, homework_id, creator_id).await
    }

    /// 获取用户提交历史
    pub async fn list_user_submissions(
        &self,
        request: &HttpRequest,
        homework_id: i64,
        creator_id: i64,
    ) -> ActixResult<HttpResponse> {
        history::list_user_submissions(self, request, homework_id, creator_id).await
    }

    /// 列出提交
    pub async fn list_submissions(
        &self,
        request: &HttpRequest,
        query: SubmissionListQuery,
    ) -> ActixResult<HttpResponse> {
        list::list_submissions(self, request, query).await
    }

    /// 删除/撤回提交
    pub async fn delete_submission(
        &self,
        request: &HttpRequest,
        submission_id: i64,
        user_id: i64,
    ) -> ActixResult<HttpResponse> {
        delete::delete_submission(self, request, submission_id, user_id).await
    }

    /// 获取提交概览（按学生聚合）
    pub async fn get_submission_summary(
        &self,
        request: &HttpRequest,
        homework_id: i64,
        page: Option<i64>,
        page_size: Option<i64>,
        graded: Option<bool>,
    ) -> ActixResult<HttpResponse> {
        summary::get_submission_summary(self, request, homework_id, page, page_size, graded).await
    }

    /// 获取某学生某作业的所有版本（教师视角）
    pub async fn list_user_submissions_for_teacher(
        &self,
        request: &HttpRequest,
        homework_id: i64,
        user_id: i64,
    ) -> ActixResult<HttpResponse> {
        summary::list_user_submissions_for_teacher(self, request, homework_id, user_id).await
    }

    /// 获取提交的评分
    pub async fn get_submission_grade(
        &self,
        request: &HttpRequest,
        submission_id: i64,
    ) -> ActixResult<HttpResponse> {
        grade::get_submission_grade(self, request, submission_id).await
    }
}

use crate::services::StorageProvider;

impl StorageProvider for SubmissionService {
    fn storage_ref(&self) -> Option<Arc<dyn Storage>> {
        self.storage.clone()
    }
}
