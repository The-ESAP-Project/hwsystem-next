use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};

use super::UserService;
use crate::models::{ApiResponse, ErrorCode, users::requests::UserListQuery};
use crate::services::StorageProvider;

pub async fn list_users(
    service: &UserService,
    query: UserListQuery,
    request: &HttpRequest,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request)?;

    match storage.list_users_with_pagination(query).await {
        Ok(response) => Ok(HttpResponse::Ok().json(ApiResponse::success(
            response,
            "User list retrieved successfully",
        ))),
        Err(e) => Ok(
            HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                ErrorCode::InternalServerError,
                format!("Failed to retrieve user list: {e}"),
            )),
        ),
    }
}
