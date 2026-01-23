use actix_web::{HttpRequest, HttpResponse, Result as ActixResult, web};
use once_cell::sync::Lazy;

use crate::middlewares::{self, RequireJWT};
use crate::models::notifications::requests::NotificationListQuery;
use crate::models::{ApiResponse, ErrorCode};
use crate::services::NotificationService;

// 懒加载的全局 NotificationService 实例
static NOTIFICATION_SERVICE: Lazy<NotificationService> = Lazy::new(NotificationService::new_lazy);

// 列出通知
pub async fn list_notifications(
    req: HttpRequest,
    query: web::Query<NotificationListQuery>,
) -> ActixResult<HttpResponse> {
    let user_id = match RequireJWT::extract_user_id(&req) {
        Some(id) => id,
        None => {
            return Ok(HttpResponse::Unauthorized().json(ApiResponse::error_empty(
                ErrorCode::Unauthorized,
                "无法获取用户信息",
            )));
        }
    };

    NOTIFICATION_SERVICE
        .list_notifications(&req, user_id, query.into_inner())
        .await
}

// 获取未读数量
pub async fn get_unread_count(req: HttpRequest) -> ActixResult<HttpResponse> {
    let user_id = match RequireJWT::extract_user_id(&req) {
        Some(id) => id,
        None => {
            return Ok(HttpResponse::Unauthorized().json(ApiResponse::error_empty(
                ErrorCode::Unauthorized,
                "无法获取用户信息",
            )));
        }
    };

    NOTIFICATION_SERVICE.get_unread_count(&req, user_id).await
}

// 标记单条通知为已读
pub async fn mark_as_read(req: HttpRequest, path: web::Path<i64>) -> ActixResult<HttpResponse> {
    NOTIFICATION_SERVICE
        .mark_as_read(&req, path.into_inner())
        .await
}

// 标记所有通知为已读
pub async fn mark_all_as_read(req: HttpRequest) -> ActixResult<HttpResponse> {
    let user_id = match RequireJWT::extract_user_id(&req) {
        Some(id) => id,
        None => {
            return Ok(HttpResponse::Unauthorized().json(ApiResponse::error_empty(
                ErrorCode::Unauthorized,
                "无法获取用户信息",
            )));
        }
    };

    NOTIFICATION_SERVICE.mark_all_as_read(&req, user_id).await
}

// 删除通知
pub async fn delete_notification(
    req: HttpRequest,
    path: web::Path<i64>,
) -> ActixResult<HttpResponse> {
    NOTIFICATION_SERVICE
        .delete_notification(&req, path.into_inner())
        .await
}

// 配置路由
pub fn configure_notifications_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1/notifications")
            .wrap(middlewares::RequireJWT)
            .route("", web::get().to(list_notifications))
            .route("/unread-count", web::get().to(get_unread_count))
            .route("/read-all", web::put().to(mark_all_as_read))
            .route("/{id}/read", web::put().to(mark_as_read))
            .route("/{id}", web::delete().to(delete_notification)),
    );
}
