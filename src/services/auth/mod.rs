pub mod login;
pub mod logout;
pub mod profile;
pub mod register;
pub mod token;

use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};
use std::sync::Arc;

use crate::config::AppConfig;
use crate::models::{ApiResponse, ErrorCode};
use crate::storage::Storage;

pub struct AuthService {
    storage: Option<Arc<dyn Storage>>,
}

impl AuthService {
    pub fn new_lazy() -> Self {
        Self { storage: None }
    }

    pub(crate) fn get_storage(
        &self,
        request: &HttpRequest,
    ) -> Result<Arc<dyn Storage>, actix_web::Error> {
        if let Some(storage) = &self.storage {
            Ok(storage.clone())
        } else {
            request
                .app_data::<actix_web::web::Data<Arc<dyn Storage>>>()
                .map(|data| data.get_ref().clone())
                .ok_or_else(|| {
                    actix_web::error::InternalError::from_response(
                        "Storage service unavailable",
                        HttpResponse::InternalServerError().json(ApiResponse::<()>::error_empty(
                            ErrorCode::InternalServerError,
                            "Storage service unavailable",
                        )),
                    )
                    .into()
                })
        }
    }

    pub(crate) fn get_config(&self) -> &AppConfig {
        AppConfig::get()
    }

    // 登录验证
    pub async fn login(
        &self,
        login_request: crate::models::auth::LoginRequest,
        request: &HttpRequest,
    ) -> ActixResult<HttpResponse> {
        login::handle_login(self, login_request, request).await
    }

    // 用户注册
    pub async fn register(
        &self,
        create_request: crate::models::users::requests::CreateUserRequest,
        request: &HttpRequest,
    ) -> ActixResult<HttpResponse> {
        register::handle_register(self, create_request, request).await
    }

    // 刷新令牌
    pub async fn refresh_token(&self, request: &HttpRequest) -> ActixResult<HttpResponse> {
        token::handle_refresh_token(self, request).await
    }

    // 验证令牌
    pub async fn verify_token(&self, request: &HttpRequest) -> ActixResult<HttpResponse> {
        token::handle_verify_token(self, request).await
    }

    // 获取用户信息
    pub async fn get_user(&self, request: &HttpRequest) -> ActixResult<HttpResponse> {
        // 这里可以实现获取用户信息的逻辑
        // 例如从数据库或缓存中获取用户信息
        token::handle_get_user(self, request).await
    }

    // 更新用户资料
    pub async fn update_profile(
        &self,
        update_request: crate::models::auth::requests::UpdateProfileRequest,
        request: &HttpRequest,
    ) -> ActixResult<HttpResponse> {
        profile::handle_update_profile(self, update_request, request).await
    }

    // 用户登出
    pub async fn logout(&self) -> ActixResult<HttpResponse> {
        logout::handle_logout().await
    }
}
