use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};

use super::ClassService;
use crate::{
    middlewares::RequireJWT,
    models::{ApiResponse, ErrorCode, classes::entities::Class, users::entities::UserRole},
};

pub async fn delete_class(
    service: &ClassService,
    request: &HttpRequest,
    class_id: i64,
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
    let class = match storage.get_class_by_id(class_id).await {
        Ok(Some(class)) => class,
        Ok(None) => {
            return Ok(HttpResponse::NotFound().json(ApiResponse::error_empty(
                ErrorCode::ClassNotFound,
                "Class not found",
            )));
        }
        Err(e) => {
            return Ok(
                HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                    ErrorCode::InternalServerError,
                    format!("Failed to get class information: {e}"),
                )),
            );
        }
    };

    // 权限校验
    if let Err(resp) = check_class_delete_permission(role, uid, &class) {
        return Ok(resp);
    }

    match storage.delete_class(class_id).await {
        Ok(true) => {
            Ok(HttpResponse::Ok().json(ApiResponse::success_empty("Class deleted successfully")))
        }
        Ok(false) => Ok(HttpResponse::NotFound().json(ApiResponse::error_empty(
            ErrorCode::ClassNotFound,
            "Class not found",
        ))),
        Err(e) => Ok(
            HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                ErrorCode::ClassDeleteFailed,
                format!("Class deletion failed: {e}"),
            )),
        ),
    }
}

/// 权限校验辅助函数
fn check_class_delete_permission(
    role: Option<UserRole>,
    uid: i64,
    class: &Class,
) -> Result<(), HttpResponse> {
    match role {
        Some(UserRole::Admin) => Ok(()),
        Some(UserRole::Teacher) => {
            if class.teacher_id != uid {
                Err(HttpResponse::Forbidden().json(ApiResponse::error_empty(
                    ErrorCode::ClassPermissionDenied,
                    "You do not have permission to delete another teacher's class",
                )))
            } else {
                Ok(())
            }
        }
        _ => Err(HttpResponse::Forbidden().json(ApiResponse::error_empty(
            ErrorCode::ClassPermissionDenied,
            "You do not have permission to delete this class",
        ))),
    }
}
