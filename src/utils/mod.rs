pub mod extractor;
pub mod file_magic; // 保留模块但不再使用，已被 infer crate 替代
pub mod jwt;
pub mod parameter_error_handler;
pub mod password;
pub mod random_code;
pub mod sql;
pub mod validate;

pub use extractor::{
    SafeClassCode, SafeClassIdI64, SafeFileToken, SafeGradeIdI64, SafeHomeworkIdI64, SafeIDI64,
    SafeNotificationIdI64, SafeSettingKey, SafeSubmissionIdI64,
};
// pub use file_magic::validate_magic_bytes; // 已被 infer crate 替代
pub use parameter_error_handler::json_error_handler;
pub use parameter_error_handler::query_error_handler;
pub use sql::escape_like_pattern;
