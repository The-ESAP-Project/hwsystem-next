use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};
use std::sync::Arc;

use super::GradeService;
use crate::middlewares::RequireJWT;
use crate::models::users::entities::UserRole;
use crate::models::{ApiResponse, ErrorCode};
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
            return Err(HttpResponse::NotFound()
                .json(ApiResponse::error_empty(ErrorCode::NotFound, "提交不存在")));
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

    match current_user.role {
        UserRole::Teacher => {
            // 教师只能查看自己班级的评分
            let homework = match storage.get_homework_by_id(submission.homework_id).await {
                Ok(Some(hw)) => hw,
                _ => {
                    return Err(HttpResponse::NotFound()
                        .json(ApiResponse::error_empty(ErrorCode::NotFound, "作业不存在")));
                }
            };

            let class = match storage.get_class_by_id(homework.class_id).await {
                Ok(Some(c)) => c,
                _ => {
                    return Err(HttpResponse::NotFound().json(ApiResponse::error_empty(
                        ErrorCode::ClassNotFound,
                        "班级不存在",
                    )));
                }
            };

            if class.teacher_id != current_user.id {
                return Err(HttpResponse::Forbidden().json(ApiResponse::error_empty(
                    ErrorCode::Forbidden,
                    "只能查看自己班级的评分",
                )));
            }
        }
        UserRole::User => {
            // 学生只能查看自己的评分
            if submission.creator_id != current_user.id {
                return Err(HttpResponse::Forbidden().json(ApiResponse::error_empty(
                    ErrorCode::Forbidden,
                    "只能查看自己的评分",
                )));
            }
        }
        _ => {
            return Err(HttpResponse::Forbidden().json(ApiResponse::error_empty(
                ErrorCode::Forbidden,
                "没有查看评分的权限",
            )));
        }
    }

    Ok(())
}

pub async fn get_grade(
    service: &GradeService,
    request: &HttpRequest,
    grade_id: i64,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request);

    // 获取当前用户信息
    let current_user = match RequireJWT::extract_user_claims(request) {
        Some(user) => user,
        None => {
            return Ok(HttpResponse::Unauthorized()
                .json(ApiResponse::error_empty(ErrorCode::Unauthorized, "未登录")));
        }
    };

    // 获取评分
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

    // 权限验证
    if let Err(resp) =
        check_grade_access_permission(&storage, &current_user, grade.submission_id).await
    {
        return Ok(resp);
    }

    Ok(HttpResponse::Ok().json(ApiResponse::success(grade, "查询成功")))
}

pub async fn get_grade_by_submission(
    service: &GradeService,
    request: &HttpRequest,
    submission_id: i64,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request);

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

    match storage.get_grade_by_submission_id(submission_id).await {
        Ok(Some(grade)) => Ok(HttpResponse::Ok().json(ApiResponse::success(grade, "查询成功"))),
        Ok(None) => Ok(HttpResponse::NotFound().json(ApiResponse::error_empty(
            ErrorCode::NotFound,
            "该提交暂无评分",
        ))),
        Err(e) => Ok(
            HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                ErrorCode::InternalServerError,
                format!("查询评分失败: {e}"),
            )),
        ),
    }
}
