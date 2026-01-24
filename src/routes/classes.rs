use actix_web::{HttpRequest, HttpResponse, Result as ActixResult, web};
use once_cell::sync::Lazy;

use crate::middlewares::{self, RateLimit};
use crate::models::classes::requests::{ClassQueryParams, CreateClassRequest, UpdateClassRequest};
use crate::models::users::entities::UserRole;
use crate::services::ClassService;
use crate::utils::SafeClassIdI64;

// 懒加载的全局 CLASS_SERVICE 实例
static CLASS_SERVICE: Lazy<ClassService> = Lazy::new(ClassService::new_lazy);

// HTTP处理程序
pub async fn list_classes(
    req: HttpRequest,
    query: web::Query<ClassQueryParams>,
) -> ActixResult<HttpResponse> {
    CLASS_SERVICE.list_classes(&req, query.into_inner()).await
}

pub async fn create_class(
    req: HttpRequest,
    class_data: web::Json<CreateClassRequest>,
) -> ActixResult<HttpResponse> {
    CLASS_SERVICE
        .create_class(&req, class_data.into_inner())
        .await
}

pub async fn get_class_by_code(
    req: HttpRequest,
    code: web::Path<String>,
) -> ActixResult<HttpResponse> {
    CLASS_SERVICE
        .get_class_by_code(&req, code.into_inner())
        .await
}

pub async fn get_class(req: HttpRequest, class_id: SafeClassIdI64) -> ActixResult<HttpResponse> {
    CLASS_SERVICE.get_class(&req, class_id.0).await
}

pub async fn update_class(
    req: HttpRequest,
    class_id: SafeClassIdI64,
    update_data: web::Json<UpdateClassRequest>,
) -> ActixResult<HttpResponse> {
    CLASS_SERVICE
        .update_class(&req, class_id.0, update_data.into_inner())
        .await
}

pub async fn delete_class(req: HttpRequest, class_id: SafeClassIdI64) -> ActixResult<HttpResponse> {
    CLASS_SERVICE.delete_class(&req, class_id.0).await
}

// 配置路由
pub fn configure_classes_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1/classes")
            .wrap(middlewares::RequireJWT)
            .service(
                // 用户查询自己的班级列表，管理员可以查询所有班级
                web::resource("").route(web::get().to(list_classes)).route(
                    web::post()
                        .to(create_class)
                        // 教师创建自己的班级，管理员可以创建指定教师的班级
                        .wrap(middlewares::RequireRole::new_any(UserRole::teacher_roles())),
                ),
            )
            .service(
                web::resource("/code/{code}")
                    // 邀请码查询限制：10次/分钟/IP（防止暴力枚举）
                    .wrap(RateLimit::invite_code())
                    .route(
                        web::get()
                            .to(get_class_by_code)
                            // 学生使用邀请码查询班级信息
                            .wrap(middlewares::RequireRole::new(&UserRole::User)),
                    ),
            )
            .service(
                web::resource("/{class_id}")
                    .route(
                        web::get()
                            .to(get_class)
                            // 教师获取自己班级详情，管理员可以获取所有班级，学生获取自己加入的班级
                            .wrap(middlewares::RequireRole::new_any(UserRole::all_roles())),
                    )
                    .route(
                        web::put()
                            .to(update_class)
                            // 教师更新自己班级，管理员可以更新所有班级
                            .wrap(middlewares::RequireRole::new_any(UserRole::teacher_roles())),
                    )
                    .route(
                        web::delete()
                            .to(delete_class)
                            // 教师删除自己班级，管理员可以删除所有班级
                            .wrap(middlewares::RequireRole::new_any(UserRole::teacher_roles())),
                    ),
            ),
    );
}
