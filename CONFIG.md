# 配置说明

本项目使用 TOML 格式配置文件。

## 配置文件

- `config.toml` - 默认配置
- `config.development.toml` - 开发环境
- `config.production.toml` - 生产环境
- `config.example.toml` - 示例配置

### 加载顺序

1. 加载 `config.toml`（如果存在）
2. 根据 `APP_ENV` 加载对应环境配置
3. 环境变量覆盖

## 快速开始

```bash
cp config.example.toml config.toml
# 编辑 config.toml
```

## 配置项

### app - 应用设置

| 字段 | 类型 | 说明 |
|------|------|------|
| system_name | string | 系统名称 |
| environment | string | 运行环境 (development/production) |
| log_level | string | 日志级别 (trace/debug/info/warn/error) |

### server - 服务器设置

| 字段 | 类型 | 说明 |
|------|------|------|
| host | string | 监听地址 |
| port | u16 | 监听端口 |
| unix_socket_path | string | Unix 套接字路径（留空不使用） |
| workers | usize | 工作线程数（0=自动） |
| max_workers | usize | 最大工作线程数 |

#### server.timeouts

| 字段 | 类型 | 说明 |
|------|------|------|
| client_request | u64 | 客户端请求超时 (毫秒) |
| client_disconnect | u64 | 客户端断连超时 (毫秒) |
| keep_alive | u64 | Keep-Alive 超时 (秒) |

#### server.limits

| 字段 | 类型 | 说明 |
|------|------|------|
| max_payload_size | usize | 最大请求体大小 (字节) |

### jwt - JWT 设置

| 字段 | 类型 | 说明 |
|------|------|------|
| secret | string | JWT 密钥（生产环境务必使用强密钥） |
| access_token_expiry | i64 | Access Token 有效期 (分钟) |
| refresh_token_expiry | i64 | Refresh Token 有效期 (天) |
| refresh_token_remember_me_expiry | i64 | 记住我选项有效期 (天) |

### database - 数据库设置

| 字段 | 类型 | 说明 |
|------|------|------|
| url | string | 数据库连接 URL（从 scheme 自动推断类型） |
| pool_size | u32 | 连接池大小 |
| timeout | u64 | 连接超时 (秒) |

URL 格式：
- SQLite: `hwsystem.db` 或 `sqlite://hwsystem.db`
- MySQL: `mysql://user:password@localhost/hwsystem`
- PostgreSQL: `postgres://user:password@localhost/hwsystem`

### cache - 缓存设置

| 字段 | 类型 | 说明 |
|------|------|------|
| type | string | 缓存类型 (moka/redis) |
| default_ttl | u64 | 默认 TTL (秒) |

#### cache.redis

| 字段 | 类型 | 说明 |
|------|------|------|
| url | string | Redis 连接 URL |
| key_prefix | string | 键前缀 |
| pool_size | u64 | 连接池大小 |

#### cache.memory

| 字段 | 类型 | 说明 |
|------|------|------|
| max_capacity | u64 | 最大缓存条目数 |

### cors - 跨域设置

| 字段 | 类型 | 说明 |
|------|------|------|
| allowed_origins | Vec<string> | 允许的源（空数组=允许所有） |
| allowed_methods | Vec<string> | 允许的方法 |
| allowed_headers | Vec<string> | 允许的头部 |
| max_age | usize | 预检请求缓存时间 (秒) |

### upload - 上传设置

| 字段 | 类型 | 说明 |
|------|------|------|
| dir | string | 上传目录 |
| max_size | usize | 单文件最大字节数 |
| ~~allowed_types~~ | ~~Vec<string>~~ | **已废弃**：请使用数据库动态配置（系统设置 > upload.allowed_types） |
| timeout | u64 | 文件操作超时（毫秒） |

### argon2 - 密码哈希设置

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| memory_cost | u32 | 65536 | 内存消耗 (KiB) |
| time_cost | u32 | 3 | 迭代次数 |
| parallelism | u32 | 4 | 并行度 |

## 环境变量

使用 `HWSYSTEM_` 前缀覆盖配置：

```bash
HWSYSTEM_SERVER_PORT=9000
HWSYSTEM_JWT_SECRET=my_secret
HWSYSTEM_DATABASE_URL=postgres://...
```

兼容旧格式：

| 环境变量 | 配置项 |
|----------|--------|
| APP_ENV | app.environment |
| RUST_LOG | app.log_level |
| SERVER_HOST | server.host |
| SERVER_PORT | server.port |
| UNIX_SOCKET | server.unix_socket_path |
| CPU_COUNT | server.workers |
| JWT_SECRET | jwt.secret |
| DATABASE_URL | database.url |
| REDIS_URL | cache.redis.url |
| REDIS_KEY_PREFIX | cache.redis.key_prefix |
| REDIS_TTL | cache.redis.default_ttl |
