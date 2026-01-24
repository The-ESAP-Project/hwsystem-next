use actix_web::{HttpRequest, HttpResponse, Result as ActixResult, web};

use crate::cache::{CacheResult, ObjectCache};
use crate::middlewares::{self, RateLimit};
use crate::models::system::responses::WebSocketStatusResponse;
use crate::models::users::entities::User;
use crate::models::{ApiResponse, ErrorCode};
use crate::services::websocket::WebSocketService;
use crate::storage::Storage;
use std::sync::Arc;

/// WebSocket 连接处理
pub async fn ws_handler(
    req: HttpRequest,
    query: web::Query<WsQuery>,
    body: web::Payload,
) -> ActixResult<HttpResponse> {
    // 从 query 参数获取 token
    let token = &query.token;

    // 验证 token
    let user = match validate_token_and_get_user(&req, token).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };

    // 升级到 WebSocket
    let (response, session, stream) = actix_ws::handle(&req, body)?;

    // 在后台任务中处理 WebSocket 连接
    actix_web::rt::spawn(async move {
        WebSocketService::handle_connection(user.id, session, stream).await;
    });

    Ok(response)
}

/// WebSocket 查询参数
#[derive(Debug, serde::Deserialize)]
pub struct WsQuery {
    pub token: String,
}

/// 验证 token 并获取用户
async fn validate_token_and_get_user(req: &HttpRequest, token: &str) -> Result<User, HttpResponse> {
    // 验证 JWT token
    crate::utils::jwt::JwtUtils::verify_access_token(token).map_err(|_| {
        HttpResponse::Unauthorized().json(ApiResponse::<()>::error_empty(
            ErrorCode::Unauthorized,
            "Invalid token",
        ))
    })?;

    // 尝试从缓存获取用户
    let cache = req
        .app_data::<web::Data<Arc<dyn ObjectCache>>>()
        .expect("Cache not found")
        .get_ref()
        .clone();

    if let CacheResult::Found(json) = cache.get_raw(&format!("user:{token}")).await
        && let Ok(user) = serde_json::from_str::<User>(&json)
    {
        return Ok(user);
    }

    // 从数据库获取用户
    let storage = req
        .app_data::<web::Data<Arc<dyn Storage>>>()
        .expect("Storage not found")
        .get_ref()
        .clone();

    let claims = crate::utils::jwt::JwtUtils::decode_token(token).map_err(|_| {
        HttpResponse::Unauthorized().json(ApiResponse::<()>::error_empty(
            ErrorCode::Unauthorized,
            "Invalid token format",
        ))
    })?;

    let user_id = claims.sub.parse::<i64>().map_err(|_| {
        HttpResponse::Unauthorized().json(ApiResponse::<()>::error_empty(
            ErrorCode::Unauthorized,
            "Invalid user ID",
        ))
    })?;

    let user = storage
        .get_user_by_id(user_id)
        .await
        .map_err(|_| {
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error_empty(
                ErrorCode::InternalServerError,
                "Failed to get user",
            ))
        })?
        .ok_or_else(|| {
            HttpResponse::Unauthorized().json(ApiResponse::<()>::error_empty(
                ErrorCode::Unauthorized,
                "User not found",
            ))
        })?;

    Ok(user)
}

/// WebSocket 状态端点
pub async fn ws_status() -> ActixResult<HttpResponse> {
    let online_count = crate::services::websocket::get_online_count();
    Ok(HttpResponse::Ok().json(ApiResponse::success(
        WebSocketStatusResponse {
            online_users: online_count,
            status: "ok".to_string(),
        },
        "WebSocket 服务正常",
    )))
}

/// 配置 WebSocket 路由
pub fn configure_websocket_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1/ws")
            // WebSocket 连接 - 添加速率限制防止 DDoS（20次/分钟/IP）
            .route(
                "",
                web::get()
                    .to(ws_handler)
                    .wrap(RateLimit::new(20, 60).with_prefix("ws_connect")),
            )
            // WebSocket 状态端点 - 需要 JWT 验证
            .route(
                "/status",
                web::get().to(ws_status).wrap(middlewares::RequireJWT),
            ),
    );
}
