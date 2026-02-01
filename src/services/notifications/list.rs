use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};

use super::NotificationService;
use crate::models::ApiResponse;
use crate::models::notifications::requests::NotificationListQuery;
use crate::services::{StorageProvider, error_response};

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
        Err(e) => Ok(error_response(e)),
    }
}
