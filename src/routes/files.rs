use actix_web::{HttpRequest, HttpResponse, Result as ActixResult, middleware, web};
use once_cell::sync::Lazy;

use crate::middlewares::{self, RateLimit};
use crate::services::FileService;
use crate::utils::SafeFileToken;

// 懒加载的全局 FileService 实例
static FILE_SERVICE: Lazy<FileService> = Lazy::new(FileService::new_lazy);

pub async fn handle_upload(
    request: HttpRequest,
    payload: actix_multipart::Multipart,
) -> ActixResult<HttpResponse> {
    FILE_SERVICE.handle_upload(&request, payload).await
}

pub async fn handle_download(
    request: HttpRequest,
    file_token: SafeFileToken,
) -> ActixResult<HttpResponse> {
    FILE_SERVICE.handle_download(&request, file_token.0).await
}
// 配置路由
pub fn configure_file_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1/files")
            .wrap(middlewares::RequireJWT)
            .wrap(middleware::Compress::default())
            // 文件上传：10次/分钟/用户
            .service(
                web::resource("/upload")
                    .wrap(RateLimit::file_upload())
                    .route(web::post().to(handle_upload)),
            )
            .route("/download/{file_token}", web::get().to(handle_download)),
    );
}
