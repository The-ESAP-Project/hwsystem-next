use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};
use std::sync::Arc;

use super::SubmissionService;
use crate::middlewares::RequireJWT;
use crate::models::class_users::entities::ClassUserRole;
use crate::models::users::entities::UserRole;
use crate::models::{ApiResponse, ErrorCode};
use crate::services::StorageProvider;
use crate::storage::Storage;

/// 检查用户是否有权限访问某个提交的评分
async fn check_grade_access_permission(
    storage: &Arc<dyn Storage>,
    current_user: &crate::models::users::entities::User,
    submission_id: i64,
) -> Result<(), HttpResponse> {
    // Admin 直接放行
    if current_user.role == UserRole::Admin {
        return Ok(());
    }

    // 获取提交信息
    let submission = match storage.get_submission_by_id(submission_id).await {
        Ok(Some(sub)) => sub,
        Ok(None) => {
            return Err(HttpResponse::NotFound().json(ApiResponse::error_empty(
                ErrorCode::SubmissionNotFound,
                "提交不存在",
            )));
        }
        Err(e) => {
            return Err(
                HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                    ErrorCode::InternalServerError,
                    format!("查询提交失败: {e}"),
                )),
            );
        }
    };

    // 如果是提交者本人，允许查看自己的成绩
    if submission.creator_id == current_user.id {
        return Ok(());
    }

    // 获取作业信息以确定班级
    let homework = match storage.get_homework_by_id(submission.homework_id).await {
        Ok(Some(hw)) => hw,
        Ok(None) => {
            return Err(HttpResponse::NotFound().json(ApiResponse::error_empty(
                ErrorCode::HomeworkNotFound,
                "作业不存在",
            )));
        }
        Err(e) => {
            return Err(
                HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                    ErrorCode::InternalServerError,
                    format!("查询作业失败: {e}"),
                )),
            );
        }
    };

    // 检查用户在班级中的角色
    let class_user = match storage
        .get_class_user_by_user_id_and_class_id(current_user.id, homework.class_id)
        .await
    {
        Ok(Some(cu)) => cu,
        Ok(None) => {
            return Err(HttpResponse::Forbidden().json(ApiResponse::error_empty(
                ErrorCode::ClassPermissionDenied,
                "您不是该班级成员",
            )));
        }
        Err(e) => {
            return Err(
                HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                    ErrorCode::InternalServerError,
                    format!("查询班级成员失败: {e}"),
                )),
            );
        }
    };

    // 教师可以查看班级内的成绩
    if class_user.role == ClassUserRole::Teacher {
        return Ok(());
    }

    // 其他角色（包括课代表和学生）只能查看自己的成绩（上面已处理）
    Err(HttpResponse::Forbidden().json(ApiResponse::error_empty(
        ErrorCode::Forbidden,
        "没有查看该评分的权限",
    )))
}

/// 获取提交的评分
/// GET /submissions/{id}/grade
pub async fn get_submission_grade(
    service: &SubmissionService,
    request: &HttpRequest,
    submission_id: i64,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request)?;

    // 获取当前用户信息
    let current_user = match RequireJWT::extract_user_claims(request) {
        Some(user) => user,
        None => {
            return Ok(HttpResponse::Unauthorized()
                .json(ApiResponse::error_empty(ErrorCode::Unauthorized, "未登录")));
        }
    };

    // 权限验证
    if let Err(resp) = check_grade_access_permission(&storage, &current_user, submission_id).await {
        return Ok(resp);
    }

    // 获取评分
    match storage.get_grade_by_submission_id(submission_id).await {
        Ok(Some(grade)) => Ok(HttpResponse::Ok().json(ApiResponse::success(grade, "查询成功"))),
        Ok(None) => Ok(HttpResponse::NotFound().json(ApiResponse::error_empty(
            ErrorCode::GradeNotFound,
            "该提交尚未评分",
        ))),
        Err(e) => Ok(
            HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                ErrorCode::InternalServerError,
                format!("查询评分失败: {e}"),
            )),
        ),
    }
}
