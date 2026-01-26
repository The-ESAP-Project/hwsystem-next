use actix_multipart::Multipart;
use actix_web::{HttpRequest, HttpResponse, Result as ActixResult, web};
use once_cell::sync::Lazy;

use crate::middlewares;
use crate::models::users::entities::UserRole;
use crate::models::users::requests::{
    CreateUserRequest, ImportTemplateParams, UpdateUserRequest, UserExportParams, UserListParams,
};
use crate::services::UserService;
use crate::utils::SafeIDI64;

// 懒加载的全局 UserService 实例
static USER_SERVICE: Lazy<UserService> = Lazy::new(UserService::new_lazy);

// HTTP处理程序
pub async fn list_users(
    req: HttpRequest,
    query: web::Query<UserListParams>,
) -> ActixResult<HttpResponse> {
    USER_SERVICE.list_users(query.into_inner(), &req).await
}

pub async fn create_user(
    req: HttpRequest,
    user_data: web::Json<CreateUserRequest>,
) -> ActixResult<HttpResponse> {
    USER_SERVICE.create_user(user_data.into_inner(), &req).await
}

pub async fn get_user(req: HttpRequest, user_id: SafeIDI64) -> ActixResult<HttpResponse> {
    USER_SERVICE.get_user(user_id.0, &req).await
}

pub async fn update_user(
    req: HttpRequest,
    user_id: SafeIDI64,
    update_data: web::Json<UpdateUserRequest>,
) -> ActixResult<HttpResponse> {
    USER_SERVICE
        .update_user(user_id.0, update_data.into_inner(), &req)
        .await
}

pub async fn delete_user(req: HttpRequest, user_id: SafeIDI64) -> ActixResult<HttpResponse> {
    USER_SERVICE.delete_user(user_id.0, &req).await
}

pub async fn export_users(
    req: HttpRequest,
    query: web::Query<UserExportParams>,
) -> ActixResult<HttpResponse> {
    USER_SERVICE.export_users(query.into_inner(), &req).await
}

pub async fn import_users(req: HttpRequest, payload: Multipart) -> ActixResult<HttpResponse> {
    USER_SERVICE.import_users(payload, &req).await
}

pub async fn download_import_template(
    query: web::Query<ImportTemplateParams>,
) -> ActixResult<HttpResponse> {
    USER_SERVICE.download_import_template(&query.format).await
}

pub async fn get_my_stats(req: HttpRequest) -> ActixResult<HttpResponse> {
    USER_SERVICE.get_my_stats(&req).await
}

// 配置路由
pub fn configure_user_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1/users")
            .wrap(middlewares::RequireJWT)
            // 所有登录用户可访问的路由
            .service(web::resource("/me/stats").route(web::get().to(get_my_stats)))
            // 管理员专属路由
            .service(
                web::scope("")
                    .wrap(middlewares::RequireRole::new_any(UserRole::admin_roles()))
                    .route("", web::get().to(list_users))
                    .route("", web::post().to(create_user))
                    .route("/export", web::get().to(export_users))
                    .route("/import", web::post().to(import_users))
                    .route("/import/template", web::get().to(download_import_template))
                    .route("/{id}", web::get().to(get_user))
                    .route("/{id}", web::put().to(update_user))
                    .route("/{id}", web::delete().to(delete_user)),
            ),
    );
}
