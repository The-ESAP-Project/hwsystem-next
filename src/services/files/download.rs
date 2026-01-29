use actix_files::NamedFile;
use actix_web::{HttpRequest, HttpResponse, Result as ActixResult, http::header};
use std::path::Path;

use super::FileService;
use crate::config::AppConfig;
use crate::errors::HWSystemError;
use crate::models::{ApiResponse, ErrorCode};

// TODO: 实现更细粒度的文件访问权限检查
// 目前 download_token 已经提供了一定程度的保护（需要知道 token 才能下载）
// 后续可考虑：
// 1. 作业附件：验证用户是否是该班级成员
// 2. 提交附件：验证用户是否是提交者或班级教师
// 3. 添加 token 过期机制

pub async fn handle_download(
    service: &FileService,
    request: &HttpRequest,
    file_token: String,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request)?;

    let db_file = match storage.get_file_by_token(&file_token).await {
        Ok(Some(f)) => f,
        Ok(None) => {
            return Ok(HttpResponse::NotFound().json(ApiResponse::error_empty(
                ErrorCode::FileNotFound,
                "File not found",
            )));
        }
        Err(e) => {
            return Ok(
                HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                    ErrorCode::InternalServerError,
                    format!("File query failed: {e}"),
                )),
            );
        }
    };

    let config = AppConfig::get();
    let upload_dir = &config.upload.dir;
    let file_path = format!("{}/{}", upload_dir, db_file.stored_name);

    if !Path::new(&file_path).exists() {
        return Ok(HttpResponse::NotFound().json(ApiResponse::error_empty(
            ErrorCode::FileNotFound,
            "文件不存在",
        )));
    }

    // 使用 NamedFile 进行流式传输（自动支持 Range 请求）
    match NamedFile::open(&file_path) {
        Ok(file) => {
            // 设置 Content-Disposition（使用原始文件名）
            let file = file.set_content_disposition(header::ContentDisposition {
                disposition: header::DispositionType::Attachment,
                parameters: vec![header::DispositionParam::Filename(
                    db_file.original_name.clone(),
                )],
            });

            Ok(file.into_response(request))
        }
        Err(e) => {
            tracing::error!("{:?}", HWSystemError::file_operation(format!("{e:?}")));
            Ok(
                HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                    ErrorCode::InternalServerError,
                    "File open failed",
                )),
            )
        }
    }
}
