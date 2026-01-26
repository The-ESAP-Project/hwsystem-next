use actix_web::{HttpRequest, HttpResponse, Result as ActixResult, web};
use once_cell::sync::Lazy;

use crate::middlewares::{self, RequireJWT};
use crate::models::grades::requests::{CreateGradeRequest, GradeListQuery, UpdateGradeRequest};
use crate::models::users::entities::UserRole;
use crate::models::{ApiResponse, ErrorCode};
use crate::services::GradeService;
use crate::utils::SafeIDI64;

// 懒加载的全局 GradeService 实例
static GRADE_SERVICE: Lazy<GradeService> = Lazy::new(GradeService::new_lazy);

// 列出评分
pub async fn list_grades(
    req: HttpRequest,
    query: web::Query<GradeListQuery>,
) -> ActixResult<HttpResponse> {
    GRADE_SERVICE.list_grades(&req, query.into_inner()).await
}

// 创建评分
pub async fn create_grade(
    req: HttpRequest,
    body: web::Json<CreateGradeRequest>,
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

    GRADE_SERVICE
        .create_grade(&req, user_id, body.into_inner())
        .await
}

// 获取评分详情
pub async fn get_grade(req: HttpRequest, path: SafeIDI64) -> ActixResult<HttpResponse> {
    GRADE_SERVICE.get_grade(&req, path.0).await
}

// 更新评分
pub async fn update_grade(
    req: HttpRequest,
    path: SafeIDI64,
    body: web::Json<UpdateGradeRequest>,
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

    GRADE_SERVICE
        .update_grade(&req, path.0, body.into_inner(), user_id)
        .await
}

// 配置路由
pub fn configure_grades_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1/grades")
            .wrap(middlewares::RequireJWT)
            .service(
                web::resource("")
                    // 列出评分 - 所有登录用户可访问（业务层会根据用户过滤）
                    .route(web::get().to(list_grades))
                    // 创建评分 - 仅教师和管理员
                    .route(
                        web::post()
                            .to(create_grade)
                            .wrap(middlewares::RequireRole::new_any(UserRole::teacher_roles())),
                    ),
            )
            .service(
                web::resource("/{id}")
                    // 获取评分详情 - 所有登录用户可访问（业务层会验证权限）
                    .route(web::get().to(get_grade))
                    // 更新评分 - 仅教师和管理员
                    .route(
                        web::put()
                            .to(update_grade)
                            .wrap(middlewares::RequireRole::new_any(UserRole::teacher_roles())),
                    ),
            ),
    );
}
