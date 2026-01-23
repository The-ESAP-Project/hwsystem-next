use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};
use argon2::Argon2;
use argon2::password_hash::{PasswordHasher, SaltString, rand_core::OsRng};

use crate::models::{ApiResponse, ErrorCode, users::requests::CreateUserRequest};
use crate::utils::validate::{validate_email, validate_password, validate_username};

use super::AuthService;

pub async fn handle_register(
    service: &AuthService,
    mut create_request: CreateUserRequest,
    request: &HttpRequest,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request);

    // 1. 检查用户名是否已存在
    if let Err(response) = check_username_exists(&storage, &create_request.username).await {
        return Ok(response);
    }

    // 2. 检查邮箱是否已存在
    if let Err(response) = check_email_exists(&storage, &create_request.email).await {
        return Ok(response);
    }

    // 验证用户名合法性
    if let Err(msg) = validate_username(&create_request.username) {
        return Ok(HttpResponse::BadRequest()
            .json(ApiResponse::error_empty(ErrorCode::UserNameInvalid, msg)));
    }

    // 验证邮箱
    if let Err(msg) = validate_email(&create_request.email) {
        return Ok(HttpResponse::BadRequest()
            .json(ApiResponse::error_empty(ErrorCode::UserEmailInvalid, msg)));
    }

    // 验证密码策略
    let password_result = validate_password(&create_request.password);
    if !password_result.is_valid {
        return Ok(HttpResponse::BadRequest().json(ApiResponse::error_empty(
            ErrorCode::PasswordPolicyViolation,
            password_result.error_message(),
        )));
    }

    // 3. 哈希密码合法性
    match hash_password(&create_request.password) {
        Ok(password_hash) => {
            // 将明文密码替换为哈希后的密码
            create_request.password = password_hash;

            // 4. 创建用户
            match storage.create_user(create_request).await {
                Ok(user) => {
                    Ok(HttpResponse::Created().json(ApiResponse::success(user, "注册成功")))
                }
                Err(e) => Ok(
                    HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                        ErrorCode::RegisterFailed,
                        format!("注册失败: {e}"),
                    )),
                ),
            }
        }
        Err(e) => Ok(
            HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                ErrorCode::RegisterFailed,
                format!("密码哈希失败: {e}"),
            )),
        ),
    }
}

async fn check_username_exists(
    storage: &std::sync::Arc<dyn crate::storage::Storage>,
    username: &str,
) -> Result<(), HttpResponse> {
    match storage.get_user_by_username(username).await {
        Ok(Some(_)) => Err(HttpResponse::Conflict().json(ApiResponse::error_empty(
            ErrorCode::UserNameAlreadyExists,
            "Username already exists",
        ))),
        Ok(None) => Ok(()),
        Err(e) => Err(
            HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                ErrorCode::RegisterFailed,
                format!("Register failed: {e}"),
            )),
        ),
    }
}

async fn check_email_exists(
    storage: &std::sync::Arc<dyn crate::storage::Storage>,
    email: &str,
) -> Result<(), HttpResponse> {
    match storage.get_user_by_email(email).await {
        Ok(Some(_)) => Err(HttpResponse::Conflict().json(ApiResponse::error_empty(
            ErrorCode::UserEmailAlreadyExists,
            "Email already exists",
        ))),
        Ok(None) => Ok(()),
        Err(e) => Err(
            HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                ErrorCode::RegisterFailed,
                format!("Register failed: {e}"),
            )),
        ),
    }
}

// 生成密码哈希
fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
}
