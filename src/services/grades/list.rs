use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};

use super::GradeService;
use crate::middlewares::RequireJWT;
use crate::models::grades::requests::GradeListQuery;
use crate::models::users::entities::UserRole;
use crate::models::{ApiResponse, ErrorCode};

pub async fn list_grades(
    service: &GradeService,
    request: &HttpRequest,
    query: GradeListQuery,
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

    // 权限过滤
    match current_user.role {
        UserRole::Admin => {
            // Admin 可查看所有评分，不需要过滤
        }
        UserRole::Teacher => {
            // 教师必须指定 homework_id，并验证是否有权限
            if let Some(homework_id) = query.homework_id {
                // 获取作业信息
                match storage.get_homework_by_id(homework_id).await {
                    Ok(Some(homework)) => {
                        // 获取班级信息
                        match storage.get_class_by_id(homework.class_id).await {
                            Ok(Some(class)) => {
                                if class.teacher_id != current_user.id {
                                    return Ok(HttpResponse::Forbidden().json(
                                        ApiResponse::error_empty(
                                            ErrorCode::Forbidden,
                                            "只能查看自己班级的评分",
                                        ),
                                    ));
                                }
                            }
                            _ => {
                                return Ok(HttpResponse::NotFound().json(
                                    ApiResponse::error_empty(
                                        ErrorCode::ClassNotFound,
                                        "班级不存在",
                                    ),
                                ));
                            }
                        }
                    }
                    _ => {
                        return Ok(HttpResponse::NotFound()
                            .json(ApiResponse::error_empty(ErrorCode::NotFound, "作业不存在")));
                    }
                }
            } else {
                // 教师未指定作业，返回错误
                return Ok(HttpResponse::BadRequest().json(ApiResponse::error_empty(
                    ErrorCode::BadRequest,
                    "请指定作业 ID (homework_id) 以查看评分列表",
                )));
            }
        }
        UserRole::User => {
            // 学生不能通过此接口查看评分列表
            return Ok(HttpResponse::Forbidden().json(ApiResponse::error_empty(
                ErrorCode::Forbidden,
                "学生请通过 /api/v1/submissions/{submission_id}/grade 查看评分",
            )));
        }
    }

    match storage.list_grades_with_pagination(query).await {
        Ok(response) => Ok(HttpResponse::Ok().json(ApiResponse::success(response, "查询成功"))),
        Err(e) => Ok(
            HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                ErrorCode::InternalServerError,
                format!("查询评分列表失败: {e}"),
            )),
        ),
    }
}
