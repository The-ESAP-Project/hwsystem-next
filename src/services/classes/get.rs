use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};

use super::ClassService;
use crate::{
    middlewares::RequireJWT,
    models::{ApiResponse, ErrorCode, classes::entities::Class, users::entities::UserRole},
};

pub async fn get_class(
    service: &ClassService,
    request: &HttpRequest,
    class_id: i64,
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

    match storage.get_class_by_id(class_id).await {
        Ok(Some(class)) => {
            // 权限校验
            if let Err(resp) = check_class_access_permission(role, uid, &class) {
                return Ok(resp);
            }
            Ok(HttpResponse::Ok().json(ApiResponse::success(
                class,
                "Class information retrieved successfully",
            )))
        }
        Ok(None) => Ok(HttpResponse::NotFound().json(ApiResponse::error_empty(
            ErrorCode::ClassNotFound,
            "Class not found",
        ))),
        Err(e) => Ok(
            HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                ErrorCode::InternalServerError,
                format!("Failed to get class information: {e}"),
            )),
        ),
    }
}

/// 权限校验辅助函数
fn check_class_access_permission(
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
                    "You do not have permission to view another teacher's class",
                )));
            }
            Ok(())
        }
        _ => Err(HttpResponse::Forbidden().json(ApiResponse::error_empty(
            ErrorCode::ClassPermissionDenied,
            "You do not have permission to view this class",
        ))),
    }
}

pub async fn get_class_by_code(
    service: &ClassService,
    request: &HttpRequest,
    code: String,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request);

    match storage.get_class_by_code(&code).await {
        Ok(Some(class)) => Ok(HttpResponse::Ok().json(ApiResponse::success(
            class,
            "Class information retrieved successfully",
        ))),
        Ok(None) => Ok(HttpResponse::NotFound().json(ApiResponse::error_empty(
            ErrorCode::ClassNotFound,
            "Class not found",
        ))),
        Err(e) => Ok(
            HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                ErrorCode::InternalServerError,
                format!("Failed to get class information: {e}"),
            )),
        ),
    }
}
