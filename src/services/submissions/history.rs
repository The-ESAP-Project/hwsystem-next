use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};

use super::SubmissionService;
use crate::models::{ApiResponse, ErrorCode};

pub async fn list_user_submissions(
    service: &SubmissionService,
    request: &HttpRequest,
    homework_id: i64,
    creator_id: i64,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request);

    match storage.list_user_submissions(homework_id, creator_id).await {
        Ok(submissions) => {
            Ok(HttpResponse::Ok().json(ApiResponse::success(submissions, "查询成功")))
        }
        Err(e) => Ok(
            HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                ErrorCode::InternalServerError,
                format!("查询提交历史失败: {e}"),
            )),
        ),
    }
}
