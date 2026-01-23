use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};

use super::NotificationService;
use crate::models::notifications::responses::UnreadCountResponse;
use crate::models::{ApiResponse, ErrorCode};

pub async fn get_unread_count(
    service: &NotificationService,
    request: &HttpRequest,
    user_id: i64,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request);

    match storage.get_unread_notification_count(user_id).await {
        Ok(count) => Ok(HttpResponse::Ok().json(ApiResponse::success(
            UnreadCountResponse { count },
            "查询成功",
        ))),
        Err(e) => Ok(
            HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                ErrorCode::InternalServerError,
                format!("查询未读通知数量失败: {e}"),
            )),
        ),
    }
}
