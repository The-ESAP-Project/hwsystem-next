use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};

use super::NotificationService;
use crate::models::notifications::responses::MarkAllReadResponse;
use crate::models::{ApiResponse, ErrorCode};

pub async fn mark_as_read(
    service: &NotificationService,
    request: &HttpRequest,
    notification_id: i64,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request);

    match storage.mark_notification_as_read(notification_id).await {
        Ok(true) => Ok(HttpResponse::Ok().json(ApiResponse::success_empty("通知已标记为已读"))),
        Ok(false) => Ok(HttpResponse::NotFound()
            .json(ApiResponse::error_empty(ErrorCode::NotFound, "通知不存在"))),
        Err(e) => Ok(
            HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                ErrorCode::InternalServerError,
                format!("标记通知已读失败: {e}"),
            )),
        ),
    }
}

pub async fn mark_all_as_read(
    service: &NotificationService,
    request: &HttpRequest,
    user_id: i64,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request);

    match storage.mark_all_notifications_as_read(user_id).await {
        Ok(count) => Ok(HttpResponse::Ok().json(ApiResponse::success(
            MarkAllReadResponse {
                marked_count: count,
            },
            "操作成功",
        ))),
        Err(e) => Ok(
            HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                ErrorCode::InternalServerError,
                format!("标记全部通知已读失败: {e}"),
            )),
        ),
    }
}
