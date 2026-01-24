pub mod create;
pub mod detail;
pub mod list;
pub mod update;

use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};
use std::sync::Arc;

use crate::models::grades::requests::{CreateGradeRequest, GradeListQuery, UpdateGradeRequest};
use crate::storage::Storage;

pub struct GradeService {
    storage: Option<Arc<dyn Storage>>,
}

impl GradeService {
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

    /// 创建评分
    pub async fn create_grade(
        &self,
        request: &HttpRequest,
        grader_id: i64,
        req: CreateGradeRequest,
    ) -> ActixResult<HttpResponse> {
        create::create_grade(self, request, grader_id, req).await
    }

    /// 获取评分详情
    pub async fn get_grade(
        &self,
        request: &HttpRequest,
        grade_id: i64,
    ) -> ActixResult<HttpResponse> {
        detail::get_grade(self, request, grade_id).await
    }

    /// 通过提交 ID 获取评分
    pub async fn get_grade_by_submission(
        &self,
        request: &HttpRequest,
        submission_id: i64,
    ) -> ActixResult<HttpResponse> {
        detail::get_grade_by_submission(self, request, submission_id).await
    }

    /// 更新评分
    pub async fn update_grade(
        &self,
        request: &HttpRequest,
        grade_id: i64,
        req: UpdateGradeRequest,
        user_id: i64,
    ) -> ActixResult<HttpResponse> {
        update::update_grade(self, request, grade_id, req, user_id).await
    }

    /// 列出评分
    pub async fn list_grades(
        &self,
        request: &HttpRequest,
        query: GradeListQuery,
    ) -> ActixResult<HttpResponse> {
        list::list_grades(self, request, query).await
    }
}
