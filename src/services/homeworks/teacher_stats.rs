//! 教师作业统计

use crate::middlewares::RequireJWT;
use crate::models::homeworks::responses::TeacherHomeworkStatsResponse;
use crate::models::users::entities::UserRole;
use crate::models::{ApiResponse, ErrorCode};
use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};

use super::HomeworkService;

pub async fn get_teacher_homework_stats(
    service: &HomeworkService,
    request: &HttpRequest,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request);

    // 获取当前用户信息
    let user = match RequireJWT::extract_user_claims(request) {
        Some(u) => u,
        None => {
            return Ok(HttpResponse::Unauthorized()
                .json(ApiResponse::error_empty(ErrorCode::Unauthorized, "未登录")));
        }
    };

    // 验证权限：只有教师和管理员可以访问
    if user.role != UserRole::Teacher && user.role != UserRole::Admin {
        return Ok(HttpResponse::Forbidden().json(ApiResponse::error_empty(
            ErrorCode::PermissionDenied,
            "只有教师和管理员可以访问此接口",
        )));
    }

    match storage.get_teacher_homework_stats(user.id).await {
        Ok((total_homeworks, pending_review, total_submissions, graded_submissions)) => {
            let response = TeacherHomeworkStatsResponse {
                total_homeworks,
                pending_review,
                total_submissions,
                graded_submissions,
            };
            Ok(HttpResponse::Ok().json(ApiResponse::success(response, "获取教师统计成功")))
        }
        Err(e) => Ok(
            HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                ErrorCode::InternalServerError,
                format!("获取教师统计失败: {e}"),
            )),
        ),
    }
}
