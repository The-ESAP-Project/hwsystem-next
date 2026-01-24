use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};

use super::SubmissionService;
use crate::middlewares::RequireJWT;
use crate::models::users::entities::UserRole;
use crate::models::{ApiResponse, ErrorCode};

pub async fn get_submission(
    service: &SubmissionService,
    request: &HttpRequest,
    submission_id: i64,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request);
    let user_role = RequireJWT::extract_user_role(request);
    let user_id = RequireJWT::extract_user_id(request);

    // 获取提交信息
    let submission = match storage.get_submission_by_id(submission_id).await {
        Ok(Some(sub)) => sub,
        Ok(None) => {
            return Ok(HttpResponse::NotFound()
                .json(ApiResponse::error_empty(ErrorCode::NotFound, "提交不存在")));
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
    match user_role {
        Some(UserRole::Admin) => {
            // 管理员可以查看任何提交
        }
        Some(UserRole::Teacher) => {
            // 教师只能查看自己班级的提交
            if let Some(uid) = user_id {
                // 获取作业信息
                if let Ok(Some(homework)) = storage.get_homework_by_id(submission.homework_id).await
                {
                    // 获取班级信息
                    if let Ok(Some(class)) = storage.get_class_by_id(homework.class_id).await
                        && class.teacher_id != uid {
                            return Ok(HttpResponse::Forbidden().json(ApiResponse::error_empty(
                                ErrorCode::Forbidden,
                                "只能查看自己班级的提交",
                            )));
                        }
                }
            }
        }
        Some(UserRole::User) | None => {
            // 学生只能查看自己的提交
            if let Some(uid) = user_id {
                if submission.creator_id != uid {
                    return Ok(HttpResponse::Forbidden().json(ApiResponse::error_empty(
                        ErrorCode::Forbidden,
                        "只能查看自己的提交",
                    )));
                }
            } else {
                return Ok(HttpResponse::Unauthorized().json(ApiResponse::error_empty(
                    ErrorCode::Unauthorized,
                    "无法获取用户信息",
                )));
            }
        }
    }

    Ok(HttpResponse::Ok().json(ApiResponse::success(submission, "查询成功")))
}

pub async fn get_latest_submission(
    service: &SubmissionService,
    request: &HttpRequest,
    homework_id: i64,
    creator_id: i64,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request);

    match storage.get_latest_submission(homework_id, creator_id).await {
        Ok(Some(submission)) => {
            Ok(HttpResponse::Ok().json(ApiResponse::success(submission, "查询成功")))
        }
        // 尚未提交时返回 200 + null，而不是 404
        Ok(None) => Ok(HttpResponse::Ok().json(ApiResponse::success_empty("暂无提交"))),
        Err(e) => Ok(
            HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                ErrorCode::InternalServerError,
                format!("查询提交失败: {e}"),
            )),
        ),
    }
}
