use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};

use super::UserService;
use crate::{
    middlewares::RequireJWT,
    models::{ApiResponse, ErrorCode, users::entities::UserRole},
};

pub async fn delete_user(
    service: &UserService,
    user_id: i64,
    request: &HttpRequest,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request);

    // 获取当前操作者信息
    let current_user = match RequireJWT::extract_user_claims(request) {
        Some(user) => user,
        None => {
            return Ok(HttpResponse::Unauthorized()
                .json(ApiResponse::error_empty(ErrorCode::Unauthorized, "未登录")));
        }
    };

    // 禁止删除自己
    if user_id == current_user.id {
        return Ok(HttpResponse::BadRequest().json(ApiResponse::error_empty(
            ErrorCode::CanNotDeleteCurrentUser,
            "不能删除当前登录用户",
        )));
    }

    // 获取目标用户信息
    let target_user = match storage.get_user_by_id(user_id).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Ok(HttpResponse::NotFound().json(ApiResponse::error_empty(
                ErrorCode::UserNotFound,
                "用户不存在",
            )));
        }
        Err(e) => {
            return Ok(
                HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                    ErrorCode::InternalServerError,
                    format!("查询用户失败: {e}"),
                )),
            );
        }
    };

    // 禁止删除管理员用户（只有超级管理员才能删除，这里暂时完全禁止）
    if target_user.role == UserRole::Admin {
        return Ok(HttpResponse::Forbidden().json(ApiResponse::error_empty(
            ErrorCode::CanNotDeleteCurrentUser,
            "无法删除管理员用户",
        )));
    }

    match storage.delete_user(user_id).await {
        Ok(true) => Ok(HttpResponse::Ok().json(ApiResponse::success_empty("用户删除成功"))),
        Ok(false) => Ok(HttpResponse::NotFound().json(ApiResponse::error_empty(
            ErrorCode::UserNotFound,
            "用户不存在",
        ))),
        Err(e) => Ok(
            HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                ErrorCode::UserDeleteFailed,
                format!("用户删除失败: {e}"),
            )),
        ),
    }
}
