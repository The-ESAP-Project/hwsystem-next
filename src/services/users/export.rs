//! 用户导出服务

use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};
use rust_xlsxwriter::{Format, Workbook};
use tracing::error;

use super::UserService;
use crate::models::users::requests::UserExportParams;
use crate::models::{ApiResponse, ErrorCode};

/// 导出用户列表
pub async fn export_users(
    service: &UserService,
    params: UserExportParams,
    request: &HttpRequest,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request)?;

    // 最多导出 10000 条
    let users = match storage
        .list_users_for_export_filtered(10000, params.role, params.status, params.search.as_deref())
        .await
    {
        Ok(users) => users,
        Err(e) => {
            error!("导出用户失败: {}", e);
            return Ok(
                HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                    ErrorCode::InternalServerError,
                    format!("导出用户失败: {e}"),
                )),
            );
        }
    };

    match params.format.as_str() {
        "xlsx" => export_xlsx(&users),
        _ => export_csv(&users),
    }
}

/// 下载导入模板
pub async fn download_template(format: &str) -> ActixResult<HttpResponse> {
    match format {
        "xlsx" => generate_template_xlsx(),
        _ => generate_template_csv(),
    }
}

fn export_csv(users: &[crate::models::users::entities::User]) -> ActixResult<HttpResponse> {
    let mut wtr = csv::Writer::from_writer(vec![]);

    // 写入表头
    if let Err(e) = wtr.write_record([
        "id",
        "username",
        "email",
        "role",
        "status",
        "display_name",
        "created_at",
    ]) {
        error!("CSV 写入失败: {}", e);
        return Ok(
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error_empty(
                ErrorCode::ExportFailed,
                format!("CSV 写入失败: {e}"),
            )),
        );
    }

    // 写入数据
    for user in users {
        if let Err(e) = wtr.write_record([
            user.id.to_string(),
            user.username.clone(),
            user.email.clone(),
            user.role.to_string(),
            user.status.to_string(),
            user.display_name.clone().unwrap_or_default(),
            user.created_at.to_rfc3339(),
        ]) {
            error!("CSV 写入失败: {}", e);
            return Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error_empty(
                    ErrorCode::ExportFailed,
                    format!("CSV 写入失败: {e}"),
                )),
            );
        }
    }

    let data = match wtr.into_inner() {
        Ok(d) => d,
        Err(e) => {
            error!("CSV 生成失败: {}", e);
            return Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error_empty(
                    ErrorCode::ExportFailed,
                    format!("CSV 生成失败: {e}"),
                )),
            );
        }
    };

    Ok(HttpResponse::Ok()
        .content_type("text/csv; charset=utf-8")
        .insert_header(("Content-Disposition", "attachment; filename=\"users.csv\""))
        .body(data))
}

fn export_xlsx(users: &[crate::models::users::entities::User]) -> ActixResult<HttpResponse> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    // 表头格式
    let header_format = Format::new().set_bold();

    // 写入表头
    let headers = [
        "ID",
        "用户名",
        "邮箱",
        "角色",
        "状态",
        "显示名称",
        "创建时间",
    ];
    for (col, header) in headers.iter().enumerate() {
        if let Err(e) = worksheet.write_string_with_format(0, col as u16, *header, &header_format) {
            error!("XLSX 写入失败: {}", e);
            return Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error_empty(
                    ErrorCode::ExportFailed,
                    format!("XLSX 写入失败: {e}"),
                )),
            );
        }
    }

    // 写入数据
    for (row, user) in users.iter().enumerate() {
        let row = (row + 1) as u32;
        worksheet.write_number(row, 0, user.id as f64).ok();
        worksheet.write_string(row, 1, &user.username).ok();
        worksheet.write_string(row, 2, &user.email).ok();
        worksheet.write_string(row, 3, user.role.to_string()).ok();
        worksheet.write_string(row, 4, user.status.to_string()).ok();
        worksheet
            .write_string(row, 5, user.display_name.as_deref().unwrap_or(""))
            .ok();
        worksheet
            .write_string(row, 6, user.created_at.to_rfc3339())
            .ok();
    }

    // 生成二进制数据
    let buffer = match workbook.save_to_buffer() {
        Ok(b) => b,
        Err(e) => {
            error!("XLSX 生成失败: {}", e);
            return Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error_empty(
                    ErrorCode::ExportFailed,
                    format!("XLSX 生成失败: {e}"),
                )),
            );
        }
    };

    Ok(HttpResponse::Ok()
        .content_type("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
        .insert_header(("Content-Disposition", "attachment; filename=\"users.xlsx\""))
        .body(buffer))
}

fn generate_template_csv() -> ActixResult<HttpResponse> {
    let mut wtr = csv::Writer::from_writer(vec![]);

    // 写入表头
    if let Err(e) = wtr.write_record(["username", "email", "password", "role", "display_name"]) {
        error!("CSV 写入失败: {}", e);
        return Ok(
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error_empty(
                ErrorCode::ExportFailed,
                format!("CSV 写入失败: {e}"),
            )),
        );
    }

    // 写入示例行
    if let Err(e) = wtr.write_record([
        "example_user",
        "user@example.com",
        "password123",
        "user",
        "示例用户",
    ]) {
        error!("CSV 写入失败: {}", e);
        return Ok(
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error_empty(
                ErrorCode::ExportFailed,
                format!("CSV 写入失败: {e}"),
            )),
        );
    }

    let data = match wtr.into_inner() {
        Ok(d) => d,
        Err(e) => {
            error!("CSV 生成失败: {}", e);
            return Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error_empty(
                    ErrorCode::ExportFailed,
                    format!("CSV 生成失败: {e}"),
                )),
            );
        }
    };

    Ok(HttpResponse::Ok()
        .content_type("text/csv; charset=utf-8")
        .insert_header((
            "Content-Disposition",
            "attachment; filename=\"user_import_template.csv\"",
        ))
        .body(data))
}

fn generate_template_xlsx() -> ActixResult<HttpResponse> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    // 表头格式
    let header_format = Format::new().set_bold();

    // 写入表头
    let headers = ["username", "email", "password", "role", "display_name"];
    for (col, header) in headers.iter().enumerate() {
        if let Err(e) = worksheet.write_string_with_format(0, col as u16, *header, &header_format) {
            error!("XLSX 写入失败: {}", e);
            return Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error_empty(
                    ErrorCode::ExportFailed,
                    format!("XLSX 写入失败: {e}"),
                )),
            );
        }
    }

    // 写入示例行
    worksheet.write_string(1, 0, "example_user").ok();
    worksheet.write_string(1, 1, "user@example.com").ok();
    worksheet.write_string(1, 2, "password123").ok();
    worksheet.write_string(1, 3, "user").ok();
    worksheet.write_string(1, 4, "示例用户").ok();

    // 生成二进制数据
    let buffer = match workbook.save_to_buffer() {
        Ok(b) => b,
        Err(e) => {
            error!("XLSX 生成失败: {}", e);
            return Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error_empty(
                    ErrorCode::ExportFailed,
                    format!("XLSX 生成失败: {e}"),
                )),
            );
        }
    };

    Ok(HttpResponse::Ok()
        .content_type("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
        .insert_header((
            "Content-Disposition",
            "attachment; filename=\"user_import_template.xlsx\"",
        ))
        .body(buffer))
}
