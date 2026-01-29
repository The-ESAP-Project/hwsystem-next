use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};

use super::NotificationService;
use crate::middlewares::RequireJWT;
use crate::models::{ApiResponse, ErrorCode};

pub async fn delete_notification(
    service: &NotificationService,
    request: &HttpRequest,
    notification_id: i64,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request)?;

    // 获取当前用户 ID
    let current_user_id = match RequireJWT::extract_user_id(request) {
        Some(id) => id,
        None => {
            return Ok(HttpResponse::Unauthorized().json(ApiResponse::error_empty(
                ErrorCode::Unauthorized,
                "无法获取用户信息",
            )));
        }
    };

    // 先获取通知，验证所有权
    let notification = match storage.get_notification_by_id(notification_id).await {
        Ok(Some(n)) => n,
        Ok(None) => {
            return Ok(HttpResponse::NotFound().json(ApiResponse::error_empty(
                ErrorCode::NotificationNotFound,
                "通知不存在",
            )));
        }
        Err(e) => {
            return Ok(
                HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                    ErrorCode::InternalServerError,
                    format!("查询通知失败: {e}"),
                )),
            );
        }
    };

    // 验证通知是否属于当前用户
    if notification.user_id != current_user_id {
        return Ok(HttpResponse::Forbidden().json(ApiResponse::error_empty(
            ErrorCode::PermissionDenied,
            "无权操作此通知",
        )));
    }

    match storage.delete_notification(notification_id).await {
        Ok(true) => Ok(HttpResponse::Ok().json(ApiResponse::success_empty("通知已删除"))),
        Ok(false) => Ok(HttpResponse::NotFound().json(ApiResponse::error_empty(
            ErrorCode::NotificationNotFound,
            "通知不存在",
        ))),
        Err(e) => Ok(
            HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                ErrorCode::InternalServerError,
                format!("删除通知失败: {e}"),
            )),
        ),
    }
}
