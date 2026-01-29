# 前后端代码问题审计报告

## 调查概述
基于对项目代码的深度审计，发现以下问题。按严重程度分类。

---

## 🔴 严重问题 (需要立即修复)

### 1. 安全问题

#### 1.1 ~~CORS 配置过于宽松~~ ✅ 已修复
- **文件**: `src/main.rs:85-113`
- ~~**问题**: 生产环境允许任意来源、任意方法、任意头部~~
- ✅ 已改为从配置文件 `config.cors.*` 读取，支持精确控制

#### 1.2 ~~Fallback Token 机制不安全~~ ✅ 已修复
- **文件**: `src/models/users/entities.rs:144-154`
- ~~**问题**: JWT 生成失败时使用可预测的 fallback token~~
- ✅ 已移除 fallback 机制，改为直接返回 `Result<TokenPair, String>`

#### 1.3 ~~前端 Token 存储在 localStorage~~ ✅ 已修复
- **文件**: `frontend/src/stores/useUserStore.ts`
- ~~**风险**: XSS 攻击可窃取 token~~
- ✅ Access Token 改为存储在 Zustand 内存 store 中
- ✅ 页面刷新后通过 httpOnly cookie 的 refresh token 自动恢复
- ✅ 修改了 `api.ts`、`providers.tsx`、`useWebSocket.ts`、`fileService.ts` 统一从 store 获取 token

#### 1.4 Token 暴露在 WebSocket URL
- **文件**: `frontend/src/hooks/useWebSocket.ts:126-129`
- **风险**: Token 可能被记录在服务器日志、代理日志或浏览器历史中

#### 1.5 文件下载缺少细粒度权限检查
- **文件**: `src/services/files/download.rs:11-16`
- **问题**: 只验证 token，没有验证用户是否有权访问该文件

### 2. API 实现问题

#### 2.1 ~~前端调用了未实现的文件删除 API~~ ✅ 已实现
- **路由**: `DELETE /api/v1/files/{file_token}`
- **权限**: 只有上传者可以删除自己的文件
- **逻辑**: 引用计数为 0 时同时删除物理文件

---

## 🟠 中等问题

### 3. 性能问题

#### 3.1 N+1 查询问题
- **教师统计**: `src/storage/sea_orm_storage/users.rs` - ~~循环中逐个查询班级学生数~~ ✅ 已改为 GROUP BY 批量查询
- **作业列表创建者查询**: `src/storage/sea_orm_storage/homeworks.rs` - ~~循环逐个查询用户~~ ✅ 已改为 IN 批量查询
- **作业统计班级学生数**: `src/storage/sea_orm_storage/homeworks.rs` - ~~循环逐个 COUNT~~ ✅ 已改为 GROUP BY 批量查询（2 处）
- **提交详情附件查询**: `src/storage/sea_orm_storage/submissions.rs` - ~~循环逐个查询文件~~ ✅ 已改为 IN 批量查询
- **提交概览**: `src/storage/sea_orm_storage/submissions.rs:443-448` - ~~先查询所有提交再内存聚合~~ ✅ 已改为 GROUP BY 数据库聚合 + 数据库分页

#### 3.2 文件下载全量加载
- **文件**: `src/services/files/download.rs`
- ~~**问题**: 整个文件读入内存，大文件会 OOM~~
- ✅ 已改用 `actix_files::NamedFile` 流式传输，支持 Range 请求（断点续传）

#### 3.3 前端大列表一次性加载
- **文件**: `frontend/src/features/homework/components/HomeworkListCard.tsx`
- ~~**问题**: 一次性加载 200 条数据后前端分页~~
- ✅ `TeacherHomeworksPage` 班级列表 page_size 100→20，`useAllClassesHomeworks` page_size 100→50

### 4. 错误处理问题

#### 4.1 ~~expect/panic 使用不当~~ ✅ 已修复
~~多处使用 `expect()` 可能导致服务崩溃：~~
- ~~`src/middlewares/require_jwt.rs:122, 142`~~
- ~~`src/routes/websocket.rs:52, 65`~~
- ~~`src/cache/object_cache/redis.rs:28`~~
- ✅ 中间件/路由改用 `ok_or_else` 返回错误
- ✅ Service 层 `get_storage` 返回 `Result<Arc<dyn Storage>, actix_web::Error>`

#### 4.2 ~~前端空 catch 块吞掉错误~~ ✅ 已修复
- ~~**文件**: `frontend/src/stores/useUserStore.ts:72-74, 111-112`~~
- ~~**问题**: 完全吞掉错误，连日志都没有~~
- ✅ 创建统一的 `src/lib/logger.ts` 模块
- ✅ 所有 catch 块添加 `logger.error` 或 `logger.warn`

### 5. 类型安全问题

#### 5.1 ~~过度使用类型断言~~ ✅ 已修复
- **文件**: `frontend/src/features/auth/services/auth.ts` 等多个文件
- ~~**问题**: 大量使用 `as unknown as Stringify<T>` 断言~~
- ✅ 移除了 `Stringify<T>` 类型工具和所有相关断言
- ✅ 直接使用 ts-rs 生成的类型（后端已将 i64 序列化为 string）

#### 5.2 ~~bigint 类型转换精度丢失风险~~ ✅ 已修复
- **文件**: `frontend/src/features/homework/services/homeworkService.ts:129`
- ~~**问题**: ts-rs 将 i64 生成为 bigint，Number 转换会丢精度~~
- ✅ 后端所有 i64/u64 字段现在序列化为 string，前端类型全部为 string
- ✅ 使用 `#[serde(serialize_with = "serialize_i64_as_string")]` + `#[ts(type = "string")]`

### 6. 文档不一致

#### 6.1 ~~API 文档与实现不同步~~ ✅ 已修复
- ~~登出 API: 文档标注未实现 (`docs/API.md:243`)，实际已实现~~
- ~~分页参数: 文档用 `size`，实现用 `page_size`~~
- ✅ 更新文档版本至 v3.0
- ✅ 移除登出 API 的"未实现"标记
- ✅ 统一分页参数为 `page_size`
- ✅ 更新文件删除 API 文档（路径改为 `/files/{token}`，添加完整说明）

### 7. 用户体验问题

#### 7.1 ~~批量删除无确认对话框~~ ✅ 已修复
- **文件**: `frontend/src/features/admin/pages/UserListPage.tsx`
- ✅ 添加了 AlertDialog 确认对话框，显示要删除的用户数量

#### 7.2 ~~加载状态不一致~~ ✅ 已修复
- **文件**: `frontend/src/app/router.tsx`
- ~~硬编码 "Loading..." 未国际化~~
- ✅ 已改用 `t("common.loading")` 国际化

---

## 🟡 低优先级问题

### 8. 代码质量

#### 8.1 重复代码
- ~~Storage 获取模式在所有 Service 中重复~~ ✅ 已提取 `StorageProvider` trait
- ~~防抖搜索模式在多个组件中重复 (`ClassListPage`, `UserListPage`, `HomeworkListCard`)~~ ✅ 已创建 `useDebouncedSearch` hook
- ~~布局组件重复 (`NotificationLayout`, `SettingsLayout`)~~ ✅ 已创建 `useRoleNavItems` hook 和 `RoleBasedLayout` 组件

#### 8.2 组件过大 - ✅ 已完成
- `UserListPage.tsx`: ~~485 行~~ → 404 行 (提取了 `useBatchSelection` hook 和 `UserListFilters` 组件)
- `HomeworkDetailPage.tsx`: ~~451 行~~ → 275 行 ✅ (提取了 `useHomeworkStatus` hook、`HomeworkInfoCard`、`MySubmissionCard`、`SubmissionManagementCard` 组件)
- `HomeworkListCard.tsx`: ~~260 行~~ → 158 行 ✅ (提取了 `useHomeworkFilters` hook、`HomeworkListToolbar` 和 `HomeworkStatusTabs` 组件)

#### 8.3 ~~魔法数字~~ ✅ 已修复
- ~~`frontend/src/lib/api.ts:47`: `timeout: 10000`~~
- ~~`frontend/src/features/class/pages/ClassListPage.tsx:44`: `pageSize = 12`~~
- ✅ 已创建 `frontend/src/lib/constants.ts` 集中管理常量

### 9. 可访问性问题

#### 9.1 ~~缺少 ARIA 标签~~ ✅ 已修复
- ~~**文件**: `frontend/src/features/auth/pages/LoginPage.tsx:116-128`~~
- ~~密码可见性切换按钮无 `aria-label`~~
- ✅ 已添加 `aria-label` 属性和 i18n 键

#### 9.2 ~~颜色对比度依赖~~ ✅ 非问题
- **文件**: `frontend/src/features/admin/pages/UserListPage.tsx:53-65`
- ~~角色和状态仅通过颜色区分~~
- ✅ 经审查，代码已正确使用文字标签配合颜色（符合 WCAG 1.4.1）

### 10. 配置问题

#### 10.1 ~~JWT Secret 硬编码默认值~~ ✅ 已修复
- **文件**: `config.toml:37`
- ~~`secret = "default_secret_key"` 未在黑名单中~~
- ✅ 已在 `src/config/impl.rs` 的 `validate_security()` 黑名单中添加 `"default_secret_key"`

#### 10.2 ~~Argon2 配置未使用~~ ✅ 已修复
- ~~**配置**: `config.toml` 中定义了 Argon2 配置~~
- ~~**实际**: `src/services/auth/register.rs:115-122` 使用 `Argon2::default()`~~
- ✅ 已删除局部 `hash_password` 函数，改用 `crate::utils::password::hash_password`

---

## 问题统计

| 严重程度 | 总数 | 已修复 | 剩余 |
|---------|------|--------|------|
| 🔴 严重 | 6 | 4 | 2 |
| 🟠 中等 | 10 | 10 | 0 |
| 🟡 低 | 8 | 8 | 0 |

---

## 建议修复顺序

1. **第一优先级 - 安全修复**
   - [x] 修复 CORS 配置
   - [x] 移除 fallback token 机制
   - [x] 修复 JWT Secret 黑名单验证
   - [x] 修复 Argon2 配置未使用
   - [x] Token 存储方案改进（Access Token 存内存，Refresh Token 用 httpOnly cookie）
   - [ ] 修复文件下载权限检查

2. **第二优先级 - 功能修复**
   - [x] 实现文件删除 API
   - [x] 更新 API 文档

3. **第三优先级 - 性能优化**
   - [x] 修复 N+1 查询（6 处全部修复）
   - [x] 实现流式文件下载（NamedFile + Range 支持）
   - [x] 前端分页参数优化（page_size 调整）

4. **第四优先级 - 代码质量**
   - [x] 提取防抖搜索重复代码（`useDebouncedSearch` hook）
   - [x] 提取魔法数字为常量（`constants.ts`）
   - [x] 添加 ARIA 可访问性标签
   - [x] 提取 Storage 获取重复代码（`StorageProvider` trait）
   - [x] 拆分过大组件
     - [x] 提取 `useBatchSelection` 通用 hook
     - [x] 提取 `useHomeworkFilters` hook
     - [x] 提取 `useHomeworkStatus` hook
     - [x] 提取 `UserListFilters` 组件
     - [x] 提取 `HomeworkListToolbar` 组件
     - [x] 提取 `HomeworkStatusTabs` 组件
     - [x] 提取 `HomeworkInfoCard` 组件
     - [x] 提取 `MySubmissionCard` 组件
     - [x] 提取 `SubmissionManagementCard` 组件
   - [ ] 布局组件重复

---

## 验证方式

修复后需要验证：
1. 运行所有后端测试: `cargo test`
2. 运行前端测试: `cd frontend && npm test`
3. 手动测试文件上传/下载/删除流程
4. 使用 OWASP ZAP 或类似工具进行安全扫描
5. 检查生产环境 CORS 配置是否正确
