pub mod auth;

pub mod users;

pub mod classes;

pub mod class_users;

pub mod files;

pub mod homeworks;

pub mod submissions;

pub mod grades;

pub mod notifications;

pub mod system;

pub mod frontend;

pub mod websocket;

pub use auth::configure_auth_routes;
pub use class_users::configure_class_users_routes;
pub use classes::configure_classes_routes;
pub use files::configure_file_routes;
pub use frontend::configure_frontend_routes;
pub use grades::configure_grades_routes;
pub use homeworks::configure_homeworks_routes;
pub use notifications::configure_notifications_routes;
pub use submissions::configure_submissions_routes;
pub use system::configure_system_routes;
pub use users::configure_user_routes;
pub use websocket::configure_websocket_routes;
