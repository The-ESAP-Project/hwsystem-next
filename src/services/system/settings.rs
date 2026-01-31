use std::sync::Arc;

use actix_web::{HttpRequest, HttpResponse, Result as ActixResult, web};

use super::{DynamicConfig, SystemService};
use crate::middlewares::RequireJWT;
use crate::models::{
    ApiResponse, ErrorCode,
    system::{
        requests::{SettingAuditQuery, UpdateSettingRequest},
        responses::{
            AdminSettingsListResponse, ClientConfigResponse, SettingResponse,
            SystemSettingsResponse,
        },
    },
};
use crate::storage::Storage;
use crate::utils::SafeSettingKey;

/// 获取公开系统设置（只读）
pub async fn get_settings(
    service: &SystemService,
    _req: &HttpRequest,
) -> ActixResult<HttpResponse> {
    // 获取配置
    let config = service.get_config();

    let response = SystemSettingsResponse {
        system_name: DynamicConfig::system_name().await,
        max_file_size: DynamicConfig::upload_max_size().await as u64,
        allowed_file_types: DynamicConfig::upload_allowed_types().await,
        environment: config.app.environment.clone(),
        log_level: config.app.log_level.clone(),
    };

    // 构建响应
    Ok(HttpResponse::Ok().json(ApiResponse::success(
        response,
        "Settings retrieved successfully",
    )))
}

/// 获取前端客户端配置
pub async fn get_client_config(
    service: &SystemService,
    _req: &HttpRequest,
) -> ActixResult<HttpResponse> {
    let config = service.get_config();

    let response = ClientConfigResponse {
        api_timeout: config.server.timeouts.client_request,
        file_operation_timeout: config.upload.timeout,
        max_file_size: DynamicConfig::upload_max_size().await as u64,
        allowed_file_types: DynamicConfig::upload_allowed_types().await,
    };

    Ok(HttpResponse::Ok().json(ApiResponse::success(
        response,
        "Client config retrieved successfully",
    )))
}

/// 获取所有管理员配置
pub async fn get_admin_settings(
    _req: HttpRequest,
    storage: web::Data<Arc<dyn Storage>>,
) -> ActixResult<HttpResponse> {
    let settings = match storage.list_all_settings().await {
        Ok(s) => s,
        Err(e) => {
            return Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error_empty(
                    ErrorCode::InternalServerError,
                    format!("获取配置列表失败: {e}"),
                )),
            );
        }
    };

    let response = AdminSettingsListResponse { settings };

    Ok(HttpResponse::Ok().json(ApiResponse::success(
        response,
        "Admin settings retrieved successfully",
    )))
}

/// 更新单个配置
pub async fn update_setting(
    req: HttpRequest,
    path: SafeSettingKey,
    body: web::Json<UpdateSettingRequest>,
    storage: web::Data<Arc<dyn Storage>>,
) -> ActixResult<HttpResponse> {
    let key = path.0;

    // 获取当前用户 ID
    let user_id = match RequireJWT::extract_user_id(&req) {
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

    // 获取客户端 IP
    let ip_address = req
        .connection_info()
        .realip_remote_addr()
        .map(|s| s.to_string());

    // 更新配置
    let setting = match storage
        .update_setting(&key, &body.value, user_id, ip_address)
        .await
    {
        Ok(s) => s,
        Err(e) => {
            return Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error_empty(
                    ErrorCode::InternalServerError,
                    format!("更新配置失败: {e}"),
                )),
            );
        }
    };

    // 更新缓存
    DynamicConfig::update(&key, &body.value).await;

    let response = SettingResponse { setting };

    Ok(HttpResponse::Ok().json(ApiResponse::success(
        response,
        "Setting updated successfully",
    )))
}

/// 获取审计日志
pub async fn get_setting_audits(
    _req: HttpRequest,
    query: web::Query<SettingAuditQuery>,
    storage: web::Data<Arc<dyn Storage>>,
) -> ActixResult<HttpResponse> {
    let audits = match storage.list_setting_audits(query.into_inner()).await {
        Ok(a) => a,
        Err(e) => {
            return Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error_empty(
                    ErrorCode::InternalServerError,
                    format!("获取审计日志失败: {e}"),
                )),
            );
        }
    };

    Ok(HttpResponse::Ok().json(ApiResponse::success(
        audits,
        "Setting audits retrieved successfully",
    )))
}
