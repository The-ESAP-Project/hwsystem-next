# 安全设计文档

> 版本：v2.1
> 更新日期：2026-01-26

---

## 一、认证机制

### 1.1 JWT 双 Token 设计

系统采用 Access Token + Refresh Token 双令牌机制。

| Token 类型 | 有效期 | 存储位置 | 用途 |
|------------|--------|----------|------|
| Access Token | 15 分钟 | localStorage / 请求头 | API 认证 |
| Refresh Token | 7 天（记住我 30 天） | HttpOnly Cookie | 刷新 Access Token |

**设计原因**：
- Access Token 短期有效，即使泄露影响有限
- Refresh Token 存储在 HttpOnly Cookie，防止 XSS 窃取
- 分离认证与刷新，减少敏感操作暴露

### 1.2 Token Claims 结构

```json
{
    "sub": "user_id",           // 用户 ID
    "role": "user",             // 系统角色
    "token_type": "access",     // access / refresh
    "exp": 1706140800,          // 过期时间
    "iat": 1706140000           // 签发时间
}
```

### 1.3 Token 验证流程

```
1. 从 Authorization 头提取 Token
2. 验证 Token 签名
3. 验证 Token 是否过期
4. 验证 token_type 是否为 access
5. 尝试从缓存获取用户信息
6. 缓存未命中则查询数据库
7. 验证用户状态为 Active
8. 将用户信息写入缓存（TTL 1小时）
9. 将用户实体注入请求扩展
```

### 1.4 Refresh Token 安全

```rust
Cookie::build(("refresh_token", token))
    .http_only(true)              // 禁止 JavaScript 访问
    .secure(is_production)        // 生产环境仅 HTTPS
    .same_site(SameSite::Strict)  // 严格同站策略
    .path("/")                    // 全站有效
    .max_age(Duration::days(7))   // 有效期
```

---

## 二、密钥管理

### 2.1 JWT 密钥

**要求**：
- 最小长度：32 字符
- 推荐：64 字符随机字符串
- 来源：必须从环境变量 `JWT_SECRET` 读取

**启动检查**：
```rust
// 拒绝使用默认密钥启动
if config.jwt.secret == "default_secret_key" {
    panic!("JWT_SECRET must be set in production!");
}
```

**生成方式**：
```bash
# 使用 openssl 生成
openssl rand -base64 48

# 使用 Node.js 生成
node -e "console.log(require('crypto').randomBytes(48).toString('base64'))"
```

### 2.2 密钥轮换

建议每 90 天轮换一次 JWT 密钥：

1. 生成新密钥
2. 更新环境变量
3. 重启服务
4. 所有用户需重新登录（旧 Token 失效）

---

## 三、密码策略

### 3.1 密码要求

| 要求 | 规则 |
|------|------|
| 最小长度 | 8 字符 |
| 大写字母 | 至少 1 个 |
| 小写字母 | 至少 1 个 |
| 数字 | 至少 1 个 |

**验证正则**：
```rust
const PASSWORD_REGEX: &str = r"^(?=.*[a-z])(?=.*[A-Z])(?=.*\d).{8,}$";
```

### 3.2 密码存储

使用 Argon2id 算法哈希存储：

```rust
use argon2::{Argon2, PasswordHasher};

let argon2 = Argon2::default();
let salt = SaltString::generate(&mut OsRng);
let hash = argon2.hash_password(password.as_bytes(), &salt)?;
```

**Argon2 参数**：
- 算法：Argon2id
- 内存：64 MB
- 迭代：3 次
- 并行度：4

### 3.3 密码验证

```rust
use argon2::{Argon2, PasswordVerifier};

let parsed_hash = PasswordHash::new(&stored_hash)?;
Argon2::default().verify_password(password.as_bytes(), &parsed_hash)?;
```

---

## 四、速率限制

### 4.1 限制策略

| 端点 | 限制 | 维度 |
|------|------|------|
| POST /auth/login | 5 次/分钟 | IP |
| POST /auth/register | 3 次/分钟 | IP |
| POST /files/upload | 10 次/分钟 | 用户 |
| 其他 API | 100 次/分钟 | 用户 |

### 4.2 实现方式

使用滑动窗口算法 + Redis/内存缓存：

```rust
pub struct RateLimiter {
    window_size: Duration,     // 窗口大小（60秒）
    max_requests: u32,         // 最大请求数
}

impl RateLimiter {
    pub fn check(&self, key: &str) -> Result<(), RateLimitError> {
        let count = cache.incr(key, 1, self.window_size)?;
        if count > self.max_requests {
            Err(RateLimitError::TooManyRequests)
        } else {
            Ok(())
        }
    }
}
```

### 4.3 响应头

达到限制时返回：

```
HTTP/1.1 429 Too Many Requests
Retry-After: 60
X-RateLimit-Limit: 5
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1706140860
```

---

## 五、CORS 配置

### 5.1 生产环境配置

```toml
[cors]
allowed_origins = ["https://your-frontend.com"]
allowed_methods = ["GET", "POST", "PUT", "DELETE", "OPTIONS"]
allowed_headers = ["Authorization", "Content-Type"]
expose_headers = ["X-RateLimit-Limit", "X-RateLimit-Remaining"]
max_age = 3600
credentials = true
```

### 5.2 开发环境配置

```toml
[cors]
allowed_origins = ["http://localhost:3000", "http://127.0.0.1:3000"]
```

### 5.3 实现代码

```rust
let cors = Cors::default()
    .allowed_origin_fn(|origin, _| {
        config.cors.allowed_origins.iter()
            .any(|allowed| allowed == origin.to_str().unwrap_or(""))
    })
    .allowed_methods(config.cors.allowed_methods.clone())
    .allowed_headers(config.cors.allowed_headers.clone())
    .expose_headers(config.cors.expose_headers.clone())
    .max_age(config.cors.max_age)
    .supports_credentials();
```

---

## 六、文件上传安全

### 6.1 MIME 类型白名单

文件类型白名单通过动态系统设置配置，可在运行时通过管理员接口修改：

```rust
// 从数据库动态读取允许的文件类型
let allowed_types = DynamicConfig::upload_allowed_types().await;
```

**默认允许的类型**：
- `image/png`
- `image/jpeg`
- `image/gif`
- `application/pdf`
- `text/plain`
- `application/zip`
- `application/x-zip-compressed`

**修改方式**：通过 `PUT /api/v1/system/admin/settings/upload_allowed_types` 接口更新。

### 6.2 魔术字节验证

仅依赖 Content-Type 头不安全，需验证文件实际内容：

```rust
fn validate_file_magic(content: &[u8], claimed_type: &str) -> bool {
    match claimed_type {
        "image/png" => content.starts_with(&[0x89, 0x50, 0x4E, 0x47]),
        "image/jpeg" => content.starts_with(&[0xFF, 0xD8, 0xFF]),
        "image/gif" => content.starts_with(b"GIF87a") || content.starts_with(b"GIF89a"),
        "application/pdf" => content.starts_with(b"%PDF"),
        "application/zip" => content.starts_with(&[0x50, 0x4B, 0x03, 0x04]),
        _ => false,
    }
}
```

### 6.3 文件大小限制

```toml
[upload]
max_size = 10485760  # 10 MB
```

### 6.4 文件名处理

```rust
fn sanitize_filename(original: &str) -> String {
    // 1. 生成随机 UUID 作为存储文件名
    let stored_name = format!("{}_{}", Uuid::new_v4(), sanitized);

    // 2. 保留原始文件名供显示，但不用于存储路径
    // 3. 防止路径遍历攻击
    stored_name.replace("..", "").replace("/", "").replace("\\", "")
}
```

### 6.5 存储路径

```
uploads/
├── 2026/
│   └── 01/
│       └── 24/
│           └── {uuid}_{sanitized_name}
```

---

## 七、权限控制

### 7.1 三层权限架构

```
┌─────────────────────────────────────┐
│         Layer 1: JWT 验证            │
│    验证 Token 有效性，提取用户信息      │
└─────────────────┬───────────────────┘
                  │
┌─────────────────▼───────────────────┐
│      Layer 2: 系统角色验证            │
│    检查 UserRole (Admin/Teacher/User) │
└─────────────────┬───────────────────┘
                  │
┌─────────────────▼───────────────────┐
│      Layer 3: 班级角色验证            │
│   检查 ClassUserRole (Teacher/Rep/Student) │
└─────────────────────────────────────┘
```

### 7.2 中间件链配置

```rust
// 示例：班级成员管理
web::resource("/{class_id}/students")
    .wrap(RequireJWT)                              // 第一层
    .wrap(RequireRole::new_any(UserRole::user_roles()))  // 第二层
    .wrap(RequireClassRole::new_any(              // 第三层
        ClassUserRole::class_representative_roles()
    ))
```

### 7.3 Admin 特权

Admin 用户自动绕过班级角色检查：

```rust
// RequireClassRole 中间件
if user.role == UserRole::Admin {
    return Ok(next.call(req).await);  // 直接放行
}
```

---

## 八、SQL 注入防护

### 8.1 参数化查询

使用 SeaORM 自动参数化所有查询：

```rust
// 安全 - 使用参数化查询
User::find()
    .filter(user::Column::Username.eq(username))
    .one(db)
    .await?;

// 危险 - 永远不要这样做
// db.execute_unprepared(&format!("SELECT * FROM users WHERE username = '{}'", username))
```

### 8.2 审计日志

记录所有敏感操作：

```rust
tracing::info!(
    user_id = %user.id,
    action = "create_homework",
    class_id = %class_id,
    "User created homework"
);
```

---

## 九、XSS 防护

### 9.1 响应 Content-Type

所有 API 响应固定为 JSON：

```rust
HttpResponse::Ok()
    .content_type("application/json; charset=utf-8")
    .json(response)
```

### 9.2 输出编码

存储时保持原始内容，前端渲染时进行编码。

### 9.3 CSP 头（前端配置）

```
Content-Security-Policy: default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline';
```

---

## 十、CSRF 防护

### 10.1 SameSite Cookie

Refresh Token 使用 `SameSite=Strict`，防止跨站请求携带：

```rust
Cookie::build(("refresh_token", token))
    .same_site(SameSite::Strict)
```

### 10.2 Token 验证

所有状态修改操作都需要 Access Token（不在 Cookie 中），天然防止 CSRF。

---

## 十一、日志安全

### 11.1 敏感信息过滤

不记录以下信息：
- 密码（明文或哈希）
- JWT Token 完整内容
- 个人隐私数据

```rust
// 错误示例
tracing::info!("User login: password={}", password);

// 正确示例
tracing::info!(username = %username, "User login attempt");
```

### 11.2 结构化日志

使用 tracing 输出结构化日志：

```rust
tracing::info!(
    user_id = %user.id,
    ip = %req.connection_info().realip_remote_addr().unwrap_or("unknown"),
    user_agent = %req.headers().get("User-Agent").map(|v| v.to_str().unwrap_or("")).unwrap_or(""),
    "API request"
);
```

### 11.3 生产环境日志

```toml
[app]
log_level = "warn"  # 生产环境只记录警告及以上
log_format = "json" # JSON 格式便于收集
```

---

## 十二、安全检查清单

### 12.1 部署前检查

- [ ] JWT_SECRET 已从默认值更改
- [ ] CORS allowed_origins 已配置为前端域名
- [ ] 数据库连接使用 TLS（生产环境）
- [ ] Redis 连接设置密码（如使用）
- [ ] 日志级别设置为 warn 或 error
- [ ] 文件上传目录权限正确（755）
- [ ] HTTPS 已启用

### 12.2 定期审计

- [ ] 每周：检查失败登录日志
- [ ] 每月：审计用户权限
- [ ] 每季度：轮换 JWT 密钥
- [ ] 每半年：依赖安全更新

---

## 十三、事件响应

### 13.1 可疑活动检测

监控以下指标：
- 单 IP 短时间大量登录失败
- 单用户短时间多次刷新 Token
- 异常时间段的管理员操作

### 13.2 应急措施

1. **Token 泄露**：
   - 立即轮换 JWT 密钥
   - 强制所有用户重新登录

2. **账户被盗**：
   - 封禁账户 (`status = banned`)
   - 清除所有 Session

3. **数据泄露**：
   - 通知受影响用户
   - 强制密码重置

---

## 十四、更新日志

| 版本 | 日期 | 变更内容 |
|------|------|----------|
| v2.1 | 2026-01-26 | 更新文件类型白名单说明（改为动态配置） |
| v2.0 | 2026-01-24 | 添加速率限制、文件魔术字节验证、安全检查清单 |
| v1.0 | 2025-01-23 | 初始版本 |
