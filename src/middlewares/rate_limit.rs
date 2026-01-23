/*!
 * 速率限制中间件
 *
 * 此中间件用于限制请求频率，防止暴力破解和 DDoS 攻击。
 *
 * ## 使用方法
 *
 * ```rust
 * use actix_web::{web, App};
 * use crate::middlewares::rate_limit::RateLimit;
 *
 * App::new()
 *     .service(
 *         web::scope("/api/v1/auth")
 *             .wrap(RateLimit::new(5, 60))  // 5次/分钟
 *             .route("/login", web::post().to(login_handler))
 *     )
 * ```
 *
 * ## 限制规则
 *
 * - 默认使用客户端 IP 作为限制键
 * - 支持自定义限制键（如用户 ID）
 * - 超过限制返回 429 Too Many Requests
 */

use actix_service::{Service, Transform};
use actix_web::{
    Error, HttpMessage, HttpResponse,
    body::EitherBody,
    dev::{ServiceRequest, ServiceResponse},
    http::StatusCode,
    http::header::CONTENT_TYPE,
};
use futures_util::future::{LocalBoxFuture, Ready, ready};
use moka::future::Cache;
use once_cell::sync::Lazy;
use std::rc::Rc;
use std::time::Duration;
use tracing::warn;

use crate::models::{ApiResponse, ErrorCode};

/// 全局速率限制缓存
/// 键: IP:路由前缀，值: 请求计数
static RATE_LIMIT_CACHE: Lazy<Cache<String, u32>> = Lazy::new(|| {
    Cache::builder()
        .time_to_live(Duration::from_secs(60)) // 1分钟过期
        .max_capacity(100_000)
        .build()
});

/// 速率限制配置
#[derive(Clone)]
pub struct RateLimit {
    /// 时间窗口内允许的最大请求数
    max_requests: u32,
    /// 时间窗口（秒）
    window_secs: u64,
    /// 限制键前缀（用于区分不同端点）
    key_prefix: String,
}

impl RateLimit {
    /// 创建新的速率限制器
    ///
    /// # 参数
    /// - `max_requests`: 时间窗口内允许的最大请求数
    /// - `window_secs`: 时间窗口（秒）
    pub fn new(max_requests: u32, window_secs: u64) -> Self {
        Self {
            max_requests,
            window_secs,
            key_prefix: String::new(),
        }
    }

    /// 设置限制键前缀
    pub fn with_prefix(mut self, prefix: &str) -> Self {
        self.key_prefix = prefix.to_string();
        self
    }

    /// 登录端点限制：5次/分钟/IP
    pub fn login() -> Self {
        Self::new(5, 60).with_prefix("login")
    }

    /// 注册端点限制：3次/分钟/IP
    pub fn register() -> Self {
        Self::new(3, 60).with_prefix("register")
    }

    /// 文件上传限制：10次/分钟/用户
    pub fn file_upload() -> Self {
        Self::new(10, 60).with_prefix("upload")
    }

    /// 通用 API 限制：100次/分钟/用户
    pub fn api() -> Self {
        Self::new(100, 60).with_prefix("api")
    }
}

/// 从请求中提取客户端 IP
fn extract_client_ip(req: &ServiceRequest) -> String {
    // 优先从 X-Forwarded-For 头获取（用于反向代理场景）
    if let Some(forwarded) = req.headers().get("X-Forwarded-For")
        && let Ok(value) = forwarded.to_str()
        && let Some(ip) = value.split(',').next()
    {
        return ip.trim().to_string();
    }

    // 从 X-Real-IP 头获取
    if let Some(real_ip) = req.headers().get("X-Real-IP")
        && let Ok(ip) = real_ip.to_str()
    {
        return ip.to_string();
    }

    // 从连接信息获取
    req.connection_info()
        .realip_remote_addr()
        .unwrap_or("unknown")
        .to_string()
}

/// 从请求中提取用户 ID（如果已认证）
fn extract_user_id(req: &ServiceRequest) -> Option<i64> {
    use crate::models::users::entities::User;
    req.extensions().get::<User>().map(|user| user.id)
}

/// 创建速率限制错误响应
fn create_rate_limit_response(retry_after: u64) -> HttpResponse {
    HttpResponse::build(StatusCode::TOO_MANY_REQUESTS)
        .insert_header((CONTENT_TYPE, "application/json; charset=utf-8"))
        .insert_header(("Retry-After", retry_after.to_string()))
        .insert_header(("X-RateLimit-Remaining", "0"))
        .json(ApiResponse::<()>::error_empty(
            ErrorCode::RateLimitExceeded,
            "请求过于频繁，请稍后再试",
        ))
}

impl<S, B> Transform<S, ServiceRequest> for RateLimit
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = RateLimitMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RateLimitMiddleware {
            service: Rc::new(service),
            max_requests: self.max_requests,
            window_secs: self.window_secs,
            key_prefix: self.key_prefix.clone(),
        }))
    }
}

pub struct RateLimitMiddleware<S> {
    service: Rc<S>,
    max_requests: u32,
    window_secs: u64,
    key_prefix: String,
}

impl<S, B> Service<ServiceRequest> for RateLimitMiddleware<S>
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
        let max_requests = self.max_requests;
        let window_secs = self.window_secs;
        let key_prefix = self.key_prefix.clone();

        Box::pin(async move {
            // 构建限制键
            let identifier = extract_user_id(&req)
                .map(|id| format!("user:{}", id))
                .unwrap_or_else(|| format!("ip:{}", extract_client_ip(&req)));

            let cache_key = if key_prefix.is_empty() {
                identifier
            } else {
                format!("{}:{}", key_prefix, identifier)
            };

            // 获取当前计数
            let current_count = RATE_LIMIT_CACHE.get(&cache_key).await.unwrap_or(0);

            // 检查是否超过限制
            if current_count >= max_requests {
                warn!(
                    "Rate limit exceeded for key: {} (count: {}/{})",
                    cache_key, current_count, max_requests
                );
                return Ok(req
                    .into_response(create_rate_limit_response(window_secs).map_into_right_body()));
            }

            // 增加计数
            RATE_LIMIT_CACHE
                .insert(cache_key.clone(), current_count + 1)
                .await;

            // 添加速率限制头
            let remaining = max_requests.saturating_sub(current_count + 1);
            req.extensions_mut().insert(RateLimitInfo {
                remaining,
                limit: max_requests,
                reset: window_secs,
            });

            // 继续处理请求
            let res = srv.call(req).await?.map_into_left_body();
            Ok(res)
        })
    }
}

/// 速率限制信息（可在响应中添加）
#[derive(Clone)]
pub struct RateLimitInfo {
    pub remaining: u32,
    pub limit: u32,
    pub reset: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_presets() {
        let login = RateLimit::login();
        assert_eq!(login.max_requests, 5);
        assert_eq!(login.window_secs, 60);
        assert_eq!(login.key_prefix, "login");

        let register = RateLimit::register();
        assert_eq!(register.max_requests, 3);
        assert_eq!(register.window_secs, 60);

        let upload = RateLimit::file_upload();
        assert_eq!(upload.max_requests, 10);
    }
}
