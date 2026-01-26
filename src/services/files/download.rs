use actix_web::{HttpRequest, HttpResponse, Result as ActixResult, http::header};
use std::fs::File;
use std::io::Read;
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
    let storage = service.get_storage(request);

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

    let mut file = match File::open(&file_path) {
        Ok(f) => f,
        Err(e) => {
            tracing::error!("{:?}", HWSystemError::file_operation(format!("{e:?}")));
            return Ok(
                HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                    ErrorCode::InternalServerError,
                    "File open failed",
                )),
            );
        }
    };

    let mut buf = Vec::new();
    if file.read_to_end(&mut buf).is_err() {
        tracing::error!("{:?}", HWSystemError::file_operation("File read failed"));
        return Ok(
            HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                ErrorCode::InternalServerError,
                "File read failed",
            )),
        );
    }

    // 使用数据库中的原始文件名
    Ok(HttpResponse::Ok()
        .insert_header((header::CONTENT_TYPE, db_file.file_type.as_str()))
        .insert_header((
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", db_file.original_name),
        ))
        .body(buf))
}
