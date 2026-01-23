use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};

use super::SubmissionService;
use crate::models::submissions::requests::SubmissionListQuery;
use crate::models::{ApiResponse, ErrorCode};

pub async fn list_submissions(
    service: &SubmissionService,
    request: &HttpRequest,
    query: SubmissionListQuery,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request);

    match storage.list_submissions_with_pagination(query).await {
        Ok(response) => Ok(HttpResponse::Ok().json(ApiResponse::success(response, "查询成功"))),
        Err(e) => Ok(
            HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                ErrorCode::InternalServerError,
                format!("查询提交列表失败: {e}"),
            )),
        ),
    }
}
