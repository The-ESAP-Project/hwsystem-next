use async_trait::async_trait;
use moka::future::Cache;
use moka::Expiry;
use std::time::{Duration, Instant};
use tracing::debug;

use crate::cache::{CacheResult, ObjectCache};
use crate::config::AppConfig;
use crate::declare_object_cache_plugin;

declare_object_cache_plugin!("moka", MokaCacheWrapper);

/// 缓存值包装器，携带过期时间
#[derive(Clone)]
struct CacheValue {
    data: String,
    ttl_secs: u64,
}

/// 自定义过期策略，支持按项 TTL
struct TtlExpiry {
    default_ttl: Duration,
}

impl Expiry<String, CacheValue> for TtlExpiry {
    fn expire_after_create(
        &self,
        _key: &String,
        value: &CacheValue,
        _current_time: Instant,
    ) -> Option<Duration> {
        if value.ttl_secs > 0 {
            Some(Duration::from_secs(value.ttl_secs))
        } else {
            // ttl_secs == 0 表示使用默认 TTL
            Some(self.default_ttl)
        }
    }

    fn expire_after_read(
        &self,
        _key: &String,
        _value: &CacheValue,
        _current_time: Instant,
        _current_duration: Option<Duration>,
        _last_modified_at: Instant,
    ) -> Option<Duration> {
        // 读取时不更新过期时间
        None
    }

    fn expire_after_update(
        &self,
        _key: &String,
        value: &CacheValue,
        _current_time: Instant,
        _current_duration: Option<Duration>,
    ) -> Option<Duration> {
        // 更新时使用新值的 TTL
        if value.ttl_secs > 0 {
            Some(Duration::from_secs(value.ttl_secs))
        } else {
            Some(self.default_ttl)
        }
    }
}

pub struct MokaCacheWrapper {
    inner: Cache<String, CacheValue>,
}

impl Default for MokaCacheWrapper {
    fn default() -> Self {
        Self::new().expect("MokaCacheWrapper 初始化失败，请检查配置")
    }
}

impl MokaCacheWrapper {
    pub fn new() -> Result<Self, String> {
        let config = AppConfig::get();
        let default_ttl = Duration::from_secs(config.cache.default_ttl);

        let inner = Cache::builder()
            .max_capacity(config.cache.memory.max_capacity)
            .expire_after(TtlExpiry { default_ttl })
            .build();

        debug!(
            "MokaCacheWrapper initialized with max capacity: {}, default TTL: {}s",
            config.cache.memory.max_capacity, config.cache.default_ttl
        );
        Ok(Self { inner })
    }
}

#[async_trait]
impl ObjectCache for MokaCacheWrapper {
    async fn get_raw(&self, key: &str) -> CacheResult<String> {
        if let Some(value) = self.inner.get(key).await {
            debug!("Cache hit: {}", key);
            CacheResult::Found(value.data)
        } else {
            debug!("Cache miss: {}", key);
            CacheResult::NotFound
        }
    }

    async fn insert_raw(&self, key: String, value: String, ttl: u64) {
        debug!("Cache insert: {} (TTL: {}s)", key, if ttl == 0 { "default".to_string() } else { ttl.to_string() });
        self.inner
            .insert(key, CacheValue { data: value, ttl_secs: ttl })
            .await;
    }

    async fn remove(&self, key: &str) {
        debug!("Cache remove: {}", key);
        self.inner.invalidate(key).await;
    }

    async fn invalidate_all(&self) {
        debug!("Cache invalidate all");
        self.inner.invalidate_all();
    }
}
