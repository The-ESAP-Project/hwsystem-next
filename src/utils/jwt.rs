use crate::config::AppConfig;
use actix_web::cookie::{Cookie, SameSite};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

/// 缓存 JWT 编解码 Key，避免每次操作都重新构建
static JWT_KEYS: OnceLock<(EncodingKey, DecodingKey)> = OnceLock::new();

fn get_jwt_keys() -> &'static (EncodingKey, DecodingKey) {
    JWT_KEYS.get_or_init(|| {
        let secret = AppConfig::get().jwt.secret.clone();
        let encoding = EncodingKey::from_secret(secret.as_bytes());
        let decoding = DecodingKey::from_secret(secret.as_bytes());
        (encoding, decoding)
    })
}

// JWT Claims 结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,        // Subject (user ID)
    pub role: String,       // 用户角色
    pub token_type: String, // token类型: "access" 或 "refresh"
    pub exp: usize,         // Expiration time (时间戳)
    pub iat: usize,         // Issued at (签发时间)
}

// Token 响应结构体
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
}

pub struct JwtUtils;

impl JwtUtils {
    // 生成 Access Token
    pub fn generate_access_token(
        user_id: i64,
        role: &str,
    ) -> Result<String, jsonwebtoken::errors::Error> {
        let config = AppConfig::get();
        Self::generate_token_with_expiry(
            user_id,
            role,
            "access",
            chrono::Duration::minutes(config.jwt.access_token_expiry),
        )
    }

    // 生成 Refresh Token
    pub fn generate_refresh_token(
        user_id: i64,
        role: &str,
        token_expiry: Option<chrono::Duration>,
    ) -> Result<String, jsonwebtoken::errors::Error> {
        let config = AppConfig::get();
        match token_expiry {
            Some(expiry) => Self::generate_token_with_expiry(user_id, role, "refresh", expiry),
            None => Self::generate_token_with_expiry(
                user_id,
                role,
                "refresh",
                chrono::Duration::days(config.jwt.refresh_token_expiry),
            ),
        }
    }

    // 生成带自定义过期时间的 Token
    pub fn generate_token_with_expiry(
        user_id: i64,
        role: &str,
        token_type: &str,
        expiry_duration: chrono::Duration,
    ) -> Result<String, jsonwebtoken::errors::Error> {
        let now = chrono::Utc::now();
        let expiration = now + expiry_duration;

        let claims = Claims {
            sub: user_id.to_string(),
            role: role.to_string(),
            token_type: token_type.to_string(),
            exp: expiration.timestamp() as usize,
            iat: now.timestamp() as usize,
        };

        let (encoding_key, _) = get_jwt_keys();
        encode(&Header::default(), &claims, encoding_key)
    }

    // 生成完整的 Token 响应（包含 access 和 refresh token）
    pub fn generate_token_pair(
        user_id: i64,
        role: &str,
        refresh_token_expiry: Option<chrono::Duration>,
    ) -> Result<TokenPair, jsonwebtoken::errors::Error> {
        let access_token = Self::generate_access_token(user_id, role)?;
        let refresh_token = Self::generate_refresh_token(user_id, role, refresh_token_expiry)?;

        Ok(TokenPair {
            access_token,
            refresh_token,
        })
    }

    // 验证 JWT token
    pub fn verify_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let (_, decoding_key) = get_jwt_keys();
        let validation = Validation::default();
        decode::<Claims>(token, decoding_key, &validation).map(|token_data| token_data.claims)
    }

    // 验证 token 是否为指定类型
    pub fn verify_token_type(
        token: &str,
        expected_type: &str,
    ) -> Result<Claims, jsonwebtoken::errors::Error> {
        let claims = Self::verify_token(token)?;
        if claims.token_type != expected_type {
            return Err(jsonwebtoken::errors::Error::from(
                jsonwebtoken::errors::ErrorKind::InvalidToken,
            ));
        }
        Ok(claims)
    }

    // 验证 Access Token
    pub fn verify_access_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        Self::verify_token_type(token, "access")
    }

    // 验证 Refresh Token
    pub fn verify_refresh_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        Self::verify_token_type(token, "refresh")
    }

    pub fn decode_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let (_, decoding_key) = get_jwt_keys();
        let validation = Validation::default();
        decode::<Claims>(token, decoding_key, &validation).map(|token_data| token_data.claims)
    }

    // 使用 Refresh Token 生成新的 Access Token
    pub fn refresh_access_token(
        refresh_token: &str,
    ) -> Result<String, jsonwebtoken::errors::Error> {
        let claims = Self::verify_refresh_token(refresh_token)?;
        let user_id = claims
            .sub
            .parse::<i64>()
            .map_err(|_| jsonwebtoken::errors::ErrorKind::InvalidToken)?;
        Self::generate_access_token(user_id, &claims.role)
    }

    /// 创建 Refresh Token Cookie
    pub fn create_refresh_token_cookie(refresh_token: &str) -> Cookie<'static> {
        let config = AppConfig::get();
        Cookie::build("refresh_token", refresh_token.to_string())
            .path("/")
            .max_age(actix_web::cookie::time::Duration::days(
                config.jwt.refresh_token_expiry,
            ))
            .same_site(SameSite::Strict)
            .http_only(true)
            .secure(config.is_production()) // 生产环境下使用 HTTPS
            .finish()
    }

    /// 创建空的 Refresh Token Cookie（用于注销）
    pub fn create_empty_refresh_token_cookie() -> Cookie<'static> {
        let config = AppConfig::get();
        Cookie::build("refresh_token", "")
            .path("/")
            .max_age(actix_web::cookie::time::Duration::seconds(0))
            .same_site(SameSite::Strict)
            .http_only(true)
            .secure(config.is_production())
            .finish()
    }

    /// 从请求中提取 Refresh Token
    pub fn extract_refresh_token_from_cookie(req: &actix_web::HttpRequest) -> Option<String> {
        req.cookie("refresh_token")
            .map(|cookie| cookie.value().to_string())
    }
}
