use std::sync::Arc;

use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};
use tracing::{error, info};

use super::ClassService;
use crate::middlewares::RequireJWT;
use crate::models::class_users::entities::ClassUserRole;
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

    // 权限校验并确定最终的 teacher_id
    let teacher_id = match check_class_create_permission(role, uid, &class_data, &storage).await {
        Ok(tid) => tid,
        Err(resp) => return Ok(resp),
    };

    // 创建班级（使用确定后的 teacher_id）
    let mut class_data = class_data;
    class_data.teacher_id = Some(teacher_id);

    match storage.create_class(class_data).await {
        Ok(class) => {
            // 将创建者（教师）加入 class_users 表
            if let Err(e) = storage
                .join_class(teacher_id, class.id, ClassUserRole::Teacher)
                .await
            {
                error!(
                    "Failed to add teacher {} to class_users for class {}: {}",
                    teacher_id, class.id, e
                );
            }

            info!("Class {} created successfully by {}", class.name, uid);
            Ok(HttpResponse::Created()
                .json(ApiResponse::success(class, "Class created successfully")))
        }
        Err(e) => Ok(handle_class_create_error(&e.to_string())),
    }
}

/// 权限校验辅助函数
/// 返回值：成功时返回最终确定的 teacher_id
async fn check_class_create_permission(
    role: Option<UserRole>,
    uid: i64,
    class_data: &CreateClassRequest,
    storage: &Arc<dyn Storage>,
) -> Result<i64, HttpResponse> {
    match role {
        Some(UserRole::Admin) => {
            // 管理员必须指定 teacher_id
            let teacher_id = match class_data.teacher_id {
                Some(id) => id,
                None => {
                    return Err(HttpResponse::BadRequest().json(ApiResponse::error_empty(
                        ErrorCode::BadRequest,
                        "Admin must specify teacher_id",
                    )));
                }
            };

            match storage.get_user_by_id(teacher_id).await {
                Ok(Some(user)) => {
                    if user.role != UserRole::Teacher {
                        return Err(HttpResponse::Forbidden().json(ApiResponse::error_empty(
                            ErrorCode::ClassPermissionDenied,
                            "Admin can only create classes for teachers",
                        )));
                    }
                    Ok(teacher_id)
                }
                Ok(None) => Err(HttpResponse::NotFound().json(ApiResponse::error_empty(
                    ErrorCode::UserNotFound,
                    "Teacher not found",
                ))),
                Err(e) => {
                    error!("Failed to get user by id: {}", e);
                    Err(
                        HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                            ErrorCode::InternalServerError,
                            "Internal server error while fetching user",
                        )),
                    )
                }
            }
        }
        Some(UserRole::Teacher) => {
            // 教师：如果未指定 teacher_id，自动使用当前用户 ID
            let teacher_id = class_data.teacher_id.unwrap_or(uid);

            if teacher_id != uid {
                return Err(HttpResponse::Forbidden().json(ApiResponse::error_empty(
                    ErrorCode::ClassPermissionDenied,
                    "You do not have permission to create a class for another teacher",
                )));
            }
            Ok(teacher_id)
        }
        _ => Err(HttpResponse::Forbidden().json(ApiResponse::error_empty(
            ErrorCode::ClassPermissionDenied,
            "You do not have permission to create a class",
        ))),
    }
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
