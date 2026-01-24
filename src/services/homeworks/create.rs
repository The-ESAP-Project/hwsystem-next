use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};

use super::HomeworkService;
use crate::middlewares::RequireJWT;
use crate::models::homeworks::requests::CreateHomeworkRequest;
use crate::models::users::entities::UserRole;
use crate::models::{ApiResponse, ErrorCode};

pub async fn create_homework(
    service: &HomeworkService,
    request: &HttpRequest,
    created_by: i64,
    req: CreateHomeworkRequest,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request);
    let user_role = RequireJWT::extract_user_role(request);

    // 检查班级是否存在
    let class = match storage.get_class_by_id(req.class_id).await {
        Ok(Some(class)) => class,
        Ok(None) => {
            return Ok(HttpResponse::NotFound().json(ApiResponse::error_empty(
                ErrorCode::ClassNotFound,
                "班级不存在",
            )));
        }
        Err(e) => {
            return Ok(
                HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                    ErrorCode::InternalServerError,
                    format!("查询班级失败: {e}"),
                )),
            );
        }
    };

    // 权限检查：只有该班级的教师或管理员才能创建作业
    match user_role {
        Some(UserRole::Admin) => {} // 管理员可以创建任何班级的作业
        Some(UserRole::Teacher) => {
            if class.teacher_id != created_by {
                return Ok(HttpResponse::Forbidden().json(ApiResponse::error_empty(
                    ErrorCode::Forbidden,
                    "只能在自己教授的班级创建作业",
                )));
            }
        }
        _ => {
            return Ok(HttpResponse::Forbidden().json(ApiResponse::error_empty(
                ErrorCode::Forbidden,
                "没有创建作业的权限",
            )));
        }
    }

    match storage.create_homework(created_by, req).await {
        Ok(homework) => {
            Ok(HttpResponse::Created().json(ApiResponse::success(homework, "创建成功")))
        }
        Err(e) => Ok(
            HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                ErrorCode::InternalServerError,
                format!("创建作业失败: {e}"),
            )),
        ),
    }
}
