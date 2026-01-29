use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};

use super::UserService;
use crate::models::users::responses::UserResponse;
use crate::models::{ApiResponse, ErrorCode};
use crate::services::StorageProvider;

pub async fn get_user(
    service: &UserService,
    user_id: i64,
    request: &HttpRequest,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request)?;

    match storage.get_user_by_id(user_id).await {
        Ok(Some(user)) => Ok(HttpResponse::Ok().json(ApiResponse::success(
            UserResponse { user },
            "User information retrieved successfully",
        ))),
        Ok(None) => Ok(HttpResponse::NotFound().json(ApiResponse::error_empty(
            ErrorCode::UserNotFound,
            "User not found",
        ))),
        Err(e) => Ok(
            HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                ErrorCode::InternalServerError,
                format!("Failed to get user information: {e}"),
            )),
        ),
    }
}
