use actix_web::{HttpRequest, HttpResponse, Result as ActixResult, web};
use once_cell::sync::Lazy;

use crate::middlewares::{self, RequireJWT};
use crate::models::homeworks::requests::{
    CreateHomeworkRequest, HomeworkListQuery, UpdateHomeworkRequest,
};
use crate::models::users::entities::UserRole;
use crate::models::{ApiResponse, ErrorCode};
use crate::services::HomeworkService;

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
pub async fn get_homework(req: HttpRequest, path: web::Path<i64>) -> ActixResult<HttpResponse> {
    HOMEWORK_SERVICE.get_homework(&req, path.into_inner()).await
}

// 更新作业
pub async fn update_homework(
    req: HttpRequest,
    path: web::Path<i64>,
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
        .update_homework(&req, path.into_inner(), body.into_inner(), user_id)
        .await
}

// 删除作业
pub async fn delete_homework(req: HttpRequest, path: web::Path<i64>) -> ActixResult<HttpResponse> {
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
        .delete_homework(&req, path.into_inner(), user_id)
        .await
}

// 获取作业统计
pub async fn get_homework_stats(
    req: HttpRequest,
    path: web::Path<i64>,
) -> ActixResult<HttpResponse> {
    HOMEWORK_SERVICE
        .get_homework_stats(&req, path.into_inner())
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
            ),
    );
}
