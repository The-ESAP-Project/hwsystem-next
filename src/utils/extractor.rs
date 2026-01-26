#[macro_export]
macro_rules! define_safe_i64_extractor {
    ($name:ident, $key:literal) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Deserialize)]
        pub struct $name(pub i64);

        impl actix_web::FromRequest for $name {
            type Error = actix_web::Error;
            type Future = std::future::Ready<Result<Self, Self::Error>>;

            fn from_request(
                req: &actix_web::HttpRequest,
                _: &mut actix_web::dev::Payload,
            ) -> Self::Future {
                use actix_web::{HttpResponse, error};
                let id_str = req.match_info().get($key).unwrap_or("");
                match id_str.parse::<i64>() {
                    Ok(id) => std::future::ready(Ok(Self(id))),
                    Err(_) => {
                        let resp = $crate::models::common::response::ApiResponse::<()>::error_empty(
                            $crate::models::ErrorCode::BadRequest,
                            concat!($key, " format error, please provide a valid numeric ID."),
                        );
                        std::future::ready(Err(error::InternalError::from_response(
                            "Invalid ID",
                            HttpResponse::BadRequest().json(resp),
                        )
                        .into()))
                    }
                }
            }
        }
    };
}

#[macro_export]
macro_rules! define_safe_string_extractor {
    ($name:ident, $key:literal) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Deserialize)]
        pub struct $name(pub String);

        impl actix_web::FromRequest for $name {
            type Error = actix_web::Error;
            type Future = std::future::Ready<Result<Self, Self::Error>>;

            fn from_request(
                req: &actix_web::HttpRequest,
                _: &mut actix_web::dev::Payload,
            ) -> Self::Future {
                use actix_web::{HttpResponse, error};
                let value = req.match_info().get($key).unwrap_or("");
                if value.is_empty() {
                    let resp = $crate::models::common::response::ApiResponse::<()>::error_empty(
                        $crate::models::ErrorCode::BadRequest,
                        concat!($key, " is required and cannot be empty."),
                    );
                    std::future::ready(Err(error::InternalError::from_response(
                        "Invalid parameter",
                        HttpResponse::BadRequest().json(resp),
                    )
                    .into()))
                } else {
                    std::future::ready(Ok(Self(value.to_string())))
                }
            }
        }
    };
}

define_safe_i64_extractor!(SafeIDI64, "id");
define_safe_i64_extractor!(SafeClassIdI64, "class_id");
define_safe_i64_extractor!(SafeHomeworkIdI64, "homework_id");
define_safe_i64_extractor!(SafeGradeIdI64, "grade_id");
define_safe_i64_extractor!(SafeSubmissionIdI64, "submission_id");
define_safe_i64_extractor!(SafeNotificationIdI64, "notification_id");

define_safe_string_extractor!(SafeClassCode, "code");
define_safe_string_extractor!(SafeFileToken, "file_token");
define_safe_string_extractor!(SafeSettingKey, "key");
