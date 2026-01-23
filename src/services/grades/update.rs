use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};

use super::GradeService;
use crate::models::grades::requests::UpdateGradeRequest;
use crate::models::{ApiResponse, ErrorCode};

pub async fn update_grade(
    service: &GradeService,
    request: &HttpRequest,
    grade_id: i64,
    req: UpdateGradeRequest,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request);

    match storage.update_grade(grade_id, req).await {
        Ok(Some(grade)) => Ok(HttpResponse::Ok().json(ApiResponse::success(grade, "更新成功"))),
        Ok(None) => Ok(HttpResponse::NotFound()
            .json(ApiResponse::error_empty(ErrorCode::NotFound, "评分不存在"))),
        Err(e) => Ok(
            HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                ErrorCode::InternalServerError,
                format!("更新评分失败: {e}"),
            )),
        ),
    }
}
