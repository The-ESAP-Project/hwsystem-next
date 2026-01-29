use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};
use tracing::error;

use super::ClassService;
use crate::{
    middlewares::RequireJWT,
    models::{
        ApiResponse, ErrorCode,
        classes::{entities::Class, requests::UpdateClassRequest},
        users::entities::UserRole,
    },
};

pub async fn update_class(
    service: &ClassService,
    request: &HttpRequest,
    class_id: i64,
    update_data: UpdateClassRequest,
) -> ActixResult<HttpResponse> {
    let role = RequireJWT::extract_user_role(request);
    let storage = service.get_storage(request)?;

    let uid = match RequireJWT::extract_user_id(request) {
        Some(id) => id,
        None => {
            return Ok(HttpResponse::Unauthorized().json(ApiResponse::error_empty(
                ErrorCode::Unauthorized,
                "Unauthorized: missing user id",
            )));
        }
    };

    // 查询班级信息
    let class_opt = match storage.get_class_by_id(class_id).await {
        Ok(class) => class,
        Err(e) => {
            return Ok(
                HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                    ErrorCode::InternalServerError,
                    format!("Failed to get class information: {e}"),
                )),
            );
        }
    };

    let class = match class_opt {
        Some(class) => class,
        None => {
            return Ok(HttpResponse::NotFound().json(ApiResponse::error_empty(
                ErrorCode::ClassNotFound,
                "Class not found",
            )));
        }
    };

    // 权限校验
    if let Err(resp) = check_class_update_permission(role, uid, &class) {
        return Ok(resp);
    }

    match storage.update_class(class_id, update_data).await {
        Ok(Some(class)) => Ok(HttpResponse::Ok().json(ApiResponse::success(
            class,
            "Class information updated successfully",
        ))),
        Ok(None) => Ok(HttpResponse::NotFound().json(ApiResponse::error_empty(
            ErrorCode::ClassNotFound,
            "Class not found",
        ))),
        Err(e) => Ok(handle_class_create_error(&e.to_string())),
    }
}

/// 权限校验辅助函数
fn check_class_update_permission(
    role: Option<UserRole>,
    uid: i64,
    class: &Class,
) -> Result<(), HttpResponse> {
    match role {
        Some(UserRole::Admin) => Ok(()),
        Some(UserRole::Teacher) => {
            if class.teacher_id != uid {
                return Err(HttpResponse::Forbidden().json(ApiResponse::error_empty(
                    ErrorCode::ClassPermissionDenied,
                    "You do not have permission to update another teacher's class",
                )));
            }
            Ok(())
        }
        _ => Err(HttpResponse::Forbidden().json(ApiResponse::error_empty(
            ErrorCode::ClassPermissionDenied,
            "You do not have permission to update this class",
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
