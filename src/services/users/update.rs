use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};

use super::UserService;
use crate::middlewares::RequireJWT;
use crate::models::{
    ApiResponse, ErrorCode,
    users::{entities::UserRole, requests::UpdateUserRequest, responses::UserResponse},
};
use crate::services::StorageProvider;
use crate::utils::validate::validate_password_simple;

pub async fn update_user(
    service: &UserService,
    user_id: i64,
    mut update_data: UpdateUserRequest,
    request: &HttpRequest,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request)?;

    // 获取当前操作者信息
    let current_user = match RequireJWT::extract_user_claims(request) {
        Some(user) => user,
        None => {
            return Ok(HttpResponse::Unauthorized()
                .json(ApiResponse::error_empty(ErrorCode::Unauthorized, "未登录")));
        }
    };

    // 获取目标用户信息
    let target_user = match storage.get_user_by_id(user_id).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Ok(HttpResponse::NotFound().json(ApiResponse::error_empty(
                ErrorCode::UserNotFound,
                "用户不存在",
            )));
        }
        Err(e) => {
            return Ok(
                HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                    ErrorCode::InternalServerError,
                    format!("查询用户失败: {e}"),
                )),
            );
        }
    };

    // 权限验证：禁止修改管理员用户（除非是自己）
    if target_user.role == UserRole::Admin && user_id != current_user.id {
        return Ok(HttpResponse::Forbidden().json(ApiResponse::error_empty(
            ErrorCode::PermissionDenied,
            "无法修改其他管理员用户",
        )));
    }

    // 禁止修改用户角色为管理员（防止权限提升）
    if let Some(ref role) = update_data.role
        && *role == UserRole::Admin
        && current_user.role != UserRole::Admin
    {
        return Ok(HttpResponse::Forbidden().json(ApiResponse::error_empty(
            ErrorCode::PermissionDenied,
            "无权将用户提升为管理员",
        )));
    }

    if let Some(ref password) = update_data.password {
        // 验证密码策略
        if let Err(msg) = validate_password_simple(password) {
            return Ok(HttpResponse::BadRequest().json(ApiResponse::error_empty(
                ErrorCode::UserPasswordInvalid,
                msg,
            )));
        }

        match crate::utils::password::hash_password(password) {
            Ok(hash) => update_data.password = Some(hash),
            Err(e) => {
                return Ok(
                    HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                        ErrorCode::InternalServerError,
                        format!("密码哈希失败: {e}"),
                    )),
                );
            }
        }
    }

    match storage.update_user(user_id, update_data).await {
        Ok(Some(user)) => Ok(HttpResponse::Ok().json(ApiResponse::success(
            UserResponse { user },
            "用户信息更新成功",
        ))),
        Ok(None) => Ok(HttpResponse::NotFound().json(ApiResponse::error_empty(
            ErrorCode::UserNotFound,
            "用户不存在",
        ))),
        Err(e) => Ok(HttpResponse::BadRequest().json(ApiResponse::error_empty(
            ErrorCode::UserUpdateFailed,
            format!("更新用户信息失败: {e}"),
        ))),
    }
}
