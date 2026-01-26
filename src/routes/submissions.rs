use actix_web::{HttpRequest, HttpResponse, Result as ActixResult, web};
use once_cell::sync::Lazy;

use crate::middlewares::{self, RequireJWT};
use crate::models::submissions::requests::{
    CreateSubmissionRequest, SubmissionListQuery, SubmissionSummaryQuery,
};
use crate::models::{ApiResponse, ErrorCode};
use crate::services::SubmissionService;
use crate::utils::{SafeHomeworkIdI64, SafeIDI64};

// 懒加载的全局 SubmissionService 实例
static SUBMISSION_SERVICE: Lazy<SubmissionService> = Lazy::new(SubmissionService::new_lazy);

// 列出提交
pub async fn list_submissions(
    req: HttpRequest,
    query: web::Query<SubmissionListQuery>,
) -> ActixResult<HttpResponse> {
    SUBMISSION_SERVICE
        .list_submissions(&req, query.into_inner())
        .await
}

// 创建提交
pub async fn create_submission(
    req: HttpRequest,
    body: web::Json<CreateSubmissionRequest>,
) -> ActixResult<HttpResponse> {
    let user = match RequireJWT::extract_user_claims(&req) {
        Some(u) => u,
        None => {
            return Ok(HttpResponse::Unauthorized().json(ApiResponse::error_empty(
                ErrorCode::Unauthorized,
                "无法获取用户信息",
            )));
        }
    };

    SUBMISSION_SERVICE
        .create_submission(&req, user.id, user.role, body.into_inner())
        .await
}

// 获取提交详情
pub async fn get_submission(req: HttpRequest, path: SafeIDI64) -> ActixResult<HttpResponse> {
    SUBMISSION_SERVICE.get_submission(&req, path.0).await
}

// 获取我的最新提交
pub async fn get_my_latest_submission(
    req: HttpRequest,
    path: SafeHomeworkIdI64, // homework_id
) -> ActixResult<HttpResponse> {
    let user_id = match RequireJWT::extract_user_id(&req) {
        Some(id) => id,
        None => {
            return Ok(HttpResponse::Unauthorized().json(ApiResponse::error_empty(
                ErrorCode::Unauthorized,
                "无法获取用户信息",
            )));
        }
    };

    SUBMISSION_SERVICE
        .get_latest_submission(&req, path.0, user_id)
        .await
}

// 获取我的提交历史
pub async fn list_my_submissions(
    req: HttpRequest,
    path: SafeHomeworkIdI64, // homework_id
) -> ActixResult<HttpResponse> {
    let user_id = match RequireJWT::extract_user_id(&req) {
        Some(id) => id,
        None => {
            return Ok(HttpResponse::Unauthorized().json(ApiResponse::error_empty(
                ErrorCode::Unauthorized,
                "无法获取用户信息",
            )));
        }
    };

    SUBMISSION_SERVICE
        .list_user_submissions(&req, path.0, user_id)
        .await
}

// 删除/撤回提交
pub async fn delete_submission(req: HttpRequest, path: SafeIDI64) -> ActixResult<HttpResponse> {
    let user_id = match RequireJWT::extract_user_id(&req) {
        Some(id) => id,
        None => {
            return Ok(HttpResponse::Unauthorized().json(ApiResponse::error_empty(
                ErrorCode::Unauthorized,
                "无法获取用户信息",
            )));
        }
    };

    SUBMISSION_SERVICE
        .delete_submission(&req, path.0, user_id)
        .await
}

// 获取提交概览（按学生聚合）
pub async fn get_submission_summary(
    req: HttpRequest,
    path: SafeHomeworkIdI64, // homework_id
    query: web::Query<SubmissionSummaryQuery>,
) -> ActixResult<HttpResponse> {
    SUBMISSION_SERVICE
        .get_submission_summary(&req, path.0, query.page, query.size, query.graded)
        .await
}

// 获取某学生某作业的所有版本（教师视角）
pub async fn list_user_submissions_for_teacher(
    req: HttpRequest,
    path: web::Path<(i64, i64)>, // (homework_id, user_id)
) -> ActixResult<HttpResponse> {
    let (homework_id, user_id) = path.into_inner();
    SUBMISSION_SERVICE
        .list_user_submissions_for_teacher(&req, homework_id, user_id)
        .await
}

// 配置路由
pub fn configure_submissions_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1/submissions")
            .wrap(middlewares::RequireJWT)
            .route("", web::get().to(list_submissions))
            .route("", web::post().to(create_submission))
            .route("/{id}", web::get().to(get_submission))
            .route("/{id}", web::delete().to(delete_submission)),
    );

    // 作业相关的提交路由
    cfg.service(
        web::scope("/api/v1/homeworks/{homework_id}/submissions")
            .wrap(middlewares::RequireJWT)
            .route("/my/latest", web::get().to(get_my_latest_submission))
            .route("/my", web::get().to(list_my_submissions))
            .route("/summary", web::get().to(get_submission_summary))
            .route(
                "/user/{user_id}",
                web::get().to(list_user_submissions_for_teacher),
            ),
    );
}
