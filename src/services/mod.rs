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
