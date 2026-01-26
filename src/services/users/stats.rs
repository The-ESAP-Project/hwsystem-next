//! 用户统计服务

use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};

use crate::middlewares::RequireJWT;
use crate::models::{ApiResponse, ErrorCode};
use crate::services::users::UserService;

pub async fn get_my_stats(
    service: &UserService,
    request: &HttpRequest,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request);

    // 获取当前用户
    let current_user = match RequireJWT::extract_user_claims(request) {
        Some(user) => user,
        None => {
            return Ok(HttpResponse::Unauthorized().json(ApiResponse::error_empty(
                ErrorCode::Unauthorized,
                "未授权访问",
            )));
        }
    };

    // 调用 storage 层
    match storage
        .get_user_stats(current_user.id, current_user.role)
        .await
    {
        Ok(stats) => Ok(HttpResponse::Ok().json(ApiResponse::success(stats, "获取用户统计成功"))),
        Err(e) => {
            tracing::error!("获取用户统计失败: {:?}", e);
            Ok(
                HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                    ErrorCode::InternalServerError,
                    format!("获取用户统计失败: {e}"),
                )),
            )
        }
    }
}
