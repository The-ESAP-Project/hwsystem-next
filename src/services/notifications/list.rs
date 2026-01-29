use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};

use super::NotificationService;
use crate::models::notifications::requests::NotificationListQuery;
use crate::models::{ApiResponse, ErrorCode};
use crate::services::StorageProvider;

pub async fn list_notifications(
    service: &NotificationService,
    request: &HttpRequest,
    user_id: i64,
    query: NotificationListQuery,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request)?;

    match storage
        .list_notifications_with_pagination(user_id, query)
        .await
    {
        Ok(response) => Ok(HttpResponse::Ok().json(ApiResponse::success(response, "查询成功"))),
        Err(e) => Ok(
            HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                ErrorCode::InternalServerError,
                format!("查询通知列表失败: {e}"),
            )),
        ),
    }
}
