//! API 响应辅助函数
//!
//! 提供统一的响应构建函数，确保所有 API 响应格式一致。

use actix_web::{HttpResponse, http::StatusCode};
use serde::Serialize;
use ts_rs::TS;

use super::{ApiResponse, ErrorCode};

/// 成功响应（带数据）
pub fn success<T: Serialize + TS>(data: T, message: impl Into<String>) -> HttpResponse {
    HttpResponse::Ok().json(ApiResponse::success(data, message))
}

/// 成功响应（无数据）
pub fn success_empty(message: impl Into<String>) -> HttpResponse {
    HttpResponse::Ok().json(ApiResponse::<()>::success_empty(message))
}

/// 错误响应
pub fn error(status: StatusCode, code: ErrorCode, message: impl Into<String>) -> HttpResponse {
    HttpResponse::build(status).json(ApiResponse::<()>::error_empty(code, message))
}

/// 400 Bad Request
pub fn bad_request(message: impl Into<String>) -> HttpResponse {
    error(StatusCode::BAD_REQUEST, ErrorCode::BadRequest, message)
}

/// 401 Unauthorized
pub fn unauthorized(message: impl Into<String>) -> HttpResponse {
    error(StatusCode::UNAUTHORIZED, ErrorCode::Unauthorized, message)
}

/// 403 Forbidden
pub fn forbidden(message: impl Into<String>) -> HttpResponse {
    error(StatusCode::FORBIDDEN, ErrorCode::Forbidden, message)
}

/// 404 Not Found
pub fn not_found(message: impl Into<String>) -> HttpResponse {
    error(StatusCode::NOT_FOUND, ErrorCode::NotFound, message)
}

/// 500 Internal Server Error
pub fn internal_error(message: impl Into<String>) -> HttpResponse {
    error(
        StatusCode::INTERNAL_SERVER_ERROR,
        ErrorCode::InternalServerError,
        message,
    )
}

/// 429 Rate Limit Exceeded
pub fn rate_limit_exceeded(message: impl Into<String>) -> HttpResponse {
    error(
        StatusCode::TOO_MANY_REQUESTS,
        ErrorCode::RateLimitExceeded,
        message,
    )
}

/// 409 Conflict
pub fn conflict(message: impl Into<String>) -> HttpResponse {
    error(StatusCode::CONFLICT, ErrorCode::Conflict, message)
}
