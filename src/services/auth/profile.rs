use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};

use crate::middlewares::RequireJWT;
use crate::models::auth::requests::UpdateProfileRequest;
use crate::models::users::requests::UpdateUserRequest;
use crate::models::users::responses::UserResponse;
use crate::models::{ApiResponse, ErrorCode};
use crate::utils::password::hash_password;
use crate::utils::validate::validate_password_simple;

use super::AuthService;

pub async fn handle_update_profile(
    service: &AuthService,
    update_data: UpdateProfileRequest,
    request: &HttpRequest,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request);

    // 获取当前用户信息
    let current_user = match RequireJWT::extract_user_claims(request) {
        Some(user) => user,
        None => {
            return Ok(HttpResponse::Unauthorized()
                .json(ApiResponse::error_empty(ErrorCode::Unauthorized, "未登录")));
        }
    };

    // 验证邮箱唯一性（如果提供了新邮箱）
    if let Some(ref email) = update_data.email {
        // 检查邮箱是否已被其他用户使用
        if let Ok(Some(existing_user)) = storage.get_user_by_email(email).await
            && existing_user.id != current_user.id
        {
            return Ok(HttpResponse::BadRequest().json(ApiResponse::error_empty(
                ErrorCode::UserEmailAlreadyExists,
                "该邮箱已被使用",
            )));
        }
    }

    // 处理密码（如果提供了新密码）
    let hashed_password = if let Some(ref password) = update_data.password {
        // 验证密码策略
        if let Err(msg) = validate_password_simple(password) {
            return Ok(HttpResponse::BadRequest().json(ApiResponse::error_empty(
                ErrorCode::UserPasswordInvalid,
                msg,
            )));
        }

        match hash_password(password) {
            Ok(hash) => Some(hash),
            Err(e) => {
                return Ok(
                    HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                        ErrorCode::InternalServerError,
                        format!("密码哈希失败: {e}"),
                    )),
                );
            }
        }
    } else {
        None
    };

    // 构建更新请求（不包含 role 和 status，普通用户无权修改）
    let storage_update = UpdateUserRequest {
        email: update_data.email,
        password: hashed_password,
        role: None,
        status: None,
        display_name: update_data.display_name,
        avatar_url: update_data.avatar_url,
    };

    match storage.update_user(current_user.id, storage_update).await {
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
