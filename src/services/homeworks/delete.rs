use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};

use super::HomeworkService;
use crate::models::{ApiResponse, ErrorCode};

pub async fn delete_homework(
    service: &HomeworkService,
    request: &HttpRequest,
    homework_id: i64,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request);

    match storage.delete_homework(homework_id).await {
        Ok(true) => Ok(HttpResponse::Ok().json(ApiResponse::success_empty("作业已删除"))),
        Ok(false) => Ok(HttpResponse::NotFound()
            .json(ApiResponse::error_empty(ErrorCode::NotFound, "作业不存在"))),
        Err(e) => Ok(
            HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                ErrorCode::InternalServerError,
                format!("删除作业失败: {e}"),
            )),
        ),
    }
}
