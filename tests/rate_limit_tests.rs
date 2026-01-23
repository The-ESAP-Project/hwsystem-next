//! 速率限制中间件单元测试

use rust_hwsystem_next::middlewares::RateLimit;

#[test]
fn test_rate_limit_login_preset() {
    let rate_limit = RateLimit::login();
    // 登录限制：5次/分钟
    // 由于字段是私有的，我们只能测试创建不会 panic
    let _ = rate_limit;
}

#[test]
fn test_rate_limit_register_preset() {
    let rate_limit = RateLimit::register();
    // 注册限制：3次/分钟
    let _ = rate_limit;
}

#[test]
fn test_rate_limit_file_upload_preset() {
    let rate_limit = RateLimit::file_upload();
    // 文件上传限制：10次/分钟
    let _ = rate_limit;
}

#[test]
fn test_rate_limit_api_preset() {
    let rate_limit = RateLimit::api();
    // API 限制：100次/分钟
    let _ = rate_limit;
}

#[test]
fn test_rate_limit_custom() {
    let rate_limit = RateLimit::new(50, 120);
    // 自定义限制：50次/2分钟
    let _ = rate_limit;
}

#[test]
fn test_rate_limit_with_prefix() {
    let rate_limit = RateLimit::new(10, 60).with_prefix("custom");
    let _ = rate_limit;
}
