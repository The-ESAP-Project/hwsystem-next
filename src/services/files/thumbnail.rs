use actix_files::NamedFile;
use actix_web::{HttpRequest, HttpResponse, Result as ActixResult, http::header};
use image::ImageReader;
use image::imageops::FilterType;
use std::fs;
use std::io::{BufReader, Cursor};
use std::path::Path;

use super::FileService;
use crate::config::AppConfig;
use crate::errors::HWSystemError;
use crate::models::{ApiResponse, ErrorCode};
use crate::services::system::DynamicConfig;
use crate::services::{StorageProvider, error_response};

/// 获取缩略图存储目录
fn get_thumbnail_dir(upload_dir: &str) -> String {
    format!("{}/thumbnails", upload_dir)
}

/// 获取缩略图文件路径
fn get_thumbnail_path(upload_dir: &str, stored_name: &str) -> String {
    let thumb_dir = get_thumbnail_dir(upload_dir);
    // 缩略图统一使用 .jpg 扩展名
    let name_without_ext = Path::new(stored_name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(stored_name);
    format!("{}/{}.jpg", thumb_dir, name_without_ext)
}

/// 检查文件类型是否支持生成缩略图
fn is_thumbnail_supported(file_type: &str) -> bool {
    file_type.starts_with("image/")
        && !file_type.contains("svg") // SVG 不适合缩略图
}

/// 生成缩略图
fn generate_thumbnail(
    src_path: &Path,
    dst_path: &Path,
    max_width: u32,
    max_height: u32,
    quality: u8,
) -> Result<(), HWSystemError> {
    // 确保缩略图目录存在
    if let Some(parent) = dst_path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            HWSystemError::file_operation(format!("Failed to create thumbnail directory: {e}"))
        })?;
    }

    // 读取原图（使用 BufReader + with_guessed_format 从文件内容判断格式，而不是依赖扩展名）
    let file = fs::File::open(src_path)
        .map_err(|e| HWSystemError::file_operation(format!("Failed to open image: {e}")))?;
    let reader = BufReader::new(file);
    let img = ImageReader::new(reader)
        .with_guessed_format()
        .map_err(|e| HWSystemError::file_operation(format!("Failed to guess image format: {e}")))?
        .decode()
        .map_err(|e| HWSystemError::file_operation(format!("Failed to decode image: {e}")))?;

    // 计算缩略图尺寸（保持宽高比）
    let thumb = img.resize(max_width, max_height, FilterType::Lanczos3);

    // 编码为 JPEG
    let mut buffer = Vec::new();
    let mut cursor = Cursor::new(&mut buffer);

    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, quality);
    thumb
        .write_with_encoder(encoder)
        .map_err(|e| HWSystemError::file_operation(format!("Failed to encode thumbnail: {e}")))?;

    // 写入文件
    fs::write(dst_path, buffer)
        .map_err(|e| HWSystemError::file_operation(format!("Failed to write thumbnail: {e}")))?;

    Ok(())
}

pub async fn handle_thumbnail(
    service: &FileService,
    request: &HttpRequest,
    file_token: String,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request)?;

    // 1. 获取文件信息
    let db_file = match storage.get_file_by_token(&file_token).await {
        Ok(Some(f)) => f,
        Ok(None) => {
            return Ok(HttpResponse::NotFound().json(ApiResponse::error_empty(
                ErrorCode::FileNotFound,
                "File not found",
            )));
        }
        Err(e) => {
            return Ok(error_response(e));
        }
    };

    // 2. 检查是否支持缩略图
    if !is_thumbnail_supported(&db_file.file_type) {
        return Ok(HttpResponse::BadRequest().json(ApiResponse::error_empty(
            ErrorCode::BadRequest,
            "Thumbnail not supported for this file type",
        )));
    }

    let config = AppConfig::get();
    let upload_dir = &config.upload.dir;

    // 从数据库获取缩略图配置
    let max_width = DynamicConfig::upload_thumbnail_max_width().await;
    let max_height = DynamicConfig::upload_thumbnail_max_height().await;
    let quality = DynamicConfig::upload_thumbnail_quality().await;

    let original_path = format!("{}/{}", upload_dir, db_file.stored_name);
    let thumb_path = get_thumbnail_path(upload_dir, &db_file.stored_name);

    // 3. 检查原文件是否存在
    if !Path::new(&original_path).exists() {
        return Ok(HttpResponse::NotFound().json(ApiResponse::error_empty(
            ErrorCode::FileNotFound,
            "Original file not found",
        )));
    }

    // 4. 懒生成缩略图
    if !Path::new(&thumb_path).exists() {
        if let Err(e) = generate_thumbnail(
            Path::new(&original_path),
            Path::new(&thumb_path),
            max_width,
            max_height,
            quality,
        ) {
            tracing::error!("Failed to generate thumbnail: {:?}", e);
            return Ok(error_response(e));
        }
        tracing::info!("Generated thumbnail for file: {}", db_file.stored_name);
    }

    // 5. 返回缩略图
    match NamedFile::open(&thumb_path) {
        Ok(file) => {
            // 设置 Content-Type 为 JPEG
            let file = file
                .set_content_disposition(header::ContentDisposition {
                    disposition: header::DispositionType::Inline,
                    parameters: vec![],
                });

            // 构建响应并设置缓存头
            let mut response = file.into_response(request);
            response.headers_mut().insert(
                header::CACHE_CONTROL,
                header::HeaderValue::from_static("private, max-age=31536000, immutable"),
            );
            response.headers_mut().insert(
                header::CONTENT_TYPE,
                header::HeaderValue::from_static("image/jpeg"),
            );
            Ok(response)
        }
        Err(e) => {
            let hw_err = HWSystemError::file_operation(format!("{e:?}"));
            tracing::error!("{:?}", hw_err);
            Ok(error_response(hw_err))
        }
    }
}
