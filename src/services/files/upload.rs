use actix_multipart::Multipart;
use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};
use futures_util::TryStreamExt;
use futures_util::stream::StreamExt;
use std::fs;
use std::io::Write;
use std::{fs::File, path::Path};
use uuid::Uuid;

use super::FileService;
use crate::config::AppConfig;
use crate::errors::HWSystemError;
use crate::middlewares::RequireJWT;
use crate::models::ErrorCode;
use crate::models::{ApiResponse, files::responses::FileUploadResponse};

pub async fn handle_upload(
    service: &FileService,
    req: &HttpRequest,
    mut payload: Multipart,
) -> ActixResult<HttpResponse> {
    // 获取配置
    let config = AppConfig::get();
    let upload_dir = &config.upload.dir;
    let max_size = config.upload.max_size;
    let allowed_types = &config.upload.allowed_types;

    // 确保上传目录存在
    if !Path::new(upload_dir).exists() {
        fs::create_dir_all(upload_dir).map_err(|e| {
            tracing::error!("{}", HWSystemError::file_operation(format!("{e}")));
            actix_web::error::ErrorInternalServerError(HWSystemError::file_operation(
                "file create error",
            ))
        })?;
    }

    // 文件相关信息
    let mut original_name = String::new();
    let mut file_size: i64 = 0;
    let mut file_uploaded = false;
    let mut file_type = String::new();
    let mut stored_name = String::new();

    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_disposition = field.content_disposition();
        let name = content_disposition
            .and_then(|cd| cd.get_name())
            .unwrap_or_default()
            .to_string();

        if name == "file" {
            if file_uploaded {
                return Ok(HttpResponse::BadRequest().json(ApiResponse::error_empty(
                    ErrorCode::MuitifileUploadNotAllowed,
                    "Only one file can be uploaded at a time",
                )));
            }
            file_uploaded = true;
            // 获取文件类型
            file_type = field
                .content_type()
                .map(|ct| ct.to_string())
                .unwrap_or_default();
            // 校验类型
            if !allowed_types.iter().any(|t| file_type.contains(t)) {
                return Ok(HttpResponse::BadRequest().json(ApiResponse::error_empty(
                    ErrorCode::FileTypeNotAllowed,
                    "File type not allowed",
                )));
            }

            // 获取原始文件名
            original_name = content_disposition
                .and_then(|cd| cd.get_filename())
                .map(|s| s.to_string())
                .unwrap_or_default();

            stored_name = format!("{}-{}.bin", chrono::Utc::now().timestamp(), Uuid::new_v4());
            let file_path = format!("{upload_dir}/{stored_name}");
            let mut f = File::create(&file_path).map_err(|e| {
                tracing::error!("{}", HWSystemError::file_operation(format!("{e}")));
                actix_web::error::ErrorInternalServerError(HWSystemError::file_operation(
                    "file create error",
                ))
            })?;

            let mut total_size: usize = 0;
            while let Some(chunk) = field.next().await {
                let data = chunk?;
                total_size += data.len();
                // 校验大小
                if total_size > max_size {
                    let _ = fs::remove_file(&file_path);
                    return Ok(HttpResponse::BadRequest().json(ApiResponse::error_empty(
                        ErrorCode::FileSizeExceeded,
                        "File size exceeds the limit",
                    )));
                }
                f.write_all(&data)?;
            }
            file_size = total_size as i64;
        }
    }

    if !file_uploaded {
        return Ok(HttpResponse::BadRequest().json(ApiResponse::error_empty(
            ErrorCode::FileNotFound,
            "No file found in upload payload",
        )));
    }

    let storage = service.get_storage(req);

    let user_id = RequireJWT::extract_user_id(req)
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("User not authenticated"))?;

    let db_file = match storage
        .upload_file(
            &original_name,
            &stored_name,
            &file_size,
            &file_type,
            user_id,
        )
        .await
    {
        Ok(file) => FileUploadResponse {
            download_token: file.download_token,
            file_name: file.original_name,
            size: file.file_size,
            content_type: file.file_type,
            created_at: file.created_at,
        },
        Err(e) => {
            return Ok(
                HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                    ErrorCode::FileUploadFailed,
                    format!("Failed to upload file: {e}"),
                )),
            );
        }
    };

    Ok(HttpResponse::Ok().json(ApiResponse::success(db_file, "File uploaded successfully")))
}
