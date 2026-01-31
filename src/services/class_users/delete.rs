use crate::{
    middlewares::RequireJWT,
    middlewares::require_class_role::class_user_cache_key,
    models::{
        ApiResponse, ErrorCode, class_users::entities::ClassUser, classes::entities::Class,
        users::entities::UserRole,
    },
    services::{CacheProvider, ClassUserService, StorageProvider},
};
use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};

pub async fn delete_class_user(
    service: &ClassUserService,
    req: &HttpRequest,
    class_id: i64,
    user_id: i64,
) -> ActixResult<HttpResponse> {
    let user_role = RequireJWT::extract_user_role(req);
    let storage = service.get_storage(req)?;

    let uid = match RequireJWT::extract_user_id(req) {
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

    // 通过 user_id 和 class_id 获取要删除的班级用户信息
    let target_class_user = match storage
        .get_class_user_by_user_id_and_class_id(user_id, class_id)
        .await
    {
        Ok(Some(cu)) => cu,
        Ok(None) => {
            return Ok(HttpResponse::NotFound().json(ApiResponse::error_empty(
                ErrorCode::ClassUserNotFound,
                "Class user not found",
            )));
        }
        Err(e) => {
            return Ok(
                HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                    ErrorCode::InternalServerError,
                    format!("Failed to get class user: {e}"),
                )),
            );
        }
    };

    // 权限校验
    if let Err(resp) =
        check_class_user_delete_permission(user_role, uid, &target_class_user, &class)
    {
        return Ok(resp);
    }

    // 如果被删除者为本班教师，则禁止删除
    if class.teacher_id == target_class_user.user_id {
        return Ok(HttpResponse::Forbidden().json(ApiResponse::error_empty(
            ErrorCode::ClassPermissionDenied,
            "You cannot delete the class teacher. Please transfer or delete the class first.",
        )));
    }

    // 使用目标用户的 user_id 调用 leave_class
    match storage
        .leave_class(target_class_user.user_id, class_id)
        .await
    {
        Ok(true) => {
            // 失效缓存
            if let Some(cache) = service.get_cache(req) {
                cache
                    .remove(&class_user_cache_key(target_class_user.user_id, class_id))
                    .await;
            }

            Ok(HttpResponse::Ok().json(ApiResponse::success_empty(
                "Class user deleted successfully",
            )))
        }
        Ok(false) => Ok(HttpResponse::NotFound().json(ApiResponse::error_empty(
            ErrorCode::ClassUserNotFound,
            "Class user not found",
        ))),
        Err(e) => Ok(
            HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                ErrorCode::InternalServerError,
                format!("Failed to delete class user: {e}"),
            )),
        ),
    }
}

/// 权限校验辅助函数
fn check_class_user_delete_permission(
    role: Option<UserRole>,
    uid: i64,
    target_class_user: &ClassUser,
    class: &Class,
) -> Result<(), HttpResponse> {
    match role {
        Some(UserRole::Admin) => Ok(()),
        Some(UserRole::Teacher) => {
            if class.teacher_id != uid {
                Err(HttpResponse::Forbidden().json(ApiResponse::error_empty(
                    ErrorCode::ClassPermissionDenied,
                    "You do not have permission to delete another teacher's class user",
                )))
            } else {
                Ok(())
            }
        }
        Some(UserRole::User) => {
            // 学生只能删除自己
            if target_class_user.user_id == uid {
                Ok(())
            } else {
                Err(HttpResponse::Forbidden().json(ApiResponse::error_empty(
                    ErrorCode::ClassPermissionDenied,
                    "You do not have permission to delete this class user",
                )))
            }
        }
        _ => Err(HttpResponse::Forbidden().json(ApiResponse::error_empty(
            ErrorCode::ClassPermissionDenied,
            "You do not have permission to delete this class user",
        ))),
    }
}
