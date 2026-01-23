use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};

use super::GradeService;
use crate::models::grades::requests::GradeListQuery;
use crate::models::{ApiResponse, ErrorCode};

pub async fn list_grades(
    service: &GradeService,
    request: &HttpRequest,
    query: GradeListQuery,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request);

    match storage.list_grades_with_pagination(query).await {
        Ok(response) => Ok(HttpResponse::Ok().json(ApiResponse::success(response, "查询成功"))),
        Err(e) => Ok(
            HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                ErrorCode::InternalServerError,
                format!("查询评分列表失败: {e}"),
            )),
        ),
    }
}
