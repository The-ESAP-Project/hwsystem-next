use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};

use super::FileService;
use crate::middlewares::RequireJWT;
use crate::models::{ApiResponse, ErrorCode};

pub async fn handle_delete(
    service: &FileService,
    request: &HttpRequest,
    file_token: String,
) -> ActixResult<HttpResponse> {
    // 获取当前用户 ID
    let user_id = match RequireJWT::extract_user_id(request) {
        Some(id) => id,
        None => {
            return Ok(HttpResponse::Unauthorized()
                .json(ApiResponse::error_empty(ErrorCode::Unauthorized, "未登录")));
        }
    };

    let storage = service.get_storage(request)?;

    match storage.delete_file(&file_token, user_id).await {
        Ok(true) => Ok(HttpResponse::Ok().json(ApiResponse::success_empty("文件删除成功"))),
        Ok(false) => Ok(HttpResponse::NotFound().json(ApiResponse::error_empty(
            ErrorCode::FileNotFound,
            "文件不存在",
        ))),
        Err(e) => {
            // 检查是否是权限错误
            if e.to_string().contains("只有上传者可以删除") {
                return Ok(HttpResponse::Forbidden().json(ApiResponse::error_empty(
                    ErrorCode::PermissionDenied,
                    "只有上传者可以删除文件",
                )));
            }
            tracing::error!("删除文件失败: {:?}", e);
            Ok(
                HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                    ErrorCode::InternalServerError,
                    format!("删除文件失败: {e}"),
                )),
            )
        }
    }
}
