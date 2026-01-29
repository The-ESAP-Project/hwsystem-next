use crate::{
    middlewares::{RequireClassRole, RequireJWT},
    models::{
        ApiResponse, ErrorCode,
        class_users::entities::{ClassUser, ClassUserRole},
        users::entities::{User, UserRole},
    },
    services::ClassUserService,
};
use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};

pub async fn get_class_user(
    service: &ClassUserService,
    req: &HttpRequest,
    class_id: i64,
    user_id: i64,
) -> ActixResult<HttpResponse> {
    let user_claims = match RequireJWT::extract_user_claims(req) {
        Some(claims) => claims,
        None => {
            return Ok(HttpResponse::Unauthorized().json(ApiResponse::error_empty(
                ErrorCode::Unauthorized,
                "Unauthorized: missing user claims",
            )));
        }
    };

    let current_class_user = RequireClassRole::extract_user_class_user(req);
    let storage = service.get_storage(req)?;

    // 通过 user_id 和 class_id 获取目标班级用户信息
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
                    format!("Internal error: {e}"),
                )),
            );
        }
    };

    // 权限校验
    if let Err(resp) = check_class_user_get_permission(
        &user_claims,
        &current_class_user,
        class_id,
        &target_class_user,
    ) {
        return Ok(resp);
    }

    Ok(HttpResponse::Ok().json(ApiResponse::success(
        Some(target_class_user),
        "Class user retrieved successfully",
    )))
}

fn check_class_user_get_permission(
    user_claims: &User,
    current_class_user: &Option<ClassUser>,
    class_id: i64,
    target_class_user: &ClassUser,
) -> Result<(), HttpResponse> {
    // 管理员直接放行
    if user_claims.role == UserRole::Admin {
        return Ok(());
    }

    match current_class_user {
        Some(current_cu) => {
            if current_cu.class_id != class_id {
                return Err(HttpResponse::Forbidden().json(ApiResponse::error_empty(
                    ErrorCode::ClassPermissionDenied,
                    "Class ID mismatch",
                )));
            }
            match current_cu.role {
                ClassUserRole::Teacher | ClassUserRole::ClassRepresentative => Ok(()),
                ClassUserRole::Student => {
                    // 学生只能查看自己的信息
                    if current_cu.user_id == target_class_user.user_id {
                        Ok(())
                    } else {
                        Err(HttpResponse::Forbidden().json(ApiResponse::error_empty(
                            ErrorCode::ClassPermissionDenied,
                            "You do not have permission to access this resource",
                        )))
                    }
                }
            }
        }
        None => Err(HttpResponse::Forbidden().json(ApiResponse::error_empty(
            ErrorCode::ClassPermissionDenied,
            "You do not have permission to access this resource",
        ))),
    }
}
