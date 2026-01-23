use serde::{Deserialize, Serialize};
use ts_rs::TS;

// 用户角色
#[derive(Debug, Clone, Serialize, PartialEq, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../frontend/src/types/generated/user.ts")]
pub enum UserRole {
    User,    // 普通用户
    Teacher, // 教师
    Admin,   // 管理员
}

impl UserRole {
    pub const USER: &'static str = "user";
    pub const TEACHER: &'static str = "teacher";
    pub const ADMIN: &'static str = "admin";

    pub fn admin_roles() -> &'static [&'static UserRole] {
        &[&Self::Admin]
    }
    pub fn teacher_roles() -> &'static [&'static UserRole] {
        &[&Self::Teacher, &Self::Admin]
    }
    pub fn user_roles() -> &'static [&'static UserRole] {
        &[&Self::User, &Self::Teacher]
    }
    pub fn all_roles() -> &'static [&'static UserRole] {
        &[&Self::User, &Self::Teacher, &Self::Admin]
    }
}

impl<'de> Deserialize<'de> for UserRole {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            UserRole::USER => Ok(UserRole::User),
            UserRole::TEACHER => Ok(UserRole::Teacher),
            UserRole::ADMIN => Ok(UserRole::Admin),
            _ => Err(serde::de::Error::custom(format!(
                "无效的用户角色: '{s}'. 支持的角色: user, teacher, admin"
            ))),
        }
    }
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::User => write!(f, "{}", UserRole::USER),
            UserRole::Teacher => write!(f, "{}", UserRole::TEACHER),
            UserRole::Admin => write!(f, "{}", UserRole::ADMIN),
        }
    }
}

impl std::str::FromStr for UserRole {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "user" => Ok(UserRole::User),
            "teacher" => Ok(UserRole::Teacher),
            "admin" => Ok(UserRole::Admin),
            _ => Err(format!("Invalid user role: {s}")),
        }
    }
}

// 用户状态
#[derive(Debug, Clone, Serialize, PartialEq, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../frontend/src/types/generated/user.ts")]
pub enum UserStatus {
    Active,    // 活跃
    Suspended, // 暂停
    Banned,    // 封禁
}

impl<'de> Deserialize<'de> for UserStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "active" => Ok(UserStatus::Active),
            "suspended" => Ok(UserStatus::Suspended),
            "banned" => Ok(UserStatus::Banned),
            _ => Err(serde::de::Error::custom(format!(
                "无效的用户状态: '{s}'. 支持的状态: active, suspended, banned"
            ))),
        }
    }
}

impl std::fmt::Display for UserStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserStatus::Active => write!(f, "active"),
            UserStatus::Suspended => write!(f, "suspended"),
            UserStatus::Banned => write!(f, "banned"),
        }
    }
}

impl std::str::FromStr for UserStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "active" => Ok(UserStatus::Active),
            "suspended" => Ok(UserStatus::Suspended),
            "banned" => Ok(UserStatus::Banned),
            _ => Err(format!("Invalid user status: {s}")),
        }
    }
}

// 用户实体
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/user.ts")]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing, default)] // 不序列化到JSON响应中
    #[ts(skip)]
    pub password_hash: String,
    pub role: UserRole,
    pub status: UserStatus,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub last_login: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl User {
    // 生成访问令牌（使用真正的 JWT）
    pub async fn generate_access_token(&self) -> String {
        // 使用 JwtUtils 生成 access token
        match crate::utils::jwt::JwtUtils::generate_access_token(self.id, &self.role.to_string()) {
            Ok(token) => token,
            Err(e) => {
                // 如果 JWT 生成失败，返回一个简单的 token（不推荐在生产环境中使用）
                tracing::error!("JWT token 生成失败: {}", e);
                format!(
                    "fallback_token_{}_{}",
                    self.id,
                    chrono::Utc::now().timestamp()
                )
            }
        }
    }

    // 生成刷新令牌
    pub async fn generate_refresh_token(
        &self,
        refresh_token_expiry: Option<chrono::TimeDelta>,
    ) -> String {
        match crate::utils::jwt::JwtUtils::generate_refresh_token(
            self.id,
            &self.role.to_string(),
            refresh_token_expiry,
        ) {
            Ok(token) => token,
            Err(e) => {
                tracing::error!("JWT refresh token 生成失败: {}", e);
                format!(
                    "fallback_refresh_token_{}_{}",
                    self.id,
                    chrono::Utc::now().timestamp()
                )
            }
        }
    }

    // 生成 token 对（access + refresh）
    pub async fn generate_token_pair(
        &self,
        refresh_token_expiry: Option<chrono::TimeDelta>,
    ) -> Result<crate::utils::jwt::TokenPair, String> {
        crate::utils::jwt::JwtUtils::generate_token_pair(
            self.id,
            &self.role.to_string(),
            refresh_token_expiry,
        )
        .map_err(|e| format!("生成 token 对失败: {e}"))
    }
}
