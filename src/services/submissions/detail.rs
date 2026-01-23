use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};

use super::SubmissionService;
use crate::models::{ApiResponse, ErrorCode};

pub async fn get_submission(
    service: &SubmissionService,
    request: &HttpRequest,
    submission_id: i64,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request);

    match storage.get_submission_by_id(submission_id).await {
        Ok(Some(submission)) => {
            Ok(HttpResponse::Ok().json(ApiResponse::success(submission, "查询成功")))
        }
        Ok(None) => Ok(HttpResponse::NotFound()
            .json(ApiResponse::error_empty(ErrorCode::NotFound, "提交不存在"))),
        Err(e) => Ok(
            HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                ErrorCode::InternalServerError,
                format!("查询提交失败: {e}"),
            )),
        ),
    }
}

pub async fn get_latest_submission(
    service: &SubmissionService,
    request: &HttpRequest,
    homework_id: i64,
    creator_id: i64,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request);

    match storage.get_latest_submission(homework_id, creator_id).await {
        Ok(Some(submission)) => {
            Ok(HttpResponse::Ok().json(ApiResponse::success(submission, "查询成功")))
        }
        Ok(None) => Ok(HttpResponse::NotFound()
            .json(ApiResponse::error_empty(ErrorCode::NotFound, "暂无提交"))),
        Err(e) => Ok(
            HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                ErrorCode::InternalServerError,
                format!("查询提交失败: {e}"),
            )),
        ),
    }
}
