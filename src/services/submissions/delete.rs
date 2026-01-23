use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};

use super::SubmissionService;
use crate::models::{ApiResponse, ErrorCode};

pub async fn delete_submission(
    service: &SubmissionService,
    request: &HttpRequest,
    submission_id: i64,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request);

    match storage.delete_submission(submission_id).await {
        Ok(true) => Ok(HttpResponse::Ok().json(ApiResponse::success_empty("提交已撤回"))),
        Ok(false) => Ok(HttpResponse::NotFound()
            .json(ApiResponse::error_empty(ErrorCode::NotFound, "提交不存在"))),
        Err(e) => Ok(
            HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                ErrorCode::InternalServerError,
                format!("撤回提交失败: {e}"),
            )),
        ),
    }
}
