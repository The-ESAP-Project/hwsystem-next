//! 学生作业统计

use crate::middlewares::RequireJWT;
use crate::models::homeworks::responses::MyHomeworkStatsResponse;
use crate::models::{ApiResponse, ErrorCode};
use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};

use super::HomeworkService;

pub async fn get_my_homework_stats(
    service: &HomeworkService,
    request: &HttpRequest,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request);

    // 获取当前用户 ID
    let user_id = match RequireJWT::extract_user_id(request) {
        Some(id) => id,
        None => {
            return Ok(HttpResponse::Unauthorized()
                .json(ApiResponse::error_empty(ErrorCode::Unauthorized, "未登录")));
        }
    };

    match storage.get_my_homework_stats(user_id).await {
        Ok((pending, submitted, graded, total)) => {
            let response = MyHomeworkStatsResponse {
                pending,
                submitted,
                graded,
                total,
            };
            Ok(HttpResponse::Ok().json(ApiResponse::success(response, "获取作业统计成功")))
        }
        Err(e) => Ok(
            HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                ErrorCode::InternalServerError,
                format!("获取作业统计失败: {e}"),
            )),
        ),
    }
}
