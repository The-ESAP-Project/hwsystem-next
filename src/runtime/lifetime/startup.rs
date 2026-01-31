use crate::cache::{
    ObjectCache,
    register::{finalize_cache_registry, get_object_cache_plugin},
};
use crate::config::AppConfig;
use crate::models::users::entities::UserRole;
use crate::models::users::requests::CreateUserRequest;
use crate::services::system::DynamicConfig;
use crate::storage::Storage;
use crate::utils::password::hash_password;
use std::sync::Arc;
use tracing::{debug, info, warn};

pub struct StartupContext {
    pub storage: Arc<dyn Storage>,
    pub cache: Arc<dyn ObjectCache>,
}

/// 创建缓存实例
async fn create_cache() -> Result<Arc<dyn ObjectCache>, Box<dyn std::error::Error>> {
    let config = AppConfig::get();
    let cache_type = &config.cache.cache_type;

    warn!("Attempting to create {} cache backend", cache_type);

    // 根据配置选择缓存后端
    if let Some(constructor) = get_object_cache_plugin(cache_type) {
        match constructor().await {
            Ok(cache) => {
                warn!("Successfully created {} cache backend", cache_type);
                return Ok(Arc::from(cache));
            }
            Err(e) => {
                warn!("Failed to create {} cache: {}", cache_type, e);

                // 如果配置的缓存失败，尝试回退策略
                if cache_type == "redis" {
                    warn!("Falling back to memory cache");
                    if let Some(fallback_constructor) = get_object_cache_plugin("moka") {
                        match fallback_constructor().await {
                            Ok(cache) => {
                                warn!(
                                    "Successfully created fallback Moka (in-memory) cache backend"
                                );
                                return Ok(Arc::from(cache));
                            }
                            Err(fallback_e) => {
                                warn!("Failed to create fallback Moka cache: {}", fallback_e);
                            }
                        }
                    }
                }
            }
        }
    } else {
        warn!("Cache backend '{}' not found in registry", cache_type);

        // 如果找不到配置的缓存类型，尝试默认的内存缓存
        if cache_type != "moka" {
            warn!("Falling back to default memory cache");
            if let Some(fallback_constructor) = get_object_cache_plugin("moka") {
                match fallback_constructor().await {
                    Ok(cache) => {
                        warn!("Successfully created fallback Moka (in-memory) cache backend");
                        return Ok(Arc::from(cache));
                    }
                    Err(fallback_e) => {
                        warn!("Failed to create fallback Moka cache: {}", fallback_e);
                    }
                }
            }
        }
    }

    Err(format!("No cache backend available (tried: {cache_type})").into())
}

/// 生成随机密码
fn generate_random_password(length: usize) -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%";
    let mut rng = rand::rng();
    (0..length)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// 初始化动态配置缓存
/// 从数据库加载配置并初始化全局缓存
async fn init_dynamic_config(storage: &Arc<dyn Storage>) {
    match storage.list_all_settings().await {
        Ok(settings) => {
            let settings_vec: Vec<(String, String)> =
                settings.into_iter().map(|s| (s.key, s.value)).collect();
            DynamicConfig::init(settings_vec).await;
        }
        Err(e) => {
            warn!(
                "Failed to load dynamic config from database: {}, using defaults",
                e
            );
            // 使用空配置初始化，DynamicConfig 会回退到 AppConfig
            DynamicConfig::init(vec![]).await;
        }
    }
}

/// 初始化默认管理员账号
/// 如果数据库中没有任何用户，则创建一个默认的 admin 账号
async fn seed_admin(storage: &Arc<dyn Storage>) {
    // 检查是否已有用户
    match storage.count_users().await {
        Ok(count) if count > 0 => {
            debug!(
                "Database already has {} user(s), skipping admin seed",
                count
            );
            return;
        }
        Ok(_) => {
            info!("No users found in database, creating default admin account...");
        }
        Err(e) => {
            warn!("Failed to count users: {}, skipping admin seed", e);
            return;
        }
    }

    // 获取密码：优先从环境变量，否则生成随机密码
    let password = std::env::var("ADMIN_PASSWORD").unwrap_or_else(|_| {
        let pwd = generate_random_password(16);
        warn!("==========================================================");
        warn!("  ADMIN PASSWORD NOT SET - USING GENERATED PASSWORD");
        warn!("  Generated admin password: {}", pwd);
        warn!("  Please save this password or set ADMIN_PASSWORD env var");
        warn!("==========================================================");
        pwd
    });

    // 哈希密码
    let password_hash = match hash_password(&password) {
        Ok(hash) => hash,
        Err(e) => {
            warn!("Failed to hash admin password: {}, skipping admin seed", e);
            return;
        }
    };

    // 创建管理员账号
    let admin_request = CreateUserRequest {
        username: "admin".to_string(),
        email: "admin@localhost".to_string(),
        password: password_hash,
        role: UserRole::Admin,
        display_name: Some("Administrator".to_string()),
        avatar_url: None,
    };

    match storage.create_user(admin_request).await {
        Ok(user) => {
            info!(
                "Default admin account created successfully (ID: {}, username: {})",
                user.id, user.username
            );
        }
        Err(e) => {
            warn!("Failed to create admin account: {}", e);
        }
    }
}

/// 准备服务器启动的上下文
/// 包括存储、缓存和路由配置等
pub async fn prepare_server_startup() -> StartupContext {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    // 冻结缓存注册表（在 main() 前通过 ctor 注册的插件）
    finalize_cache_registry();

    if cfg!(debug_assertions) {
        crate::cache::register::debug_object_cache_registry();
        debug!("Debug mode: Cache registry is enabled");
    }

    let storage = crate::storage::create_storage()
        .await
        .expect("Failed to create storage backend");
    warn!("Storage backend initialized and migrations completed");

    // 初始化动态配置缓存
    init_dynamic_config(&storage).await;

    // 初始化默认管理员账号（如果需要）
    seed_admin(&storage).await;

    // 创建缓存实例
    let cache = create_cache().await.expect("Failed to create cache");
    warn!("Cache backend initialized");

    StartupContext { storage, cache }
}
