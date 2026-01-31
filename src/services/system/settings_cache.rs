//! 动态配置缓存
//!
//! 提供从数据库加载的动态配置的全局缓存访问。
//! 使用 RwLock 保护，支持热更新。

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
use tokio::sync::RwLock;

use crate::config::AppConfig;

/// 动态配置缓存
static DYNAMIC_CONFIG: OnceLock<RwLock<DynamicConfigCache>> = OnceLock::new();

/// 动态配置缓存内部结构
#[derive(Debug, Default)]
struct DynamicConfigCache {
    /// 使用 Arc<str> 避免每次读取时 clone 整个 String
    settings: HashMap<String, Arc<str>>,
    initialized: bool,
}

/// 动态配置访问接口
pub struct DynamicConfig;

impl DynamicConfig {
    /// 初始化动态配置缓存
    /// 在应用启动时调用，从数据库加载配置
    pub async fn init(settings: Vec<(String, String)>) {
        let cache = DYNAMIC_CONFIG.get_or_init(|| RwLock::new(DynamicConfigCache::default()));

        let mut guard = cache.write().await;
        guard.settings.clear();
        for (key, value) in settings {
            guard.settings.insert(key, Arc::from(value));
        }
        guard.initialized = true;

        tracing::info!(
            "动态配置缓存初始化完成，加载了 {} 个配置项",
            guard.settings.len()
        );
    }

    /// 更新单个配置项
    pub async fn update(key: &str, value: &str) {
        if let Some(cache) = DYNAMIC_CONFIG.get() {
            let mut guard = cache.write().await;
            guard.settings.insert(key.to_string(), Arc::from(value));
            tracing::debug!("动态配置更新: {} = {}", key, value);
        }
    }

    /// 获取字符串配置（返回 Arc<str> 避免复制）
    async fn get_string(key: &str) -> Option<Arc<str>> {
        if let Some(cache) = DYNAMIC_CONFIG.get() {
            let guard = cache.read().await;
            return guard.settings.get(key).cloned();
        }
        None
    }

    /// 获取整数配置
    async fn get_i64(key: &str) -> Option<i64> {
        Self::get_string(key).await.and_then(|v| v.parse().ok())
    }

    /// 获取 JSON 数组配置
    async fn get_json_array(key: &str) -> Option<Vec<String>> {
        Self::get_string(key)
            .await
            .and_then(|v| serde_json::from_str(&v).ok())
    }

    // ============================================
    // 具体配置项访问方法
    // ============================================

    /// 获取系统名称
    pub async fn system_name() -> String {
        Self::get_string("app.system_name")
            .await
            .map(|s| s.to_string())
            .unwrap_or_else(|| AppConfig::get().app.system_name.clone())
    }

    /// 获取 Access Token 有效期（分钟）
    pub async fn access_token_expiry() -> i64 {
        Self::get_i64("jwt.access_token_expiry")
            .await
            .unwrap_or_else(|| AppConfig::get().jwt.access_token_expiry)
    }

    /// 获取 Refresh Token 有效期（天）
    pub async fn refresh_token_expiry() -> i64 {
        Self::get_i64("jwt.refresh_token_expiry")
            .await
            .unwrap_or_else(|| AppConfig::get().jwt.refresh_token_expiry)
    }

    /// 获取记住我 Refresh Token 有效期（天）
    pub async fn refresh_token_remember_me_expiry() -> i64 {
        Self::get_i64("jwt.refresh_token_remember_me_expiry")
            .await
            .unwrap_or_else(|| AppConfig::get().jwt.refresh_token_remember_me_expiry)
    }

    /// 获取上传文件大小限制（字节）
    pub async fn upload_max_size() -> usize {
        Self::get_i64("upload.max_size")
            .await
            .map(|v| v as usize)
            .unwrap_or_else(|| AppConfig::get().upload.max_size)
    }

    /// 获取允许上传的文件类型
    pub async fn upload_allowed_types() -> Vec<String> {
        Self::get_json_array("upload.allowed_types")
            .await
            .unwrap_or_else(|| {
                // config.toml 中的 allowed_types 已废弃
                // 如果数据库未配置，返回空数组（表示无限制）
                AppConfig::get()
                    .upload
                    .allowed_types
                    .clone()
                    .unwrap_or_default()
            })
    }

    /// 获取允许的跨域来源
    pub async fn cors_allowed_origins() -> Vec<String> {
        Self::get_json_array("cors.allowed_origins")
            .await
            .unwrap_or_else(|| AppConfig::get().cors.allowed_origins.clone())
    }

    /// 获取 CORS 预检请求缓存时间（秒）
    pub async fn cors_max_age() -> usize {
        Self::get_i64("cors.max_age")
            .await
            .map(|v| v as usize)
            .unwrap_or_else(|| AppConfig::get().cors.max_age)
    }

    /// 检查缓存是否已初始化
    pub async fn is_initialized() -> bool {
        if let Some(cache) = DYNAMIC_CONFIG.get() {
            let guard = cache.read().await;
            return guard.initialized;
        }
        false
    }
}
