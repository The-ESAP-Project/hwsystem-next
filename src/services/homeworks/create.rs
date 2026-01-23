use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};

use super::HomeworkService;
use crate::models::homeworks::requests::CreateHomeworkRequest;
use crate::models::{ApiResponse, ErrorCode};

pub async fn create_homework(
    service: &HomeworkService,
    request: &HttpRequest,
    created_by: i64,
    req: CreateHomeworkRequest,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request);

    match storage.create_homework(created_by, req).await {
        Ok(homework) => {
            Ok(HttpResponse::Created().json(ApiResponse::success(homework, "创建成功")))
        }
        Err(e) => Ok(
            HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                ErrorCode::InternalServerError,
                format!("创建作业失败: {e}"),
            )),
        ),
    }
}
