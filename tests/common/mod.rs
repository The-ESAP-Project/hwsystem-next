//! 测试工具模块
//!
//! 提供测试所需的通用工具。

/// 初始化测试环境
pub fn init_test_env() {
    // 设置测试环境变量
    std::env::set_var("APP_ENV", "development");
    std::env::set_var("RUST_LOG", "debug");
}
