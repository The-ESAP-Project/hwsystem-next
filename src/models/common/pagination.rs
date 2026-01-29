use serde::{Deserialize, Serialize};
use ts_rs::TS;

// 分页常量
pub const DEFAULT_PAGE: i64 = 1;
pub const DEFAULT_PAGE_SIZE: i64 = 20;
pub const MAX_PAGE_SIZE: i64 = 100;

// 分页查询参数
#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/pagination.ts")]
pub struct PaginationQuery {
    #[serde(
        default = "default_page",
        deserialize_with = "deserialize_string_to_i64"
    )]
    #[ts(type = "number")]
    pub page: i64,
    #[serde(
        default = "default_page_size",
        deserialize_with = "deserialize_string_to_i64"
    )]
    #[ts(type = "number")]
    pub page_size: i64,
}

// 分页响应信息
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/pagination.ts")]
pub struct PaginationInfo {
    #[ts(type = "number")]
    pub page: i64,
    #[ts(type = "number")]
    pub page_size: i64,
    #[ts(type = "number")]
    pub total: i64,
    #[ts(type = "number")]
    pub total_pages: i64,
}

// 分页列表响应
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/pagination.ts")]
pub struct PaginatedResponse<T: TS> {
    pub items: Vec<T>,
    pub pagination: PaginationInfo,
}

// 自定义反序列化函数，支持字符串到i64的转换
fn deserialize_string_to_i64<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{Error, Unexpected, Visitor};
    use std::fmt;

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

fn default_page() -> i64 {
    DEFAULT_PAGE
}

fn default_page_size() -> i64 {
    DEFAULT_PAGE_SIZE
}

impl PaginationQuery {
    /// 返回验证后的分页参数 (u64)
    /// - page: 最小为 1
    /// - page_size: 范围 [1, MAX_PAGE_SIZE]
    pub fn normalized(&self) -> (u64, u64) {
        let page = self.page.max(DEFAULT_PAGE) as u64;
        let page_size = self.page_size.clamp(1, MAX_PAGE_SIZE) as u64;
        (page, page_size)
    }
}

impl Default for PaginationQuery {
    fn default() -> Self {
        Self {
            page: DEFAULT_PAGE,
            page_size: DEFAULT_PAGE_SIZE,
        }
    }
}
