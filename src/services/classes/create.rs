use std::sync::Arc;

use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};
use tracing::{error, info};

use super::ClassService;
use crate::middlewares::RequireJWT;
use crate::models::classes::requests::CreateClassRequest;
use crate::models::users::entities::UserRole;
use crate::models::{ApiResponse, ErrorCode};
use crate::storage::Storage;

pub async fn create_class(
    service: &ClassService,
    request: &HttpRequest,
    class_data: CreateClassRequest,
) -> ActixResult<HttpResponse> {
    let role = RequireJWT::extract_user_role(request);
    let storage = service.get_storage(request);

    let uid = match RequireJWT::extract_user_id(request) {
        Some(id) => id,
        None => {
            return Ok(HttpResponse::Unauthorized().json(ApiResponse::error_empty(
                ErrorCode::Unauthorized,
                "Unauthorized: missing user id",
            )));
        }
    };

    // 权限校验
    if let Err(resp) = check_class_create_permission(role, uid, &class_data, &storage).await {
        return Ok(resp);
    }

    // 创建班级
    match storage.create_class(class_data).await {
        Ok(class) => {
            info!("Class {} created successfully by {}", class.name, uid);
            Ok(HttpResponse::Created()
                .json(ApiResponse::success(class, "Class created successfully")))
        }
        Err(e) => Ok(handle_class_create_error(&e.to_string())),
    }
}

/// 权限校验辅助函数
async fn check_class_create_permission(
    role: Option<UserRole>,
    uid: i64,
    class_data: &CreateClassRequest,
    storage: &Arc<dyn Storage>,
) -> Result<(), HttpResponse> {
    match role {
        Some(UserRole::Admin) => match storage.get_user_by_id(class_data.teacher_id).await {
            Ok(Some(user)) => {
                if user.role != UserRole::Teacher {
                    return Err(HttpResponse::BadRequest().json(ApiResponse::error_empty(
                        ErrorCode::ClassPermissionDenied,
                        "Admin can only create classes for teachers",
                    )));
                }
            }
            Ok(None) => {
                return Err(HttpResponse::NotFound().json(ApiResponse::error_empty(
                    ErrorCode::UserNotFound,
                    "User not found",
                )));
            }
            Err(e) => {
                error!("Failed to get user by id: {}", e);
                return Err(
                    HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                        ErrorCode::InternalServerError,
                        "Internal server error while fetching user",
                    )),
                );
            }
        },
        Some(UserRole::Teacher) => {
            if class_data.teacher_id != uid {
                return Err(HttpResponse::Forbidden().json(ApiResponse::error_empty(
                    ErrorCode::ClassPermissionDenied,
                    "You do not have permission to create a class for another teacher",
                )));
            }
        }
        _ => {
            return Err(HttpResponse::Forbidden().json(ApiResponse::error_empty(
                ErrorCode::ClassPermissionDenied,
                "You do not have permission to create a class",
            )));
        }
    }
    Ok(())
}

/// 错误响应辅助函数
fn handle_class_create_error(e: &str) -> HttpResponse {
    let msg = format!("Class creation failed: {e}");
    error!("{}", msg);
    if msg.contains("UNIQUE constraint failed") {
        HttpResponse::Conflict().json(ApiResponse::error_empty(
            ErrorCode::ClassAlreadyExists,
            "Classname already exists",
        ))
    } else if msg.contains("FOREIGN KEY constraint failed") {
        HttpResponse::BadRequest().json(ApiResponse::error_empty(
            ErrorCode::ClassCreationFailed,
            "Teacher does not exist",
        ))
    } else {
        HttpResponse::InternalServerError().json(ApiResponse::error_empty(
            ErrorCode::ClassCreationFailed,
            msg,
        ))
    }
}
