use actix_web::{HttpRequest, HttpResponse, Result as ActixResult, web};
use once_cell::sync::Lazy;

use crate::middlewares::{self, RequireJWT};
use crate::models::homeworks::requests::{
    AllHomeworksQuery, CreateHomeworkRequest, HomeworkListQuery, UpdateHomeworkRequest,
};
use crate::models::users::entities::UserRole;
use crate::models::{ApiResponse, ErrorCode};
use crate::services::HomeworkService;
use crate::utils::SafeIDI64;

// 懒加载的全局 HomeworkService 实例
static HOMEWORK_SERVICE: Lazy<HomeworkService> = Lazy::new(HomeworkService::new_lazy);

// 列出作业
pub async fn list_homeworks(
    req: HttpRequest,
    query: web::Query<HomeworkListQuery>,
) -> ActixResult<HttpResponse> {
    HOMEWORK_SERVICE
        .list_homeworks(&req, query.into_inner())
        .await
}

// 创建作业
pub async fn create_homework(
    req: HttpRequest,
    body: web::Json<CreateHomeworkRequest>,
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

    HOMEWORK_SERVICE
        .create_homework(&req, user_id, body.into_inner())
        .await
}

// 获取作业详情
pub async fn get_homework(req: HttpRequest, path: SafeIDI64) -> ActixResult<HttpResponse> {
    HOMEWORK_SERVICE.get_homework(&req, path.0).await
}

// 更新作业
pub async fn update_homework(
    req: HttpRequest,
    path: SafeIDI64,
    body: web::Json<UpdateHomeworkRequest>,
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

    HOMEWORK_SERVICE
        .update_homework(&req, path.0, body.into_inner(), user_id)
        .await
}

// 删除作业
pub async fn delete_homework(req: HttpRequest, path: SafeIDI64) -> ActixResult<HttpResponse> {
    let user_id = match RequireJWT::extract_user_id(&req) {
        Some(id) => id,
        None => {
            return Ok(HttpResponse::Unauthorized().json(ApiResponse::error_empty(
                ErrorCode::Unauthorized,
                "无法获取用户信息",
            )));
        }
    };

    HOMEWORK_SERVICE
        .delete_homework(&req, path.0, user_id)
        .await
}

// 获取作业统计
pub async fn get_homework_stats(
    req: HttpRequest,
    path: SafeIDI64,
) -> ActixResult<HttpResponse> {
    HOMEWORK_SERVICE
        .get_homework_stats(&req, path.0)
        .await
}

// 导出作业统计
pub async fn export_homework_stats(
    req: HttpRequest,
    path: SafeIDI64,
) -> ActixResult<HttpResponse> {
    HOMEWORK_SERVICE
        .export_homework_stats(&req, path.0)
        .await
}

// 获取学生作业统计
pub async fn get_my_homework_stats(req: HttpRequest) -> ActixResult<HttpResponse> {
    HOMEWORK_SERVICE.get_my_homework_stats(&req).await
}

// 获取教师作业统计
pub async fn get_teacher_homework_stats(req: HttpRequest) -> ActixResult<HttpResponse> {
    HOMEWORK_SERVICE.get_teacher_homework_stats(&req).await
}

// 列出所有班级的作业（跨班级）
pub async fn list_all_homeworks(
    req: HttpRequest,
    query: web::Query<AllHomeworksQuery>,
) -> ActixResult<HttpResponse> {
    HOMEWORK_SERVICE
        .list_all_homeworks(&req, query.into_inner())
        .await
}

// 配置路由
pub fn configure_homeworks_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1/homeworks")
            .wrap(middlewares::RequireJWT)
            .service(
                web::resource("")
                    // 列出作业 - 所有登录用户可访问（业务层会根据用户过滤）
                    .route(web::get().to(list_homeworks))
                    // 创建作业 - 仅教师和管理员
                    .route(
                        web::post()
                            .to(create_homework)
                            .wrap(middlewares::RequireRole::new_any(UserRole::teacher_roles())),
                    ),
            )
            // 学生作业统计 - 所有登录用户可访问
            .service(web::resource("/my/stats").route(web::get().to(get_my_homework_stats)))
            // 教师作业统计 - 仅教师和管理员
            .service(
                web::resource("/teacher/stats")
                    .route(web::get().to(get_teacher_homework_stats))
                    .wrap(middlewares::RequireRole::new_any(UserRole::teacher_roles())),
            )
            // 跨班级作业列表 - 所有登录用户可访问（业务层根据角色返回不同数据）
            .service(web::resource("/all").route(web::get().to(list_all_homeworks)))
            .service(
                web::resource("/{id}")
                    // 获取作业详情 - 所有登录用户可访问（业务层会验证班级成员资格）
                    .route(web::get().to(get_homework))
                    // 更新作业 - 仅教师和管理员
                    .route(
                        web::put()
                            .to(update_homework)
                            .wrap(middlewares::RequireRole::new_any(UserRole::teacher_roles())),
                    )
                    // 删除作业 - 仅教师和管理员
                    .route(
                        web::delete()
                            .to(delete_homework)
                            .wrap(middlewares::RequireRole::new_any(UserRole::teacher_roles())),
                    ),
            )
            .service(
                web::resource("/{id}/stats")
                    // 权限在业务层检查（允许教师、课代表、管理员）
                    .route(web::get().to(get_homework_stats)),
            )
            .service(
                web::resource("/{id}/stats/export")
                    // 权限在业务层检查（允许教师、课代表、管理员）
                    .route(web::get().to(export_homework_stats)),
            ),
    );
}
