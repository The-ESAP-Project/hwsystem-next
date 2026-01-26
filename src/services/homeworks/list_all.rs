//! 跨班级作业列表服务

use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};

use crate::middlewares::RequireJWT;
use crate::models::homeworks::requests::AllHomeworksQuery;
use crate::models::users::entities::UserRole;
use crate::models::{ApiResponse, ErrorCode};
use crate::services::homeworks::HomeworkService;

pub async fn list_all_homeworks(
    service: &HomeworkService,
    request: &HttpRequest,
    query: AllHomeworksQuery,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request);

    // 获取当前用户
    let current_user = match RequireJWT::extract_user_claims(request) {
        Some(user) => user,
        None => {
            return Ok(HttpResponse::Unauthorized().json(ApiResponse::error_empty(
                ErrorCode::Unauthorized,
                "未授权访问",
            )));
        }
    };

    // 判断是否为教师视角
    let is_teacher = matches!(current_user.role, UserRole::Teacher | UserRole::Admin);

    // 调用 storage 层
    match storage
        .list_all_homeworks(current_user.id, is_teacher, query)
        .await
    {
        Ok(resp) => Ok(HttpResponse::Ok().json(ApiResponse::success(resp, "获取作业列表成功"))),
        Err(e) => {
            tracing::error!("获取跨班级作业列表失败: {:?}", e);
            Ok(
                HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                    ErrorCode::InternalServerError,
                    format!("获取作业列表失败: {e}"),
                )),
            )
        }
    }
}
