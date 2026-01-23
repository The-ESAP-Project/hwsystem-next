pub mod create;
pub mod delete;
pub mod detail;
pub mod history;
pub mod list;

use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};
use std::sync::Arc;

use crate::models::submissions::requests::{CreateSubmissionRequest, SubmissionListQuery};
use crate::storage::Storage;

pub struct SubmissionService {
    storage: Option<Arc<dyn Storage>>,
}

impl SubmissionService {
    pub fn new_lazy() -> Self {
        Self { storage: None }
    }

    pub(crate) fn get_storage(&self, request: &HttpRequest) -> Arc<dyn Storage> {
        if let Some(storage) = &self.storage {
            storage.clone()
        } else {
            request
                .app_data::<actix_web::web::Data<Arc<dyn Storage>>>()
                .expect("Storage not found in app data")
                .get_ref()
                .clone()
        }
    }

    /// 创建提交
    pub async fn create_submission(
        &self,
        request: &HttpRequest,
        creator_id: i64,
        req: CreateSubmissionRequest,
    ) -> ActixResult<HttpResponse> {
        create::create_submission(self, request, creator_id, req).await
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
    ) -> ActixResult<HttpResponse> {
        delete::delete_submission(self, request, submission_id).await
    }
}
