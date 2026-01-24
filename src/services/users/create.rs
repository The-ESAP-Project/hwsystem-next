use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};
use tracing::error;

use super::UserService;
use crate::models::{
    ApiResponse, ErrorCode,
    users::{requests::CreateUserRequest, responses::UserResponse},
};
use crate::utils::password::hash_password;
use crate::utils::validate::{validate_email, validate_password_simple, validate_username};

pub async fn create_user(
    service: &UserService,
    mut user_data: CreateUserRequest,
    request: &HttpRequest,
) -> ActixResult<HttpResponse> {
    // 验证用户名
    if let Err(msg) = validate_username(&user_data.username) {
        return Ok(HttpResponse::BadRequest()
            .json(ApiResponse::error_empty(ErrorCode::UserNameInvalid, msg)));
    }

    // 验证邮箱
    if let Err(msg) = validate_email(&user_data.email) {
        return Ok(HttpResponse::BadRequest()
            .json(ApiResponse::error_empty(ErrorCode::UserEmailInvalid, msg)));
    }

    // 验证密码策略
    if let Err(msg) = validate_password_simple(&user_data.password) {
        return Ok(HttpResponse::BadRequest().json(ApiResponse::error_empty(
            ErrorCode::UserPasswordInvalid,
            msg,
        )));
    }

    user_data.password = match hash_password(&user_data.password) {
        Ok(hash) => hash,
        Err(e) => {
            return Ok(
                HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                    ErrorCode::InternalServerError,
                    format!("Password hashing failed: {e}"),
                )),
            );
        }
    };

    let storage = service.get_storage(request);

    match storage.create_user(user_data).await {
        Ok(user) => Ok(HttpResponse::Created()
            .json(ApiResponse::success(UserResponse { user }, "用户创建成功"))),
        Err(e) => {
            let msg = format!("User creation failed: {e}");
            error!("{}", msg);
            // 判断是否唯一约束冲突
            if msg.contains("UNIQUE constraint failed") {
                Ok(HttpResponse::Conflict().json(ApiResponse::error_empty(
                    ErrorCode::UserAlreadyExists,
                    "Username or email already exists",
                )))
            } else {
                Ok(HttpResponse::InternalServerError()
                    .json(ApiResponse::error_empty(ErrorCode::UserCreationFailed, msg)))
            }
        }
    }
}
