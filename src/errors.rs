//! 统一错误处理模块
//!
//! 使用宏自动生成错误类型，支持错误代码和类型名称。

use std::fmt;

use crate::models::ErrorCode;

/// 定义错误类型的宏
///
/// 自动生成：
/// - enum 定义
/// - code() 方法 - 返回错误代码
/// - error_type() 方法 - 返回错误类型名称
/// - message() 方法 - 返回错误详情
/// - 便捷构造函数
macro_rules! define_hwsystem_errors {
    ($(
        $variant:ident($code:literal, $type_name:literal)
    ),* $(,)?) => {
        #[derive(Debug, Clone)]
        pub enum HWSystemError {
            $($variant(String),)*
        }

        impl HWSystemError {
            /// 获取错误代码
            pub fn code(&self) -> &'static str {
                match self {
                    $(HWSystemError::$variant(_) => $code,)*
                }
            }

            /// 获取错误类型名称
            pub fn error_type(&self) -> &'static str {
                match self {
                    $(HWSystemError::$variant(_) => $type_name,)*
                }
            }

            /// 获取错误详情
            pub fn message(&self) -> &str {
                match self {
                    $(HWSystemError::$variant(msg) => msg,)*
                }
            }
        }

        // 生成便捷构造函数
        paste::paste! {
            impl HWSystemError {
                $(
                    pub fn [<$variant:snake>]<T: Into<String>>(msg: T) -> Self {
                        HWSystemError::$variant(msg.into())
                    }
                )*
            }
        }
    };
}

define_hwsystem_errors! {
    CacheConnection("E001", "Cache Connection Error"),
    CachePluginNotFound("E002", "Cache Plugin Not Found"),
    DatabaseConfig("E003", "Database Configuration Error"),
    DatabaseConnection("E004", "Database Connection Error"),
    DatabaseOperation("E005", "Database Operation Error"),
    FileOperation("E006", "File Operation Error"),
    Validation("E007", "Validation Error"),
    NotFound("E008", "Resource Not Found"),
    Serialization("E009", "Serialization Error"),
    StoragePluginNotFound("E010", "Storage Plugin Not Found"),
    DateParse("E011", "Date Parse Error"),
    Authentication("E012", "Authentication Error"),
    Authorization("E013", "Authorization Error"),
}

impl HWSystemError {
    /// 格式化为彩色输出（用于开发环境）
    #[cfg(debug_assertions)]
    pub fn format_colored(&self) -> String {
        format!(
            "\x1b[1;31m[ERROR]\x1b[0m \x1b[33m{}\x1b[0m \x1b[31m{}\x1b[0m\n  {}",
            self.code(),
            self.error_type(),
            self.message()
        )
    }

    /// 格式化为简洁输出
    pub fn format_simple(&self) -> String {
        format!("{}: {}", self.error_type(), self.message())
    }
}

impl fmt::Display for HWSystemError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_simple())
    }
}

impl std::error::Error for HWSystemError {}

impl From<HWSystemError> for ErrorCode {
    fn from(err: HWSystemError) -> Self {
        match err {
            HWSystemError::CacheConnection(_) => ErrorCode::InternalServerError,
            HWSystemError::CachePluginNotFound(_) => ErrorCode::InternalServerError,
            HWSystemError::DatabaseConfig(_) => ErrorCode::InternalServerError,
            HWSystemError::DatabaseConnection(_) => ErrorCode::InternalServerError,
            HWSystemError::DatabaseOperation(_) => ErrorCode::InternalServerError,
            HWSystemError::FileOperation(_) => ErrorCode::FileUploadFailed,
            HWSystemError::Validation(_) => ErrorCode::BadRequest,
            HWSystemError::NotFound(_) => ErrorCode::NotFound,
            HWSystemError::Serialization(_) => ErrorCode::InternalServerError,
            HWSystemError::StoragePluginNotFound(_) => ErrorCode::InternalServerError,
            HWSystemError::DateParse(_) => ErrorCode::BadRequest,
            HWSystemError::Authentication(_) => ErrorCode::AuthFailed,
            HWSystemError::Authorization(_) => ErrorCode::Forbidden,
        }
    }
}

impl From<&HWSystemError> for ErrorCode {
    fn from(err: &HWSystemError) -> Self {
        match err {
            HWSystemError::CacheConnection(_) => ErrorCode::InternalServerError,
            HWSystemError::CachePluginNotFound(_) => ErrorCode::InternalServerError,
            HWSystemError::DatabaseConfig(_) => ErrorCode::InternalServerError,
            HWSystemError::DatabaseConnection(_) => ErrorCode::InternalServerError,
            HWSystemError::DatabaseOperation(_) => ErrorCode::InternalServerError,
            HWSystemError::FileOperation(_) => ErrorCode::FileUploadFailed,
            HWSystemError::Validation(_) => ErrorCode::BadRequest,
            HWSystemError::NotFound(_) => ErrorCode::NotFound,
            HWSystemError::Serialization(_) => ErrorCode::InternalServerError,
            HWSystemError::StoragePluginNotFound(_) => ErrorCode::InternalServerError,
            HWSystemError::DateParse(_) => ErrorCode::BadRequest,
            HWSystemError::Authentication(_) => ErrorCode::AuthFailed,
            HWSystemError::Authorization(_) => ErrorCode::Forbidden,
        }
    }
}

// 为常见的错误类型实现 From trait
impl From<sea_orm::DbErr> for HWSystemError {
    fn from(err: sea_orm::DbErr) -> Self {
        HWSystemError::DatabaseOperation(err.to_string())
    }
}

impl From<std::io::Error> for HWSystemError {
    fn from(err: std::io::Error) -> Self {
        HWSystemError::FileOperation(err.to_string())
    }
}

impl From<serde_json::Error> for HWSystemError {
    fn from(err: serde_json::Error) -> Self {
        HWSystemError::Serialization(err.to_string())
    }
}

impl From<chrono::ParseError> for HWSystemError {
    fn from(err: chrono::ParseError) -> Self {
        HWSystemError::DateParse(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, HWSystemError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        assert_eq!(HWSystemError::cache_connection("test").code(), "E001");
        assert_eq!(HWSystemError::database_config("test").code(), "E003");
        assert_eq!(HWSystemError::validation("test").code(), "E007");
        assert_eq!(HWSystemError::authentication("test").code(), "E012");
    }

    #[test]
    fn test_error_types() {
        assert_eq!(
            HWSystemError::cache_connection("test").error_type(),
            "Cache Connection Error"
        );
        assert_eq!(
            HWSystemError::validation("test").error_type(),
            "Validation Error"
        );
    }

    #[test]
    fn test_error_message() {
        let err = HWSystemError::validation("Invalid input");
        assert_eq!(err.message(), "Invalid input");
    }

    #[test]
    fn test_format_simple() {
        let err = HWSystemError::validation("Invalid URL");
        let formatted = err.format_simple();
        assert!(formatted.contains("Validation Error"));
        assert!(formatted.contains("Invalid URL"));
    }
}
