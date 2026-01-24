/// SQL LIKE 查询参数转义
///
/// 转义 LIKE 模式匹配中的特殊字符 `%` 和 `_`，防止用户输入影响查询行为
pub fn escape_like_pattern(input: &str) -> String {
    input
        .replace('\\', "\\\\") // 先转义反斜杠
        .replace('%', "\\%")
        .replace('_', "\\_")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_like_pattern() {
        assert_eq!(escape_like_pattern("hello"), "hello");
        assert_eq!(escape_like_pattern("hello%world"), "hello\\%world");
        assert_eq!(escape_like_pattern("hello_world"), "hello\\_world");
        assert_eq!(escape_like_pattern("100%_test"), "100\\%\\_test");
        assert_eq!(escape_like_pattern("a\\b"), "a\\\\b");
    }
}
