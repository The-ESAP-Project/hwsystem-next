use crate::cache::traits::ObjectCache;
use crate::errors::Result;
use parking_lot::Mutex;
use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    sync::{Arc, OnceLock},
};

pub type BoxedObjectCacheFuture =
    Pin<Box<dyn Future<Output = Result<Box<dyn ObjectCache>>> + Send>>;
pub type ObjectCacheConstructor = Arc<dyn Fn() -> BoxedObjectCacheFuture + Send + Sync>;

/// 临时收集器：在 main() 前收集通过 ctor 注册的插件
static PENDING_PLUGINS: Mutex<Vec<(String, ObjectCacheConstructor)>> = Mutex::new(Vec::new());

/// 最终注册表：启动后只读
static OBJECT_CACHE_REGISTRY: OnceLock<HashMap<String, ObjectCacheConstructor>> = OnceLock::new();

/// 注册缓存插件（启动前调用，通过 ctor 宏）
pub fn register_object_cache_plugin<S: Into<String>>(name: S, constructor: ObjectCacheConstructor) {
    PENDING_PLUGINS.lock().push((name.into(), constructor));
}

/// 冻结注册表（在 main() 启动时调用一次）
pub fn finalize_cache_registry() {
    let plugins = std::mem::take(&mut *PENDING_PLUGINS.lock());
    let registry: HashMap<_, _> = plugins.into_iter().collect();
    if OBJECT_CACHE_REGISTRY.set(registry).is_err() {
        tracing::warn!("Cache registry already finalized, ignoring duplicate call");
    }
}

/// 获取缓存插件构造器
pub fn get_object_cache_plugin(name: &str) -> Option<ObjectCacheConstructor> {
    OBJECT_CACHE_REGISTRY.get()?.get(name).cloned()
}

pub fn debug_object_cache_registry() {
    if let Some(registry) = OBJECT_CACHE_REGISTRY.get() {
        if registry.is_empty() {
            tracing::debug!("No object cache plugins registered.");
        } else {
            tracing::debug!("Registered object cache plugins:");
            for key in registry.keys() {
                tracing::debug!(" - {}", key);
            }
        }
    } else {
        tracing::debug!("Cache registry not yet finalized.");
    }
}
