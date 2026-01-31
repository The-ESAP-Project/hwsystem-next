/*!
 * JWT 认证中间件
 *
 * 此中间件用于验证 JWT 令牌的有效性，确保只有经过身份验证的用户才能访问受保护的路由。
 *
 * ## 使用方法
 *
 * 1. 在路由上应用中间件：
 * ```rust,ignore
 * use actix_web::{web, App, HttpServer};
 * use crate::middlewares::require_jwt::RequireJWT;
 *
 * HttpServer::new(|| {
 *     App::new()
 *         .service(
 *             web::scope("/api")
 *                 .wrap(RequireJWT)  // 应用JWT验证中间件
 *                 .route("/protected", web::get().to(protected_handler))
 *         )
 * })
 * ```
 *
 * 2. 在处理程序中提取用户信息：
 * ```rust,ignore
 * use actix_web::{web, HttpRequest, HttpResponse, Result};
 * use crate::middlewares::require_jwt::RequireJWT;
 *
 * async fn protected_handler(req: HttpRequest) -> Result<HttpResponse> {
 *     // 提取用户Claims
 *     if let Some(claims) = RequireJWT::extract_user_claims(&req) {
 *         return Ok(HttpResponse::Ok().json(format!("Hello, {}! Role: {}", claims.username, claims.role)));
 *     }
 *
 *     // 或者只提取用户ID
 *     if let Some(user_id) = RequireJWT::extract_user_id(&req) {
 *         return Ok(HttpResponse::Ok().json(format!("User ID: {}", user_id)));
 *     }
 *
 *     // 检查用户角色
 *     if RequireJWT::has_role(&req, "admin") {
 *         return Ok(HttpResponse::Ok().json("Admin access granted"));
 *     }
 *
 *     // 检查用户是否具有任一角色
 *     if RequireJWT::has_any_role(&req, &["admin", "moderator"]) {
 *         return Ok(HttpResponse::Ok().json("Admin or moderator access granted"));
 *     }
 *
 *     Ok(HttpResponse::InternalServerError().finish())
 * }
 * ```
 *
 * ## 认证流程
 *
 * 1. 客户端在请求头中包含 `Authorization: Bearer <JWT_TOKEN>`
 * 2. 中间件提取并验证JWT令牌
 * 3. 如果令牌有效，将用户信息存储在请求扩展中，继续处理请求
 * 4. 如果令牌无效或缺失，返回401未授权错误
 *
 * ## 配置
 *
 * 确保在环境变量中设置了 `JWT_SECRET`，JWT服务将使用此密钥来验证令牌。
 */

use crate::cache::{CacheResult, ObjectCache};
use crate::config::AppConfig;
use crate::models::users::entities::UserRole;
use crate::models::{ApiResponse, ErrorCode, users::entities};
use crate::storage::Storage;
use actix_service::{Service, Transform};
use actix_web::{
    Error, HttpMessage, HttpResponse,
    body::EitherBody,
    dev::{ServiceRequest, ServiceResponse},
    http::StatusCode,
    http::header::CONTENT_TYPE,
};
use futures_util::future::{LocalBoxFuture, Ready, ready};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::{rc::Rc, sync::Arc};
use tracing::{debug, info};

const BEARER_PREFIX: &str = "Bearer ";
const AUTHORIZATION_HEADER: &str = "Authorization";

/// 对 token 进行 hash，用于缩短缓存 key 长度
/// 原始 token 200-500+ 字符，hash 后只有 16 字符
fn hash_token(token: &str) -> String {
    let mut hasher = DefaultHasher::new();
    token.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

#[derive(Clone)]
pub struct RequireJWT;

// 辅助函数：创建错误响应
fn create_error_response(status: StatusCode, message: &str) -> HttpResponse {
    match status {
        StatusCode::NOT_FOUND => HttpResponse::build(status)
            .insert_header((CONTENT_TYPE, "text/plain; charset=utf-8"))
            .body(message.to_string()),
        StatusCode::NO_CONTENT => HttpResponse::build(status)
            .insert_header((CONTENT_TYPE, "text/plain; charset=utf-8"))
            .finish(),
        _ => HttpResponse::build(status)
            .insert_header((CONTENT_TYPE, "application/json; charset=utf-8"))
            .json(ApiResponse::<()>::error_empty(
                ErrorCode::Unauthorized,
                message,
            )),
    }
}

// 辅助函数：提取并验证 JWT access token
async fn extract_and_validate_jwt(req: &ServiceRequest) -> Result<entities::User, String> {
    let token = req
        .headers()
        .get(AUTHORIZATION_HEADER)
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix(BEARER_PREFIX))
        .ok_or_else(|| "Missing or invalid Authorization header".to_string())?;

    // 验证 token 并保存 Claims（避免后续重复解码）
    let verified_claims =
        crate::utils::jwt::JwtUtils::verify_access_token(token).map_err(|err| {
            info!("JWT token validation failed: {}", err);
            "Invalid JWT token".to_string()
        })?;

    let cache = req
        .app_data::<actix_web::web::Data<Arc<dyn ObjectCache>>>()
        .ok_or_else(|| "Cache not found in app data".to_string())?
        .get_ref()
        .clone();

    // 使用 hash 后的 token 作为缓存 key，节省内存
    let cache_key = format!("user:jwt:{}", hash_token(token));

    // 从缓存中获取用户信息
    match cache.get_raw(&cache_key).await {
        CacheResult::Found(json) => match serde_json::from_str::<entities::User>(&json) {
            Ok(user) => return Ok(user),
            Err(_) => {
                cache.remove(&cache_key).await;
                info!("Failed to deserialize user from cache");
            }
        },
        _ => {
            debug!("User not found in cache");
        }
    };

    let storage = req
        .app_data::<actix_web::web::Data<Arc<dyn Storage>>>()
        .ok_or_else(|| "Storage not found in app data".to_string())?
        .get_ref()
        .clone();

    // 使用已验证的 claims（避免重复解码）
    let user_id = verified_claims
        .sub
        .parse::<i64>()
        .map_err(|_| "Invalid user ID in JWT".to_string())?;

    let user = storage
        .get_user_by_id(user_id)
        .await
        .map_err(|_| "Failed to retrieve user from storage".to_string())?
        .ok_or_else(|| "User not found".to_string())?;

    if user.status != entities::UserStatus::Active {
        return Err("User is not active".to_string());
    }

    // 将用户信息存入缓存
    let app_config = AppConfig::get();
    if let Ok(user_json) = serde_json::to_string(&user) {
        cache
            .insert_raw(cache_key, user_json, app_config.cache.default_ttl)
            .await;
    }

    Ok(user)
}

impl<S, B> Transform<S, ServiceRequest> for RequireJWT
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = RequireJWTMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequireJWTMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct RequireJWTMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for RequireJWTMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &self,
        ctx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let srv = self.service.clone();
        Box::pin(async move {
            // 处理 OPTIONS 请求
            if req.method() == actix_web::http::Method::OPTIONS {
                return Ok(req.into_response(
                    create_error_response(StatusCode::NO_CONTENT, "").map_into_right_body(),
                ));
            }

            // 验证 JWT token
            match extract_and_validate_jwt(&req).await {
                Ok(user) => {
                    debug!("JWT authentication successful for ID: {}", user.id);
                    // 可以在这里将用户信息添加到请求扩展中，供后续处理程序使用
                    req.extensions_mut().insert(user);
                    let res = srv.call(req).await?.map_into_left_body();
                    Ok(res)
                }
                Err(err) => {
                    info!(
                        "JWT authentication failed for request to {}: {}",
                        req.path(),
                        err
                    );
                    Ok(req.into_response(
                        create_error_response(
                            StatusCode::UNAUTHORIZED,
                            &format!("Unauthorized: {err}"),
                        )
                        .map_into_right_body(),
                    ))
                }
            }
        })
    }
}

// 辅助函数：从请求中提取用户信息
impl RequireJWT {
    /// 从请求扩展中提取用户Claims信息
    /// 此函数应该在应用了RequireJWT中间件的路由处理程序中使用
    pub fn extract_user_claims(req: &actix_web::HttpRequest) -> Option<entities::User> {
        req.extensions().get::<entities::User>().cloned()
    }

    /// 从请求扩展中提取用户ID
    /// 此函数应该在应用了RequireJWT中间件的路由处理程序中使用
    pub fn extract_user_id(req: &actix_web::HttpRequest) -> Option<i64> {
        req.extensions().get::<entities::User>().map(|user| user.id)
    }

    /// 从请求扩展中提取用户角色
    /// 此函数应该在应用了RequireJWT中间件的路由处理程序中使用
    pub fn extract_user_role(req: &actix_web::HttpRequest) -> Option<UserRole> {
        req.extensions()
            .get::<entities::User>()
            .map(|user| user.role.clone())
    }
}
