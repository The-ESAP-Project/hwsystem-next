use crate::models::common::serialization::serialize_i64_as_string;
use crate::models::users::entities::User;
use serde::Serialize;
use ts_rs::TS;

// 用户响应模型
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/auth.ts")]
pub struct LoginResponse {
    pub access_token: String,
    #[serde(serialize_with = "serialize_i64_as_string")]
    #[ts(type = "string")]
    pub expires_in: i64,
    pub user: User,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/auth.ts")]
pub struct RefreshTokenResponse {
    pub access_token: String,
    #[serde(serialize_with = "serialize_i64_as_string")]
    #[ts(type = "string")]
    pub expires_in: i64,
}

#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/auth.ts")]
pub struct UserInfoResponse {
    pub user: User,
}

#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/auth.ts")]
pub struct TokenVerificationResponse {
    pub is_valid: bool,
}
