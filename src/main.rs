use actix_cors::Cors;
use actix_web::http;
use actix_web::middleware::{Compress, DefaultHeaders};
use actix_web::{App, HttpServer, web};
use dotenv::dotenv;
use human_panic::setup_panic;
use tracing::{debug, warn};

// 从 lib.rs 导入模块
use rust_hwsystem_next::config::AppConfig;
use rust_hwsystem_next::models::AppStartTime;
use rust_hwsystem_next::routes;
use rust_hwsystem_next::runtime::lifetime;
use rust_hwsystem_next::utils::{json_error_handler, query_error_handler};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    // 记录程序启动时间
    let app_start_time = AppStartTime {
        start_datetime: chrono::Utc::now(),
    };

    // 启动前预处理 //

    // 初始化配置
    setup_panic!();
    AppConfig::init().expect("Failed to initialize configuration");
    let config = AppConfig::get();

    // 初始化日志
    let stdout_log = std::io::stdout();
    let (non_blocking_writer, _guard) = tracing_appender::non_blocking(stdout_log);
    let filter = tracing_subscriber::EnvFilter::new(&config.app.log_level);
    let tracing_format = tracing_subscriber::fmt::format()
        .with_level(true)
        .with_ansi(true);

    let tracing_builder = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(non_blocking_writer)
        .event_format(tracing_format);

    if config.is_development() {
        tracing_builder
            .with_file(true)
            .with_line_number(true)
            .init();
    } else {
        tracing_builder.json().init();
    }

    // 打印信息
    warn!(
        "Starting pre-startup processing...
        Project: {}
        Version: {}
        Authors: {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_AUTHORS")
    );

    let startup = lifetime::startup::prepare_server_startup().await;

    let storage = startup.storage.clone();
    let cache = startup.cache.clone();

    // 输出预处理时间
    debug!(
        "Pre-startup processing completed in {} ms",
        chrono::Utc::now()
            .signed_duration_since(app_start_time.start_datetime)
            .num_milliseconds()
    );

    // 预处理完成 //

    warn!("Using {} CPU cores for the server", config.server.workers);

    // Start the HTTP server
    let server = HttpServer::new(move || {
        App::new()
            .wrap({
                let mut cors = Cors::default();

                // 配置允许的来源
                if config.cors.allowed_origins.is_empty() {
                    cors = cors.allow_any_origin();
                } else {
                    for origin in &config.cors.allowed_origins {
                        cors = cors.allowed_origin(origin);
                    }
                }

                // 配置允许的方法
                for method in &config.cors.allowed_methods {
                    if let Ok(m) = method.parse::<http::Method>() {
                        cors = cors.allowed_methods(vec![m]);
                    }
                }

                // 配置允许的头部
                if config.cors.allowed_headers.iter().any(|h| h == "*") {
                    cors = cors.allow_any_header();
                } else {
                    for header in &config.cors.allowed_headers {
                        cors = cors.allowed_header(header);
                    }
                }

                cors.max_age(config.cors.max_age)
            })
            .wrap(Compress::default())
            .wrap(
                DefaultHeaders::new()
                    .add(("Connection", "keep-alive"))
                    .add((
                        "Keep-Alive",
                        format!("timeout={}, max=1000", config.server.timeouts.keep_alive),
                    ))
                    .add(("Cache-Control", "no-cache, no-store, must-revalidate")),
            )
            .app_data(web::QueryConfig::default().error_handler(query_error_handler)) // 设置查询参数错误处理器
            .app_data(web::JsonConfig::default().error_handler(json_error_handler)) // 设置JSON错误处理器
            .app_data(web::Data::new(storage.clone()))
            .app_data(web::Data::new(cache.clone()))
            .app_data(web::Data::new(app_start_time.clone()))
            .app_data(web::PayloadConfig::new(
                config.server.limits.max_payload_size,
            )) // 设置最大请求体大小
            .configure(routes::configure_auth_routes) // 配置认证相关路由
            .configure(routes::configure_user_routes) // 配置用户相关路由
            .configure(routes::configure_class_users_routes) //配置班级成员相关路由
            .configure(routes::configure_classes_routes) // 配置班级相关路由
            .configure(routes::configure_submissions_routes) // 配置提交相关路由（必须在 homeworks 之前，因为有 /homeworks/{id}/submissions 路由）
            .configure(routes::configure_homeworks_routes) // 配置作业相关路由
            .configure(routes::configure_grades_routes) // 配置评分相关路由
            .configure(routes::configure_notifications_routes) // 配置通知相关路由
            .configure(routes::configure_websocket_routes) // 配置 WebSocket 路由
            .configure(routes::configure_file_routes) // 配置文件相关路由
            .configure(routes::configure_system_routes) // 配置系统相关路由
            .configure(routes::configure_frontend_routes) // 配置前端静态资源路由（放在最后作为 fallback）
    })
    .keep_alive(std::time::Duration::from_secs(
        config.server.timeouts.keep_alive,
    )) // 启用长连接
    .client_request_timeout(std::time::Duration::from_millis(
        config.server.timeouts.client_request,
    )) // 客户端超时
    .client_disconnect_timeout(std::time::Duration::from_millis(
        config.server.timeouts.client_disconnect,
    )) // 断连超时
    .workers(config.server.workers);

    let server = {
        #[cfg(unix)]
        {
            if let Some(socket_path) = config.unix_socket_path() {
                warn!("Starting server on Unix socket: {}", socket_path);
                if std::path::Path::new(socket_path).exists() {
                    std::fs::remove_file(socket_path)?;
                }
                Some(server.bind_uds(socket_path)?)
            } else {
                let bind_address = config.server_bind_address();
                warn!("Starting server at http://{}", bind_address);
                Some(server.bind(bind_address)?)
            }
        }

        #[cfg(not(unix))]
        {
            let bind_address = config.server_bind_address();
            warn!("Starting server at http://{}", bind_address);
            Some(server.bind(bind_address)?)
        }
    }
    .expect("Server binding failed")
    .run();

    tokio::select! {
        res = server => {
            res?;
        }
        _ = lifetime::shutdown::listen_for_shutdown() => {
            warn!("Graceful shutdown: all tasks completed");
        }
    }

    Ok(())
}
