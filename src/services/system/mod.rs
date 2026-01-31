pub mod settings;
pub mod settings_cache;

pub use settings_cache::DynamicConfig;

use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};

use crate::config::AppConfig;

pub struct SystemService;

impl SystemService {
    pub fn new_lazy() -> Self {
        Self
    }

    pub(crate) fn get_config(&self) -> &AppConfig {
        AppConfig::get()
    }

    // Handle get settings
    pub async fn get_settings(&self, request: &HttpRequest) -> ActixResult<HttpResponse> {
        settings::get_settings(self, request).await
    }

    // Handle get client config
    pub async fn get_client_config(&self, request: &HttpRequest) -> ActixResult<HttpResponse> {
        settings::get_client_config(self, request).await
    }
}
