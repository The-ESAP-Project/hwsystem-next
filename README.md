# 作业管理系统后端

基于 Rust + Actix Web 的教育管理平台后端服务。

## 技术栈

- Rust 1.92+
- Actix Web 4.x
- PostgreSQL 14+
- JWT 认证

## 快速开始

```bash
# 克隆项目
git clone https://github.com/The-ESAP-Project/rust-hwsystem-next.git
cd rust-hwsystem-next

# 配置
cp config.example.toml config.toml
# 编辑 config.toml 填入数据库连接等信息

# 初始化数据库
createdb hwsystem
cargo run --bin migrate

# 运行
cargo run
```

服务启动后访问 `http://localhost:8080`

## 常用命令

```bash
cargo build              # 构建
cargo run                # 开发运行
cargo build --release    # 生产构建
cargo test               # 测试
cargo clippy             # 代码检查
```

## 权限体系

| 角色   | 权限                           |
|--------|--------------------------------|
| 学生   | 查看/提交作业、查看成绩        |
| 课代表 | 学生权限 + 统计、提醒          |
| 教师   | 全部权限，管理用户和作业       |

## 部署

```bash
# Docker
docker-compose up -d

# 或手动构建
docker build -t hwsystem-backend .
docker run -p 8080:8080 -d hwsystem-backend
```

## 文档

- [配置说明](CONFIG.md)
- [API 文档](docs/API.md)
- [贡献指南](CONTRIBUTING.md)

## 许可证

[MIT](LICENSE)
