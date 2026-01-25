//! 用户导入服务

use actix_multipart::Multipart;
use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};
use calamine::{Reader, Xlsx};
use futures_util::StreamExt;
use std::collections::HashSet;
use std::io::Cursor;
use tracing::error;

use super::UserService;
use crate::models::users::entities::UserRole;
use crate::models::users::requests::CreateUserRequest;
use crate::models::users::responses::{ImportRowError, UserImportResponse};
use crate::models::{ApiResponse, ErrorCode};
use crate::utils::password::hash_password;
use crate::utils::validate::{validate_email, validate_password_simple, validate_username};

/// 导入解析错误
enum ImportParseError {
    MissingColumn(String),
    ParseFailed(String),
    EmptyFile,
}

impl ImportParseError {
    fn error_code(&self) -> ErrorCode {
        match self {
            Self::MissingColumn(_) => ErrorCode::ImportFileMissingColumn,
            Self::ParseFailed(_) => ErrorCode::ImportFileParseFailed,
            Self::EmptyFile => ErrorCode::ImportFileDataInvalid,
        }
    }

    fn message(&self) -> String {
        match self {
            Self::MissingColumn(col) => format!("缺少必需列: {col}"),
            Self::ParseFailed(msg) => msg.clone(),
            Self::EmptyFile => "文件中没有数据".to_string(),
        }
    }
}

/// 导入行数据
#[derive(Debug, Clone)]
struct ImportRow {
    row_num: usize,
    username: String,
    email: String,
    password: String,
    role: String,
    display_name: Option<String>,
}

/// 导入用户
pub async fn import_users(
    service: &UserService,
    mut payload: Multipart,
    request: &HttpRequest,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request);

    // 读取文件内容
    let (file_bytes, file_name) = match read_file_from_multipart(&mut payload).await {
        Ok(result) => result,
        Err(e) => {
            return Ok(HttpResponse::BadRequest().json(ApiResponse::error_empty(
                ErrorCode::FileUploadFailed,
                format!("文件读取失败: {e}"),
            )));
        }
    };

    if file_bytes.is_empty() {
        return Ok(HttpResponse::BadRequest().json(ApiResponse::error_empty(
            ErrorCode::FileUploadFailed,
            "文件内容为空",
        )));
    }

    // 根据文件扩展名解析
    let rows = if file_name.ends_with(".xlsx") {
        match parse_xlsx(&file_bytes) {
            Ok(rows) => rows,
            Err(e) => {
                return Ok(HttpResponse::BadRequest()
                    .json(ApiResponse::error_empty(e.error_code(), e.message())));
            }
        }
    } else {
        match parse_csv(&file_bytes) {
            Ok(rows) => rows,
            Err(e) => {
                return Ok(HttpResponse::BadRequest()
                    .json(ApiResponse::error_empty(e.error_code(), e.message())));
            }
        }
    };

    if rows.is_empty() {
        return Ok(HttpResponse::BadRequest().json(ApiResponse::error_empty(
            ErrorCode::ImportFileDataInvalid,
            "文件中没有数据行",
        )));
    }

    if rows.len() > 1000 {
        return Ok(HttpResponse::BadRequest().json(ApiResponse::error_empty(
            ErrorCode::ImportFileDataInvalid,
            "单次导入最多支持 1000 行",
        )));
    }

    // 验证并过滤数据
    let mut errors: Vec<ImportRowError> = Vec::new();
    let mut valid_rows: Vec<ImportRow> = Vec::new();

    for row in &rows {
        let mut row_errors = validate_row(row);
        if row_errors.is_empty() {
            valid_rows.push(row.clone());
        } else {
            errors.append(&mut row_errors);
        }
    }

    // 批量检查用户名和邮箱冲突
    let usernames: Vec<String> = valid_rows.iter().map(|r| r.username.clone()).collect();
    let emails: Vec<String> = valid_rows.iter().map(|r| r.email.clone()).collect();

    let existing_usernames = storage
        .check_usernames_exist(&usernames)
        .await
        .unwrap_or_default();
    let existing_emails = storage
        .check_emails_exist(&emails)
        .await
        .unwrap_or_default();

    let existing_usernames_set: HashSet<_> = existing_usernames.into_iter().collect();
    let existing_emails_set: HashSet<_> = existing_emails.into_iter().collect();

    // 过滤冲突行
    let mut skipped = 0;
    let mut to_create: Vec<ImportRow> = Vec::new();

    for row in valid_rows {
        if existing_usernames_set.contains(&row.username) {
            skipped += 1;
            errors.push(ImportRowError {
                row: row.row_num,
                field: "username".to_string(),
                message: "用户名已存在".to_string(),
            });
        } else if existing_emails_set.contains(&row.email) {
            skipped += 1;
            errors.push(ImportRowError {
                row: row.row_num,
                field: "email".to_string(),
                message: "邮箱已存在".to_string(),
            });
        } else {
            to_create.push(row);
        }
    }

    // 批量创建用户
    let mut success = 0;
    let mut failed = 0;

    for row in to_create {
        // 哈希密码（使用 spawn_blocking 避免阻塞）
        let password_clone = row.password.clone();
        let hashed = match tokio::task::spawn_blocking(move || hash_password(&password_clone)).await
        {
            Ok(Ok(hash)) => hash,
            Ok(Err(e)) => {
                failed += 1;
                errors.push(ImportRowError {
                    row: row.row_num,
                    field: "password".to_string(),
                    message: format!("密码哈希失败: {e}"),
                });
                continue;
            }
            Err(e) => {
                failed += 1;
                errors.push(ImportRowError {
                    row: row.row_num,
                    field: "password".to_string(),
                    message: format!("密码处理失败: {e}"),
                });
                continue;
            }
        };

        let role = match row.role.parse::<UserRole>() {
            Ok(r) => r,
            Err(_) => {
                failed += 1;
                errors.push(ImportRowError {
                    row: row.row_num,
                    field: "role".to_string(),
                    message: format!("无效的角色: {}", row.role),
                });
                continue;
            }
        };

        let create_req = CreateUserRequest {
            username: row.username,
            email: row.email,
            password: hashed,
            role,
            display_name: row.display_name,
            avatar_url: None,
        };

        match storage.create_user(create_req).await {
            Ok(_) => success += 1,
            Err(e) => {
                failed += 1;
                error!("创建用户失败: {}", e);
                errors.push(ImportRowError {
                    row: row.row_num,
                    field: "".to_string(),
                    message: format!("创建失败: {e}"),
                });
            }
        }
    }

    let response = UserImportResponse {
        total: rows.len(),
        success,
        skipped,
        failed,
        errors,
    };

    Ok(HttpResponse::Ok().json(ApiResponse::success(response, "导入完成")))
}

async fn read_file_from_multipart(payload: &mut Multipart) -> Result<(Vec<u8>, String), String> {
    let mut file_bytes = Vec::new();
    let mut file_name = String::new();

    while let Some(item) = payload.next().await {
        let mut field = item.map_err(|e| format!("读取字段失败: {e}"))?;

        if field.name().map(|n| n == "file").unwrap_or(false) {
            // 获取文件名
            if let Some(content_disposition) = field.content_disposition() {
                file_name = content_disposition
                    .get_filename()
                    .unwrap_or("upload.csv")
                    .to_string();
            }

            // 读取内容
            while let Some(chunk) = field.next().await {
                let data = chunk.map_err(|e| format!("读取数据失败: {e}"))?;
                file_bytes.extend_from_slice(&data);
            }
        }
    }

    if file_bytes.is_empty() {
        return Err("未找到文件字段".to_string());
    }

    Ok((file_bytes, file_name))
}

fn parse_csv(data: &[u8]) -> Result<Vec<ImportRow>, ImportParseError> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(Cursor::new(data));

    // 检查表头
    let headers = rdr
        .headers()
        .map_err(|e| ImportParseError::ParseFailed(format!("读取表头失败: {e}")))?;
    let header_map: std::collections::HashMap<_, _> = headers
        .iter()
        .enumerate()
        .map(|(i, h)| (h.to_lowercase(), i))
        .collect();

    // 必需列
    let username_idx = header_map
        .get("username")
        .ok_or_else(|| ImportParseError::MissingColumn("username".to_string()))?;
    let email_idx = header_map
        .get("email")
        .ok_or_else(|| ImportParseError::MissingColumn("email".to_string()))?;
    let password_idx = header_map
        .get("password")
        .ok_or_else(|| ImportParseError::MissingColumn("password".to_string()))?;
    let role_idx = header_map
        .get("role")
        .ok_or_else(|| ImportParseError::MissingColumn("role".to_string()))?;
    let display_name_idx = header_map.get("display_name");

    let mut rows = Vec::new();

    for (row_num, result) in rdr.records().enumerate() {
        let record = result.map_err(|e| {
            ImportParseError::ParseFailed(format!("第 {} 行解析失败: {e}", row_num + 2))
        })?;

        let username = record.get(*username_idx).unwrap_or("").trim().to_string();
        let email = record.get(*email_idx).unwrap_or("").trim().to_string();
        let password = record.get(*password_idx).unwrap_or("").trim().to_string();
        let role = record.get(*role_idx).unwrap_or("").trim().to_string();
        let display_name = display_name_idx
            .and_then(|i| record.get(*i))
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        rows.push(ImportRow {
            row_num: row_num + 2, // 1-based, skip header
            username,
            email,
            password,
            role,
            display_name,
        });
    }

    Ok(rows)
}

fn parse_xlsx(data: &[u8]) -> Result<Vec<ImportRow>, ImportParseError> {
    let cursor = Cursor::new(data);
    let mut workbook: Xlsx<_> = Xlsx::new(cursor)
        .map_err(|e| ImportParseError::ParseFailed(format!("打开 XLSX 失败: {e}")))?;

    // 获取第一个工作表
    let sheet_names = workbook.sheet_names().to_vec();
    let sheet_name = sheet_names
        .first()
        .ok_or_else(|| ImportParseError::ParseFailed("工作簿中没有工作表".to_string()))?;

    let range = workbook
        .worksheet_range(sheet_name)
        .map_err(|e| ImportParseError::ParseFailed(format!("读取工作表失败: {e}")))?;

    let mut rows_iter = range.rows();

    // 读取表头
    let header_row = rows_iter
        .next()
        .ok_or(ImportParseError::EmptyFile)?;
    let header_map: std::collections::HashMap<_, _> = header_row
        .iter()
        .enumerate()
        .map(|(i, cell)| (cell.to_string().to_lowercase(), i))
        .collect();

    // 必需列
    let username_idx = *header_map
        .get("username")
        .ok_or_else(|| ImportParseError::MissingColumn("username".to_string()))?;
    let email_idx = *header_map
        .get("email")
        .ok_or_else(|| ImportParseError::MissingColumn("email".to_string()))?;
    let password_idx = *header_map
        .get("password")
        .ok_or_else(|| ImportParseError::MissingColumn("password".to_string()))?;
    let role_idx = *header_map
        .get("role")
        .ok_or_else(|| ImportParseError::MissingColumn("role".to_string()))?;
    let display_name_idx = header_map.get("display_name").copied();

    let mut rows = Vec::new();

    for (row_num, row) in rows_iter.enumerate() {
        let get_cell = |idx: usize| -> String {
            row.get(idx)
                .map(|c| c.to_string().trim().to_string())
                .unwrap_or_default()
        };

        let username = get_cell(username_idx);
        let email = get_cell(email_idx);
        let password = get_cell(password_idx);
        let role = get_cell(role_idx);
        let display_name = display_name_idx.map(get_cell).filter(|s| !s.is_empty());

        rows.push(ImportRow {
            row_num: row_num + 2, // 1-based, skip header
            username,
            email,
            password,
            role,
            display_name,
        });
    }

    Ok(rows)
}

fn validate_row(row: &ImportRow) -> Vec<ImportRowError> {
    let mut errors = Vec::new();

    // 验证用户名
    if let Err(msg) = validate_username(&row.username) {
        errors.push(ImportRowError {
            row: row.row_num,
            field: "username".to_string(),
            message: msg.to_string(),
        });
    }

    // 验证邮箱
    if let Err(msg) = validate_email(&row.email) {
        errors.push(ImportRowError {
            row: row.row_num,
            field: "email".to_string(),
            message: msg.to_string(),
        });
    }

    // 验证密码
    if let Err(msg) = validate_password_simple(&row.password) {
        errors.push(ImportRowError {
            row: row.row_num,
            field: "password".to_string(),
            message: msg,
        });
    }

    // 验证角色
    if row.role.parse::<UserRole>().is_err() {
        errors.push(ImportRowError {
            row: row.row_num,
            field: "role".to_string(),
            message: format!("无效的角色值: {}，支持: user, teacher, admin", row.role),
        });
    }

    errors
}
