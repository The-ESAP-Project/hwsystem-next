use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};

use super::GradeService;
use crate::models::{ApiResponse, ErrorCode};

pub async fn get_grade(
    service: &GradeService,
    request: &HttpRequest,
    grade_id: i64,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request);

    match storage.get_grade_by_id(grade_id).await {
        Ok(Some(grade)) => Ok(HttpResponse::Ok().json(ApiResponse::success(grade, "查询成功"))),
        Ok(None) => Ok(HttpResponse::NotFound()
            .json(ApiResponse::error_empty(ErrorCode::NotFound, "评分不存在"))),
        Err(e) => Ok(
            HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                ErrorCode::InternalServerError,
                format!("查询评分失败: {e}"),
            )),
        ),
    }
}

pub async fn get_grade_by_submission(
    service: &GradeService,
    request: &HttpRequest,
    submission_id: i64,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request);

    match storage.get_grade_by_submission_id(submission_id).await {
        Ok(Some(grade)) => Ok(HttpResponse::Ok().json(ApiResponse::success(grade, "查询成功"))),
        Ok(None) => Ok(HttpResponse::NotFound().json(ApiResponse::error_empty(
            ErrorCode::NotFound,
            "该提交暂无评分",
        ))),
        Err(e) => Ok(
            HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                ErrorCode::InternalServerError,
                format!("查询评分失败: {e}"),
            )),
        ),
    }
}
