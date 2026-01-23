use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};

use super::HomeworkService;
use crate::models::{ApiResponse, ErrorCode};

pub async fn get_homework(
    service: &HomeworkService,
    request: &HttpRequest,
    homework_id: i64,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request);

    match storage.get_homework_by_id(homework_id).await {
        Ok(Some(homework)) => {
            Ok(HttpResponse::Ok().json(ApiResponse::success(homework, "查询成功")))
        }
        Ok(None) => Ok(HttpResponse::NotFound()
            .json(ApiResponse::error_empty(ErrorCode::NotFound, "作业不存在"))),
        Err(e) => Ok(
            HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                ErrorCode::InternalServerError,
                format!("查询作业失败: {e}"),
            )),
        ),
    }
}
