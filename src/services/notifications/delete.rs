use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};

use super::NotificationService;
use crate::models::{ApiResponse, ErrorCode};

pub async fn delete_notification(
    service: &NotificationService,
    request: &HttpRequest,
    notification_id: i64,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request);

    match storage.delete_notification(notification_id).await {
        Ok(true) => Ok(HttpResponse::Ok().json(ApiResponse::success_empty("通知已删除"))),
        Ok(false) => Ok(HttpResponse::NotFound()
            .json(ApiResponse::error_empty(ErrorCode::NotFound, "通知不存在"))),
        Err(e) => Ok(
            HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                ErrorCode::InternalServerError,
                format!("删除通知失败: {e}"),
            )),
        ),
    }
}
