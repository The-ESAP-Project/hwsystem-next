use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};

use super::SubmissionService;
use crate::middlewares::RequireJWT;
use crate::models::users::entities::UserRole;
use crate::models::{ApiResponse, ErrorCode};

pub async fn delete_submission(
    service: &SubmissionService,
    request: &HttpRequest,
    submission_id: i64,
    user_id: i64,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request)?;
    let user_role = RequireJWT::extract_user_role(request);

    // 获取提交信息
    let submission = match storage.get_submission_by_id(submission_id).await {
        Ok(Some(sub)) => sub,
        Ok(None) => {
            return Ok(HttpResponse::NotFound().json(ApiResponse::error_empty(
                ErrorCode::SubmissionNotFound,
                "提交不存在",
            )));
        }
        Err(e) => {
            return Ok(
                HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                    ErrorCode::InternalServerError,
                    format!("查询提交失败: {e}"),
                )),
            );
        }
    };

    // 权限检查：只有提交者本人或管理员才能删除
    match user_role {
        Some(UserRole::Admin) => {
            // 管理员可以删除任何提交
        }
        _ => {
            // 其他用户只能删除自己的提交
            if submission.creator_id != user_id {
                return Ok(HttpResponse::Forbidden().json(ApiResponse::error_empty(
                    ErrorCode::Forbidden,
                    "只能撤回自己的提交",
                )));
            }
        }
    }

    match storage.delete_submission(submission_id).await {
        Ok(true) => Ok(HttpResponse::Ok().json(ApiResponse::success_empty("提交已撤回"))),
        Ok(false) => Ok(HttpResponse::NotFound().json(ApiResponse::error_empty(
            ErrorCode::SubmissionNotFound,
            "提交不存在",
        ))),
        Err(e) => Ok(
            HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                ErrorCode::InternalServerError,
                format!("撤回提交失败: {e}"),
            )),
        ),
    }
}
