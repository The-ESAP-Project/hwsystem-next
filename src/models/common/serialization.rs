//! i64/u64 序列化/反序列化辅助函数
//!
//! 用于将 i64/u64 字段序列化为字符串，解决前端 JavaScript/TypeScript
//! 处理 bigint 类型的兼容性问题。

use serde::de::{Error, Unexpected, Visitor};
use serde::{Deserializer, Serializer};
use std::fmt;

/// 将 i64 序列化为字符串
///
/// # Example
/// ```rust
/// use serde::Serialize;
/// use rust_hwsystem_next::models::common::serialization::serialize_i64_as_string;
///
/// #[derive(Serialize)]
/// struct Example {
///     #[serde(serialize_with = "serialize_i64_as_string")]
///     id: i64,
/// }
/// ```
pub fn serialize_i64_as_string<S>(value: &i64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&value.to_string())
}

/// 将 Option<i64> 序列化为字符串（None 保持为 null）
///
/// # Example
/// ```rust
/// use serde::Serialize;
/// use rust_hwsystem_next::models::common::serialization::serialize_option_i64_as_string;
///
/// #[derive(Serialize)]
/// struct Example {
///     #[serde(serialize_with = "serialize_option_i64_as_string")]
///     user_id: Option<i64>,
/// }
/// ```
pub fn serialize_option_i64_as_string<S>(
    value: &Option<i64>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match value {
        Some(v) => serializer.serialize_some(&v.to_string()),
        None => serializer.serialize_none(),
    }
}

/// 将 u64 序列化为字符串
pub fn serialize_u64_as_string<S>(value: &u64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&value.to_string())
}

/// 从字符串或数字反序列化为 i64
///
/// 支持以下输入格式：
/// - 数字: `123`
/// - 字符串: `"123"`
///
/// # Example
/// ```rust
/// use serde::Deserialize;
/// use rust_hwsystem_next::models::common::serialization::deserialize_string_to_i64;
///
/// #[derive(Deserialize)]
/// struct Example {
///     #[serde(deserialize_with = "deserialize_string_to_i64")]
///     id: i64,
/// }
/// ```
pub fn deserialize_string_to_i64<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    struct I64Visitor;

    impl<'de> Visitor<'de> for I64Visitor {
        type Value = i64;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("an integer or a string containing an integer")
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(value)
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: Error,
        {
            if value <= i64::MAX as u64 {
                Ok(value as i64)
            } else {
                Err(Error::invalid_value(Unexpected::Unsigned(value), &self))
            }
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            value
                .parse()
                .map_err(|_| Error::invalid_value(Unexpected::Str(value), &self))
        }
    }

    deserializer.deserialize_any(I64Visitor)
}

/// 从字符串或数字反序列化为 Option<i64>
///
/// 支持以下输入格式：
/// - null: `null` -> `None`
/// - 数字: `123` -> `Some(123)`
/// - 字符串: `"123"` -> `Some(123)`
pub fn deserialize_option_string_to_i64<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    struct OptionI64Visitor;

    impl<'de> Visitor<'de> for OptionI64Visitor {
        type Value = Option<i64>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("null, an integer, or a string containing an integer")
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(None)
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(None)
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserialize_string_to_i64(deserializer).map(Some)
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(Some(value))
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: Error,
        {
            if value <= i64::MAX as u64 {
                Ok(Some(value as i64))
            } else {
                Err(Error::invalid_value(Unexpected::Unsigned(value), &self))
            }
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            value
                .parse()
                .map(Some)
                .map_err(|_| Error::invalid_value(Unexpected::Str(value), &self))
        }
    }

    deserializer.deserialize_any(OptionI64Visitor)
}
