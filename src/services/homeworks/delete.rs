use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};

use super::HomeworkService;
use crate::middlewares::RequireJWT;
use crate::models::users::entities::UserRole;
use crate::models::{ApiResponse, ErrorCode};

pub async fn delete_homework(
    service: &HomeworkService,
    request: &HttpRequest,
    homework_id: i64,
    user_id: i64,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request)?;
    let user_role = RequireJWT::extract_user_role(request);

    // 获取作业信息
    let homework = match storage.get_homework_by_id(homework_id).await {
        Ok(Some(hw)) => hw,
        Ok(None) => {
            return Ok(HttpResponse::NotFound().json(ApiResponse::error_empty(
                ErrorCode::HomeworkNotFound,
                "作业不存在",
            )));
        }
        Err(e) => {
            return Ok(
                HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                    ErrorCode::InternalServerError,
                    format!("查询作业失败: {e}"),
                )),
            );
        }
    };

    // 权限检查：只有作业创建者或管理员才能删除
    match user_role {
        Some(UserRole::Admin) => {} // 管理员可以删除任何作业
        Some(UserRole::Teacher) => {
            if homework.created_by != user_id {
                return Ok(HttpResponse::Forbidden().json(ApiResponse::error_empty(
                    ErrorCode::Forbidden,
                    "只能删除自己创建的作业",
                )));
            }
        }
        _ => {
            return Ok(HttpResponse::Forbidden().json(ApiResponse::error_empty(
                ErrorCode::Forbidden,
                "没有删除作业的权限",
            )));
        }
    }

    match storage.delete_homework(homework_id).await {
        Ok(true) => Ok(HttpResponse::Ok().json(ApiResponse::success_empty("作业已删除"))),
        Ok(false) => Ok(HttpResponse::NotFound().json(ApiResponse::error_empty(
            ErrorCode::HomeworkNotFound,
            "作业不存在",
        ))),
        Err(e) => Ok(
            HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                ErrorCode::InternalServerError,
                format!("删除作业失败: {e}"),
            )),
        ),
    }
}
