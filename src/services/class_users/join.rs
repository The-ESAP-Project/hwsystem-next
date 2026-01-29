use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};
use tracing::error;

use super::ClassUserService;
use crate::models::notifications::entities::{NotificationType, ReferenceType};
use crate::services::StorageProvider;
use crate::services::notifications::trigger::send_notification;
use crate::{
    middlewares::RequireJWT,
    models::{
        ApiResponse, ErrorCode,
        class_users::{entities::ClassUserRole, requests::JoinClassRequest},
    },
};

pub async fn join_class(
    service: &ClassUserService,
    request: &HttpRequest,
    class_id: i64,
    join_data: JoinClassRequest,
) -> ActixResult<HttpResponse> {
    let user_id = match RequireJWT::extract_user_id(request) {
        Some(id) => id,
        None => {
            return Ok(HttpResponse::Unauthorized().json(ApiResponse::error_empty(
                ErrorCode::Unauthorized,
                "Unauthorized: missing user id",
            )));
        }
    };

    let storage = service.get_storage(request)?;
    let invite_code = &join_data.invite_code;

    let (class, class_user) = match storage
        .get_class_and_class_user_by_class_id_and_code(class_id, invite_code, user_id)
        .await
    {
        Ok(res) => res,
        Err(e) => {
            error!("Error getting class and user role by id and code: {}", e);
            return Ok(
                HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                    ErrorCode::ClassJoinFailed,
                    "Failed to get class and user role",
                )),
            );
        }
    };

    match (class, class_user) {
        (None, _) => {
            return Ok(HttpResponse::NotFound().json(ApiResponse::error_empty(
                ErrorCode::ClassInviteCodeInvalid,
                "Class not found or invite code is invalid",
            )));
        }
        (Some(c), Some(_)) => {
            return Ok(HttpResponse::Conflict().json(ApiResponse::error(
                ErrorCode::ClassAlreadyJoined,
                c,
                "User has already joined the class",
            )));
        }
        (Some(_), None) => {
            // 继续执行加入逻辑
        }
    }

    match storage
        .join_class(user_id, class_id, ClassUserRole::Student)
        .await
    {
        Ok(class_user) => {
            // 异步发送通知
            let storage_clone = storage.clone();

            tokio::spawn(async move {
                if let Ok(Some(class)) = storage_clone.get_class_by_id(class_id).await {
                    send_notification(
                        storage_clone,
                        user_id,
                        NotificationType::ClassJoined,
                        format!("成功加入班级：{}", class.name),
                        Some(format!("您已成功加入班级「{}」", class.name)),
                        Some(ReferenceType::Class),
                        Some(class_id),
                    )
                    .await;
                }
            });

            Ok(HttpResponse::Ok().json(ApiResponse::success(
                class_user,
                "Class joined successfully",
            )))
        }
        Err(e) => {
            error!("Error joining class: {}", e);
            Ok(
                HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                    ErrorCode::ClassJoinFailed,
                    "Failed to join class",
                )),
            )
        }
    }
}
