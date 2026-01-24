use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};

use super::GradeService;
use crate::middlewares::RequireJWT;
use crate::models::grades::requests::UpdateGradeRequest;
use crate::models::users::entities::UserRole;
use crate::models::{ApiResponse, ErrorCode};

pub async fn update_grade(
    service: &GradeService,
    request: &HttpRequest,
    grade_id: i64,
    req: UpdateGradeRequest,
    user_id: i64,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request);
    let user_role = RequireJWT::extract_user_role(request);

    // 获取评分信息
    let grade = match storage.get_grade_by_id(grade_id).await {
        Ok(Some(g)) => g,
        Ok(None) => {
            return Ok(HttpResponse::NotFound()
                .json(ApiResponse::error_empty(ErrorCode::NotFound, "评分不存在")));
        }
        Err(e) => {
            return Ok(
                HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                    ErrorCode::InternalServerError,
                    format!("查询评分失败: {e}"),
                )),
            );
        }
    };

    // 权限检查：只有原评分者或管理员才能更新
    match user_role {
        Some(UserRole::Admin) => {} // 管理员可以更新任何评分
        Some(UserRole::Teacher) => {
            if grade.grader_id != user_id {
                return Ok(HttpResponse::Forbidden().json(ApiResponse::error_empty(
                    ErrorCode::Forbidden,
                    "只能更新自己创建的评分",
                )));
            }
        }
        _ => {
            return Ok(HttpResponse::Forbidden().json(ApiResponse::error_empty(
                ErrorCode::Forbidden,
                "没有更新评分的权限",
            )));
        }
    }

    match storage.update_grade(grade_id, req).await {
        Ok(Some(grade)) => Ok(HttpResponse::Ok().json(ApiResponse::success(grade, "更新成功"))),
        Ok(None) => Ok(HttpResponse::NotFound()
            .json(ApiResponse::error_empty(ErrorCode::NotFound, "评分不存在"))),
        Err(e) => Ok(
            HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                ErrorCode::InternalServerError,
                format!("更新评分失败: {e}"),
            )),
        ),
    }
}
