use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};

use super::SubmissionService;
use crate::models::submissions::requests::CreateSubmissionRequest;
use crate::models::{ApiResponse, ErrorCode};

pub async fn create_submission(
    service: &SubmissionService,
    request: &HttpRequest,
    creator_id: i64,
    req: CreateSubmissionRequest,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request);

    // 检查作业是否存在
    match storage.get_homework_by_id(req.homework_id).await {
        Ok(None) => {
            return Ok(HttpResponse::NotFound()
                .json(ApiResponse::error_empty(ErrorCode::NotFound, "作业不存在")));
        }
        Err(e) => {
            return Ok(
                HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                    ErrorCode::InternalServerError,
                    format!("查询作业失败: {e}"),
                )),
            );
        }
        _ => {}
    }

    match storage.create_submission(creator_id, req).await {
        Ok(submission) => {
            Ok(HttpResponse::Created().json(ApiResponse::success(submission, "提交成功")))
        }
        Err(e) => Ok(
            HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                ErrorCode::InternalServerError,
                format!("创建提交失败: {e}"),
            )),
        ),
    }
}
