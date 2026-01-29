pub mod create;
pub mod detail;
pub mod list;
pub mod update;

use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};
use std::sync::Arc;

use crate::models::grades::requests::{CreateGradeRequest, GradeListQuery, UpdateGradeRequest};
use crate::models::{ApiResponse, ErrorCode};
use crate::storage::Storage;

pub struct GradeService {
    storage: Option<Arc<dyn Storage>>,
}

impl GradeService {
    pub fn new_lazy() -> Self {
        Self { storage: None }
    }

    pub(crate) fn get_storage(
        &self,
        request: &HttpRequest,
    ) -> Result<Arc<dyn Storage>, actix_web::Error> {
        if let Some(storage) = &self.storage {
            Ok(storage.clone())
        } else {
            request
                .app_data::<actix_web::web::Data<Arc<dyn Storage>>>()
                .map(|data| data.get_ref().clone())
                .ok_or_else(|| {
                    actix_web::error::InternalError::from_response(
                        "Storage service unavailable",
                        HttpResponse::InternalServerError().json(ApiResponse::<()>::error_empty(
                            ErrorCode::InternalServerError,
                            "Storage service unavailable",
                        )),
                    )
                    .into()
                })
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
