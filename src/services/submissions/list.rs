use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};

use super::SubmissionService;
use crate::middlewares::RequireJWT;
use crate::models::submissions::requests::SubmissionListQuery;
use crate::models::users::entities::UserRole;
use crate::models::{ApiResponse, ErrorCode};
use crate::services::StorageProvider;

pub async fn list_submissions(
    service: &SubmissionService,
    request: &HttpRequest,
    mut query: SubmissionListQuery,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request)?;
    let user_role = RequireJWT::extract_user_role(request);
    let user_id = RequireJWT::extract_user_id(request);

    // 权限检查：学生只能看自己的提交，教师可以看班级所有提交
    match user_role {
        Some(UserRole::Admin) => {
            // 管理员可以查看所有提交，不需要过滤
        }
        Some(UserRole::Teacher) => {
            // 教师可以通过 homework_id 查看班级内的提交
            // 如果指定了 homework_id，验证教师是否有权限查看
            if let Some(homework_id) = query.homework_id {
                if let Some(uid) = user_id {
                    // 获取作业信息
                    if let Ok(Some(homework)) = storage.get_homework_by_id(homework_id).await {
                        // 获取班级信息
                        if let Ok(Some(class)) = storage.get_class_by_id(homework.class_id).await
                            && class.teacher_id != uid
                        {
                            return Ok(HttpResponse::Forbidden().json(ApiResponse::error_empty(
                                ErrorCode::Forbidden,
                                "只能查看自己班级的提交",
                            )));
                        }
                    }
                }
            } else {
                // 如果没有指定 homework_id，教师不能列出所有提交
                return Ok(HttpResponse::BadRequest().json(ApiResponse::error_empty(
                    ErrorCode::BadRequest,
                    "请指定作业ID来查看提交列表",
                )));
            }
        }
        Some(UserRole::User) | None => {
            // 学生只能查看自己的提交
            if let Some(uid) = user_id {
                query.creator_id = Some(uid);
            } else {
                return Ok(HttpResponse::Unauthorized().json(ApiResponse::error_empty(
                    ErrorCode::Unauthorized,
                    "无法获取用户信息",
                )));
            }
        }
    }

    match storage.list_submissions_with_pagination(query).await {
        Ok(response) => Ok(HttpResponse::Ok().json(ApiResponse::success(response, "查询成功"))),
        Err(e) => Ok(
            HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                ErrorCode::InternalServerError,
                format!("查询提交列表失败: {e}"),
            )),
        ),
    }
}
