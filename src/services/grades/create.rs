use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};

use super::GradeService;
use crate::models::grades::requests::CreateGradeRequest;
use crate::models::{ApiResponse, ErrorCode};

pub async fn create_grade(
    service: &GradeService,
    request: &HttpRequest,
    grader_id: i64,
    req: CreateGradeRequest,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request);

    // 检查提交是否存在
    match storage.get_submission_by_id(req.submission_id).await {
        Ok(None) => {
            return Ok(HttpResponse::NotFound()
                .json(ApiResponse::error_empty(ErrorCode::NotFound, "提交不存在")));
        }
        Err(e) => {
            return Ok(
                HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                    ErrorCode::InternalServerError,
                    format!("查询提交失败: {e}"),
                )),
            );
        }
        _ => {}
    }

    // 检查是否已评分
    match storage.get_grade_by_submission_id(req.submission_id).await {
        Ok(Some(_)) => {
            return Ok(HttpResponse::Conflict().json(ApiResponse::error_empty(
                ErrorCode::Conflict,
                "该提交已评分，请使用更新接口",
            )));
        }
        Err(e) => {
            return Ok(
                HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                    ErrorCode::InternalServerError,
                    format!("查询评分失败: {e}"),
                )),
            );
        }
        _ => {}
    }

    match storage.create_grade(grader_id, req).await {
        Ok(grade) => Ok(HttpResponse::Created().json(ApiResponse::success(grade, "评分成功"))),
        Err(e) => Ok(
            HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                ErrorCode::InternalServerError,
                format!("创建评分失败: {e}"),
            )),
        ),
    }
}
