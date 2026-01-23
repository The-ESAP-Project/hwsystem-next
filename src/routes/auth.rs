use actix_web::{HttpRequest, HttpResponse, Result as ActixResult, web};
use once_cell::sync::Lazy;

use crate::middlewares::{self, RateLimit};
use crate::models::auth::requests::LoginRequest;
use crate::models::users::requests::CreateUserRequest;
use crate::services::AuthService;

// 懒加载的全局 AuthService 实例
static AUTH_SERVICE: Lazy<AuthService> = Lazy::new(AuthService::new_lazy);

pub async fn login(
    req: HttpRequest,
    user_data: web::Json<LoginRequest>,
) -> ActixResult<HttpResponse> {
    AUTH_SERVICE.login(user_data.into_inner(), &req).await
}

pub async fn refresh_token(request: HttpRequest) -> ActixResult<HttpResponse> {
    AUTH_SERVICE.refresh_token(&request).await
}

pub async fn register(
    req: HttpRequest,
    user_data: web::Json<CreateUserRequest>,
) -> ActixResult<HttpResponse> {
    AUTH_SERVICE.register(user_data.into_inner(), &req).await
}

pub async fn verify_token(request: HttpRequest) -> ActixResult<HttpResponse> {
    AUTH_SERVICE.verify_token(&request).await
}

pub async fn get_user(request: HttpRequest) -> ActixResult<HttpResponse> {
    AUTH_SERVICE.get_user(&request).await
}
// 配置路由
pub fn configure_auth_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1/auth")
            // 登录端点：5次/分钟/IP
            .service(
                web::resource("/login")
                    .wrap(RateLimit::login())
                    .route(web::post().to(login)),
            )
            // 注册端点：3次/分钟/IP
            .service(
                web::resource("/register")
                    .wrap(RateLimit::register())
                    .route(web::post().to(register)),
            )
            .route("/refresh", web::post().to(refresh_token))
            .service(
                web::scope("")
                    .wrap(middlewares::RequireJWT)
                    .route("/verify-token", web::get().to(verify_token))
                    .route("/me", web::get().to(get_user)),
            ),
    );
}
