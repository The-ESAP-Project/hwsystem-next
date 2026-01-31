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
use crate::services::StorageProvider;
use crate::services::system::DynamicConfig;

pub async fn handle_upload(
    service: &FileService,
    req: &HttpRequest,
    mut payload: Multipart,
) -> ActixResult<HttpResponse> {
    // 获取配置（静态配置从 AppConfig，动态配置从 DynamicConfig）
    let config = AppConfig::get();
    let upload_dir = &config.upload.dir;
    let max_size = DynamicConfig::upload_max_size().await;
    let allowed_types = DynamicConfig::upload_allowed_types().await;

    // 确保上传目录存在
    if !Path::new(upload_dir).exists()
        && let Err(e) = fs::create_dir_all(upload_dir)
    {
        tracing::error!("{}", HWSystemError::file_operation(format!("{e}")));
        return Ok(
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error_empty(
                ErrorCode::FileUploadFailed,
                "创建上传目录失败",
            )),
        );
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
                    ErrorCode::MultifileUploadNotAllowed,
                    "Only one file can be uploaded at a time",
                )));
            }
            file_uploaded = true;

            // 先获取原始文件名
            original_name = content_disposition
                .and_then(|cd| cd.get_filename())
                .map(|s| s.to_string())
                .unwrap_or_default();

            // 提取扩展名并校验
            let extension = Path::new(&original_name)
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| format!(".{}", ext.to_lowercase()))
                .unwrap_or_default();

            if !allowed_types.iter().any(|t| t.to_lowercase() == extension) {
                return Ok(HttpResponse::BadRequest().json(ApiResponse::error_empty(
                    ErrorCode::FileTypeNotAllowed,
                    "File type not allowed",
                )));
            }

            // 获取 MIME 类型（用于存储记录，不用于校验）
            file_type = field
                .content_type()
                .map(|ct| ct.to_string())
                .unwrap_or_default();

            stored_name = format!("{}-{}.bin", chrono::Utc::now().timestamp(), Uuid::new_v4());
            let file_path = format!("{upload_dir}/{stored_name}");
            let mut f = match File::create(&file_path) {
                Ok(file) => file,
                Err(e) => {
                    tracing::error!("{}", HWSystemError::file_operation(format!("{e}")));
                    return Ok(HttpResponse::InternalServerError().json(
                        ApiResponse::<()>::error_empty(ErrorCode::FileUploadFailed, "文件创建失败"),
                    ));
                }
            };

            let mut total_size: usize = 0;
            let mut first_chunk = true;
            while let Some(chunk) = field.next().await {
                let data = chunk?;

                // 第一个 chunk 时验证魔术字节
                if first_chunk {
                    first_chunk = false;

                    // 用 infer 检测实际文件类型
                    match infer::get(&data) {
                        Some(kind) => {
                            let detected_ext = format!(".{}", kind.extension());

                            // 检查检测到的扩展名是否与用户声称的扩展名一致
                            if detected_ext.to_lowercase() != extension.to_lowercase() {
                                let _ = fs::remove_file(&file_path);
                                return Ok(HttpResponse::BadRequest().json(
                                    ApiResponse::error_empty(
                                        ErrorCode::FileTypeNotAllowed,
                                        format!(
                                            "文件内容与扩展名不匹配: 检测到 {}, 声称 {}",
                                            detected_ext, extension
                                        ),
                                    ),
                                ));
                            }
                        }
                        None => {
                            // 如果 infer 无法识别，对于文本文件允许通过，其他拒绝
                            if !matches!(
                                extension.as_str(),
                                ".txt"
                                    | ".md"
                                    | ".json"
                                    | ".csv"
                                    | ".xml"
                                    | ".html"
                                    | ".css"
                                    | ".js"
                                    | ".ts"
                            ) {
                                let _ = fs::remove_file(&file_path);
                                return Ok(HttpResponse::BadRequest().json(
                                    ApiResponse::error_empty(
                                        ErrorCode::FileTypeNotAllowed,
                                        "无法识别的文件类型",
                                    ),
                                ));
                            }
                        }
                    }
                }

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

    let storage = service.get_storage(req)?;

    let user_id = match RequireJWT::extract_user_id(req) {
        Some(id) => id,
        None => {
            return Ok(
                HttpResponse::Unauthorized().json(ApiResponse::<()>::error_empty(
                    ErrorCode::Unauthorized,
                    "用户未登录",
                )),
            );
        }
    };

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
