use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};

use super::SubmissionService;
use crate::middlewares::RequireJWT;
use crate::models::class_users::entities::ClassUserRole;
use crate::models::users::entities::UserRole;
use crate::models::{ApiResponse, ErrorCode};
use crate::services::StorageProvider;

pub async fn get_submission(
    service: &SubmissionService,
    request: &HttpRequest,
    submission_id: i64,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request)?;
    let user_role = RequireJWT::extract_user_role(request);
    let user_id = RequireJWT::extract_user_id(request);

    let user_id = match user_id {
        Some(id) => id,
        None => {
            return Ok(HttpResponse::Unauthorized().json(ApiResponse::error_empty(
                ErrorCode::Unauthorized,
                "无法获取用户信息",
            )));
        }
    };

    // 获取提交详情（完整响应，包含 creator、attachments、grade）
    let mut submission = match storage.get_submission_response(submission_id).await {
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

    // 权限检查
    // include_grades: 是否可以查看成绩（教师、管理员、本人可以，课代表不可以）
    let include_grades;

    if user_role == Some(UserRole::Admin) {
        // 管理员可以查看任何提交和成绩
        include_grades = true;
    } else {
        // 获取作业信息以确定班级
        let homework = match storage.get_homework_by_id(submission.homework_id).await {
            Ok(Some(hw)) => hw,
            Ok(None) => {
                return Ok(HttpResponse::NotFound().json(ApiResponse::error_empty(
                    ErrorCode::HomeworkNotFound,
                    "关联作业不存在",
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

        // 检查用户在班级中的角色
        let class_user = storage
            .get_class_user_by_user_id_and_class_id(user_id, homework.class_id)
            .await;

        match class_user {
            Ok(Some(cu)) => {
                if cu.role == ClassUserRole::Teacher {
                    // 教师可以查看班级内任何提交和成绩
                    include_grades = true;
                } else if cu.role == ClassUserRole::ClassRepresentative {
                    // 课代表可以查看提交，但不能查看成绩
                    include_grades = false;
                } else {
                    // 普通学生只能查看自己的提交
                    if submission.creator.id != user_id {
                        return Ok(HttpResponse::Forbidden().json(ApiResponse::error_empty(
                            ErrorCode::Forbidden,
                            "只能查看自己的提交",
                        )));
                    }
                    // 学生可以查看自己的成绩
                    include_grades = true;
                }
            }
            Ok(None) => {
                // 不是班级成员
                return Ok(HttpResponse::Forbidden().json(ApiResponse::error_empty(
                    ErrorCode::ClassPermissionDenied,
                    "您不是该班级成员",
                )));
            }
            Err(e) => {
                return Ok(
                    HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                        ErrorCode::InternalServerError,
                        format!("查询班级成员失败: {e}"),
                    )),
                );
            }
        }
    }

    // 如果不能查看成绩，将 grade 字段设为 None
    if !include_grades {
        submission.grade = None;
    }

    Ok(HttpResponse::Ok().json(ApiResponse::success(submission, "查询成功")))
}

pub async fn get_latest_submission(
    service: &SubmissionService,
    request: &HttpRequest,
    homework_id: i64,
    creator_id: i64,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request)?;

    // 使用 list_user_submissions（已含 grade 和 attachments）取最新一条
    match storage.list_user_submissions(homework_id, creator_id).await {
        Ok(items) => {
            if let Some(latest) = items.into_iter().next() {
                Ok(HttpResponse::Ok().json(ApiResponse::success(latest, "查询成功")))
            } else {
                Ok(HttpResponse::Ok().json(ApiResponse::success_empty("暂无提交")))
            }
        }
        Err(e) => Ok(
            HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                ErrorCode::InternalServerError,
                format!("查询提交失败: {e}"),
            )),
        ),
    }
}
