# 待修复问题清单

> 更新时间：2026-01-26
> 调查者：AptS:1548

---

## 实施进度

### 阶段 1：后端 API 扩展 ✅ 已完成

#### 1.1 新增跨班级作业列表 API ✅

**端点**：`GET /api/v1/homeworks/all`

**请求参数**：
```rust
pub struct AllHomeworksQuery {
    pub page: Option<i64>,
    pub size: Option<i64>,
    pub status: Option<HomeworkUserStatus>,  // pending/submitted/graded
    pub deadline_filter: Option<DeadlineFilter>,  // active/expired/all
    pub search: Option<String>,
    pub include_stats: Option<bool>,
}
```

**响应**：
```rust
pub struct AllHomeworksResponse {
    pub items: Vec<HomeworkListItem>,
    pub pagination: PaginationInfo,
    pub server_time: String,  // ISO 8601，解决时区问题
}
```

**已完成文件**：
- [x] `src/models/homeworks/entities.rs` - 添加 `HomeworkUserStatus`、`DeadlineFilter` 枚举
- [x] `src/models/homeworks/requests.rs` - 添加 `AllHomeworksQuery`
- [x] `src/models/homeworks/responses.rs` - 添加 `AllHomeworksResponse`
- [x] `src/storage/mod.rs` - 添加 trait 方法
- [x] `src/storage/sea_orm_storage/homeworks.rs` - 实现查询逻辑
- [x] `src/services/homeworks/list_all.rs` - 添加 service 方法
- [x] `src/routes/homeworks.rs` - 添加路由

#### 1.2 扩展用户统计 API ✅

**端点**：`GET /api/v1/users/me/stats`

**响应**：
```rust
pub struct UserStatsResponse {
    pub class_count: i64,           // 班级数量
    pub total_students: i64,        // 学生总数（教师视角）
    pub homework_pending: i64,      // 待完成作业
    pub homework_submitted: i64,    // 已提交作业
    pub homework_graded: i64,       // 已批改作业
    pub pending_review: i64,        // 待批改数（教师视角）
    pub server_time: String,
}
```

**已完成文件**：
- [x] `src/models/users/responses.rs` - 添加 `UserStatsResponse`
- [x] `src/storage/mod.rs` - 添加 trait 方法
- [x] `src/storage/sea_orm_storage/users.rs` - 实现统计逻辑
- [x] `src/services/users/stats.rs` - 添加 service 方法
- [x] `src/routes/users.rs` - 添加路由

---

### 阶段 2：前端工具层 ✅ 已完成

#### 2.1 状态配置文件 ✅

**文件**：`frontend/src/constants/status.ts`

#### 2.2 截止日期工具 ✅

**文件**：`frontend/src/utils/deadline.ts`

#### 2.3 扩展权限 Hook ✅

**文件**：`frontend/src/features/auth/hooks/usePermission.ts`

新增 `useClassPermission(classRole)` hook

---

### 阶段 3：前端 Hook 层 ✅ 已完成

#### 3.1 新增 Hooks ✅

**文件**：`frontend/src/features/homework/hooks/useHomework.ts`
- [x] `useAllHomeworks` hook

**文件**：`frontend/src/features/user/hooks/useUser.ts`
- [x] `useUserStats` hook

#### 3.2 新增 Service 方法 ✅

**文件**：`frontend/src/features/homework/services/homeworkService.ts`
- [x] `listAll` 方法

**文件**：`frontend/src/features/user/services/userService.ts`
- [x] `getMyStats` 方法

---

### 阶段 4：页面迁移 🔄 部分完成

| 页面 | 文件 | 状态 |
|------|------|------|
| 学生仪表盘 | `UserDashboardPage.tsx` | ✅ 已完成 |
| 学生作业列表 | `MyHomeworksPage.tsx` | ⏳ 待迁移 |
| 教师仪表盘 | `TeacherDashboardPage.tsx` | ⏳ 待迁移 |
| 教师作业列表 | `TeacherHomeworksPage.tsx` | ⏳ 待迁移 |
| 作业列表卡片 | `HomeworkListCard.tsx` | ⏳ 待迁移 |

---

### 阶段 5：清理 ⏳ 待完成

- [ ] 删除 `useAllClassesHomeworks` hook（迁移完成后）
- [ ] 移除页面中的硬编码状态字符串
- [ ] 移除前端截止日期判断逻辑，使用 `getDeadlineInfo`
- [ ] 统一权限判断，使用 `useClassPermission`

---

## 效果对比

| 场景 | 现在 | 改后 |
|------|------|------|
| 教师查看所有作业 | N 次请求（N=班级数） | 1 次请求 |
| 学生仪表盘统计 | 2+ 次请求 + 前端计算 | 1 次请求 |
| 截止日期判断 | 本地时间（时区问题） | 服务器时间 |
| 状态枚举 | 硬编码字符串 | TS-RS 生成类型 |
| 权限判断 | 分散在各文件 | 集中管理 |

---

## 已修复问题

### 2026-01-26：分页导致统计不准确

- [x] `NotificationListPage.tsx` - 使用 `useUnreadCount()` hook
- [x] `UserDashboardPage.tsx` - 统计卡片使用 `useUserStats()` API（已更新）
- [x] `MyHomeworksPage.tsx` - Tab 标签数字使用 `useMyHomeworkStats()` API
- [x] `TeacherDashboardPage.tsx` - 待批改数量使用 `useTeacherHomeworkStats()` API

**新增后端 API**：
- `GET /api/v1/homeworks/my/stats` - 学生作业统计
- `GET /api/v1/homeworks/teacher/stats` - 教师作业统计
- `GET /api/v1/homeworks/all` - 跨班级作业列表（新增）
- `GET /api/v1/users/me/stats` - 用户综合统计（新增）

---

## 待修复的代码问题

> 2026-01-26 文档审计发现

### 高优先级

| 问题 | 位置 | 说明 |
|------|------|------|
| ~~评分查询端点缺失~~ | `src/services/grades/list.rs:75` | ✅ 已修复 - 新增 `GET /submissions/{id}/grade` 路由 |
| ~~登出功能未实现~~ | `src/routes/auth.rs` | ✅ 已修复 - 新增 `POST /auth/logout` 路由 |

### 中优先级

| 问题 | 位置 | 说明 |
|------|------|------|
| ~~魔术字节验证未实现~~ | `src/services/files/upload.rs` | ✅ 已修复 - 新增 `src/utils/file_magic.rs` 验证模块 |
| ~~Argon2 参数未配置~~ | `src/utils/password.rs:7` | ✅ 已修复 - 新增 `Argon2Config` 配置结构体 |

### 低优先级

| 问题 | 位置 | 说明 |
|------|------|------|
| grades 表缺少 CHECK 约束 | migration | 文档说有 `score >= 0` 约束，但 migration 未定义 |
| system_settings 表缺少外键 | migration | `updated_by` 字段未定义外键约束 |
