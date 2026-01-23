pub mod require_class_role;
pub mod require_jwt;
pub mod require_role;

use actix_web::{
    HttpResponse,
    http::{StatusCode, header::CONTENT_TYPE},
};
pub use require_class_role::RequireClassRole;
pub use require_jwt::RequireJWT;
pub use require_role::RequireRole;

use crate::models::{ApiResponse, ErrorCode};

// 辅助函数：创建错误响应
fn create_error_response(status: StatusCode, error_code: ErrorCode, message: &str) -> HttpResponse {
    HttpResponse::build(status)
        .insert_header((CONTENT_TYPE, "application/json; charset=utf-8"))
        .json(ApiResponse::<()>::error_empty(error_code, message))
}
