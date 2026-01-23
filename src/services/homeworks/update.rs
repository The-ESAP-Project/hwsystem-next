use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};

use super::HomeworkService;
use crate::models::homeworks::requests::UpdateHomeworkRequest;
use crate::models::{ApiResponse, ErrorCode};

pub async fn update_homework(
    service: &HomeworkService,
    request: &HttpRequest,
    homework_id: i64,
    req: UpdateHomeworkRequest,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request);

    match storage.update_homework(homework_id, req).await {
        Ok(Some(homework)) => {
            Ok(HttpResponse::Ok().json(ApiResponse::success(homework, "更新成功")))
        }
        Ok(None) => Ok(HttpResponse::NotFound()
            .json(ApiResponse::error_empty(ErrorCode::NotFound, "作业不存在"))),
        Err(e) => Ok(
            HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                ErrorCode::InternalServerError,
                format!("更新作业失败: {e}"),
            )),
        ),
    }
}
