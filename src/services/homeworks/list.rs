use crate::middlewares::RequireJWT;
use crate::models::users::entities::UserRole;
use crate::models::{ApiResponse, ErrorCode, homeworks::requests::HomeworkListQuery};
use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};

use super::HomeworkService;

pub async fn list_homeworks(
    service: &HomeworkService,
    request: &HttpRequest,
    query: HomeworkListQuery,
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

    // 权限验证逻辑
    let mut filtered_query = query.clone();

    match current_user.role {
        UserRole::Admin => {
            // 管理员可以查看所有作业
        }
        UserRole::Teacher => {
            // 教师可以查看自己创建的作业，或者指定班级的作业（需要验证班级权限）
            if let Some(class_id) = query.class_id {
                // 验证教师是否有该班级的权限
                match storage
                    .get_class_user_by_user_id_and_class_id(current_user.id, class_id)
                    .await
                {
                    Ok(Some(_)) => {
                        // 教师是班级成员，允许访问
                    }
                    Ok(None) => {
                        // 检查是否是班级的创建教师（teacher_id）
                        match storage.get_class_by_id(class_id).await {
                            Ok(Some(class)) if class.teacher_id == current_user.id => {
                                // 是班级教师，允许访问
                            }
                            _ => {
                                return Ok(HttpResponse::Forbidden().json(
                                    ApiResponse::error_empty(
                                        ErrorCode::ClassPermissionDenied,
                                        "您无权查看该班级的作业",
                                    ),
                                ));
                            }
                        }
                    }
                    Err(e) => {
                        return Ok(HttpResponse::InternalServerError().json(
                            ApiResponse::error_empty(
                                ErrorCode::InternalServerError,
                                format!("验证班级权限失败: {e}"),
                            ),
                        ));
                    }
                }
            } else {
                // 教师未指定班级，默认只能看自己创建的作业
                filtered_query.created_by = Some(current_user.id);
            }
        }
        UserRole::User => {
            // 普通用户（学生）必须指定班级，且必须是班级成员
            if let Some(class_id) = query.class_id {
                // 验证用户是否为该班级成员
                match storage
                    .get_class_user_by_user_id_and_class_id(current_user.id, class_id)
                    .await
                {
                    Ok(Some(_)) => {
                        // 用户是班级成员，允许访问
                    }
                    Ok(None) => {
                        return Ok(HttpResponse::Forbidden().json(ApiResponse::error_empty(
                            ErrorCode::ClassPermissionDenied,
                            "您不是该班级成员，无权查看作业列表",
                        )));
                    }
                    Err(e) => {
                        return Ok(HttpResponse::InternalServerError().json(
                            ApiResponse::error_empty(
                                ErrorCode::InternalServerError,
                                format!("验证班级成员资格失败: {e}"),
                            ),
                        ));
                    }
                }
            } else {
                // 学生未指定班级，返回错误
                return Ok(HttpResponse::BadRequest().json(ApiResponse::error_empty(
                    ErrorCode::BadRequest,
                    "请指定班级 ID (class_id) 以查看作业列表",
                )));
            }
        }
    }

    match storage.list_homeworks_with_pagination(filtered_query).await {
        Ok(resp) => Ok(HttpResponse::Ok().json(ApiResponse::success(resp, "获取作业列表成功"))),
        Err(e) => Ok(
            HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                ErrorCode::InternalServerError,
                format!("获取作业列表失败: {e}"),
            )),
        ),
    }
}
