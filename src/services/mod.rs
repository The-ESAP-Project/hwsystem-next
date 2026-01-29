pub mod auth;
pub mod class_users;
pub mod classes;
pub mod files;
pub mod grades;
pub mod homeworks;
pub mod notifications;
pub mod submissions;
pub mod system;
pub mod users;
pub mod websocket;

// Storage provider trait for all services
use crate::models::ApiResponse;
use crate::models::ErrorCode;
use crate::storage::Storage;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use std::sync::Arc;

/// Trait for services that need to access storage.
/// This centralizes the logic for obtaining Storage from the request context
/// or from an internal field.
pub trait StorageProvider {
    /// Get the Storage instance from the request or internal field.
    fn get_storage(&self, request: &HttpRequest) -> Result<Arc<dyn Storage>, actix_web::Error> {
        if let Some(storage) = self.storage_ref() {
            Ok(storage.clone())
        } else {
            request
                .app_data::<actix_web::web::Data<Arc<dyn Storage>>>()
                .map(|data| data.get_ref().clone())
                .ok_or_else(|| {
                    actix_web::error::InternalError::from_response(
                        "Storage service unavailable",
                        HttpResponse::InternalServerError().json(ApiResponse::<()>::error_empty(
                            ErrorCode::InternalServerError,
                            "Storage service unavailable",
                        )),
                    )
                    .into()
                })
        }
    }

    /// Return a reference to the internal Storage if available.
    /// Implement this by returning `self.storage.clone()` or `None`.
    fn storage_ref(&self) -> Option<Arc<dyn Storage>>;
}

pub use auth::AuthService;
pub use class_users::ClassUserService;
pub use classes::ClassService;
pub use files::FileService;
pub use grades::GradeService;
pub use homeworks::HomeworkService;
pub use notifications::NotificationService;
pub use submissions::SubmissionService;
pub use system::SystemService;
pub use users::UserService;
pub use websocket::{
    WebSocketService, get_online_count, is_user_online, push_notification_to_user,
    push_notification_to_users,
};
