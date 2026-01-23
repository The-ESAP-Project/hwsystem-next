# 数据库设计文档

> 版本：v2.0
> 更新日期：2026-01-24
> 数据库：SQLite（开发）/ PostgreSQL（生产）

---

## 一、ER 图

```
┌─────────┐       ┌─────────────┐       ┌─────────┐
│  users  │──1:N──│ class_users │──N:1──│ classes │
└────┬────┘       └─────────────┘       └────┬────┘
     │                                       │
     │ 1:N                                   │ 1:N
     ▼                                       ▼
┌─────────────┐                        ┌───────────┐
│ submissions │──N:1───────────────────│ homeworks │
└──────┬──────┘                        └─────┬─────┘
       │                                     │
       │ 1:1                                 │ 1:N
       ▼                                     ▼
┌─────────┐                           ┌──────────────┐
│ grades  │                           │homework_files│
└─────────┘                           └──────────────┘

┌─────────┐       ┌──────────────────┐
│  files  │──1:N──│ submission_files │
└─────────┘       └──────────────────┘

┌───────────────┐
│ notifications │──N:1── users
└───────────────┘
```

---

## 二、表清单

| 序号 | 表名 | 说明 | 状态 |
|------|------|------|------|
| 1 | users | 用户表 | 已存在 |
| 2 | classes | 班级表 | 已存在 |
| 3 | class_users | 班级成员关联表 | 已存在（需修改） |
| 4 | homeworks | 作业表 | 已存在（需修改） |
| 5 | submissions | 提交表 | 已存在（需重构） |
| 6 | grades | 评分表 | 已存在（需重构） |
| 7 | files | 文件表 | 已存在 |
| 8 | homework_files | 作业附件关联表 | **新增** |
| 9 | submission_files | 提交附件关联表 | **新增** |
| 10 | notifications | 通知表 | **新增** |

---

## 三、表结构定义

### 3.1 users（用户表）

存储系统用户信息。

```sql
CREATE TABLE users (
    id              TEXT PRIMARY KEY,           -- UUID v4
    username        TEXT NOT NULL UNIQUE,       -- 用户名，唯一
    email           TEXT NOT NULL UNIQUE,       -- 邮箱，唯一
    password_hash   TEXT NOT NULL,              -- Argon2 哈希后的密码
    display_name    TEXT,                       -- 显示名称，可选
    role            TEXT NOT NULL DEFAULT 'user',  -- 系统角色
    status          TEXT NOT NULL DEFAULT 'active', -- 用户状态
    created_at      INTEGER NOT NULL,           -- 创建时间（Unix timestamp）
    updated_at      INTEGER NOT NULL            -- 更新时间（Unix timestamp）
);

-- 索引
CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_role ON users(role);
CREATE INDEX idx_users_status ON users(status);
```

**字段说明**：

| 字段 | 类型 | 约束 | 说明 |
|------|------|------|------|
| id | TEXT | PK | UUID v4 格式 |
| username | TEXT | UNIQUE, NOT NULL | 用户名，3-32字符 |
| email | TEXT | UNIQUE, NOT NULL | 邮箱地址 |
| password_hash | TEXT | NOT NULL | Argon2 哈希 |
| display_name | TEXT | - | 显示名称 |
| role | TEXT | NOT NULL | `user` / `teacher` / `admin` |
| status | TEXT | NOT NULL | `active` / `suspended` / `banned` |
| created_at | INTEGER | NOT NULL | Unix 时间戳 |
| updated_at | INTEGER | NOT NULL | Unix 时间戳 |

### 3.2 classes（班级表）

存储班级信息。

```sql
CREATE TABLE classes (
    id              TEXT PRIMARY KEY,           -- UUID v4
    name            TEXT NOT NULL,              -- 班级名称
    description     TEXT,                       -- 班级描述
    teacher_id      TEXT NOT NULL,              -- 创建者/班主任
    invite_code     TEXT NOT NULL UNIQUE,       -- 6位邀请码
    created_at      INTEGER NOT NULL,
    updated_at      INTEGER NOT NULL,

    FOREIGN KEY (teacher_id) REFERENCES users(id) ON DELETE CASCADE
);

-- 索引
CREATE INDEX idx_classes_teacher_id ON classes(teacher_id);
CREATE UNIQUE INDEX idx_classes_invite_code ON classes(invite_code);
```

**外键行为**：
- `teacher_id` → 删除用户时级联删除其创建的班级

### 3.3 class_users（班级成员表）

用户与班级的多对多关系表。

```sql
CREATE TABLE class_users (
    id              TEXT PRIMARY KEY,           -- UUID v4
    class_id        TEXT NOT NULL,              -- 班级ID
    user_id         TEXT NOT NULL,              -- 用户ID
    role            TEXT NOT NULL DEFAULT 'student', -- 班级角色
    joined_at       INTEGER NOT NULL,           -- 加入时间

    FOREIGN KEY (class_id) REFERENCES classes(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,

    UNIQUE(class_id, user_id)                   -- 每个用户在每个班级只能有一条记录
);

-- 索引
CREATE INDEX idx_class_users_class_id ON class_users(class_id);
CREATE INDEX idx_class_users_user_id ON class_users(user_id);
```

**字段说明**：

| 字段 | 类型 | 约束 | 说明 |
|------|------|------|------|
| role | TEXT | NOT NULL | `student` / `class_representative` / `teacher` |

**关键约束**：
- `UNIQUE(class_id, user_id)` - 防止重复加入

### 3.4 homeworks（作业表）

存储作业信息。

```sql
CREATE TABLE homeworks (
    id              TEXT PRIMARY KEY,           -- UUID v4
    class_id        TEXT NOT NULL,              -- 所属班级
    title           TEXT NOT NULL,              -- 作业标题
    description     TEXT,                       -- 作业描述（支持 Markdown）
    max_score       REAL NOT NULL DEFAULT 100.0,-- 最高分
    deadline        INTEGER,                    -- 截止时间（Unix timestamp），可选
    allow_late      BOOLEAN NOT NULL DEFAULT FALSE, -- 是否允许迟交
    created_by      TEXT NOT NULL,              -- 创建者（教师）
    created_at      INTEGER NOT NULL,
    updated_at      INTEGER NOT NULL,

    FOREIGN KEY (class_id) REFERENCES classes(id) ON DELETE CASCADE,
    FOREIGN KEY (created_by) REFERENCES users(id) ON DELETE CASCADE
);

-- 索引
CREATE INDEX idx_homeworks_class_id ON homeworks(class_id);
CREATE INDEX idx_homeworks_created_by ON homeworks(created_by);
CREATE INDEX idx_homeworks_deadline ON homeworks(deadline);
```

**外键行为**：
- `class_id` → 删除班级时级联删除所有作业
- `created_by` → 删除创建者时级联删除作业

### 3.5 submissions（提交表）⚠️ 重构

存储学生作业提交记录，支持版本控制。

```sql
CREATE TABLE submissions (
    id              TEXT PRIMARY KEY,           -- UUID v4
    homework_id     TEXT NOT NULL,              -- 所属作业
    creator_id      TEXT NOT NULL,              -- 提交者（学生）
    version         INTEGER NOT NULL DEFAULT 1, -- 版本号，从1开始递增
    content         TEXT,                       -- 提交内容（文本/Markdown）
    status          TEXT NOT NULL DEFAULT 'pending', -- 提交状态
    is_late         BOOLEAN NOT NULL DEFAULT FALSE,  -- 是否迟交
    submitted_at    INTEGER NOT NULL,           -- 提交时间

    FOREIGN KEY (homework_id) REFERENCES homeworks(id) ON DELETE CASCADE,
    FOREIGN KEY (creator_id) REFERENCES users(id) ON DELETE CASCADE,

    UNIQUE(homework_id, creator_id, version)    -- 同一学生同一作业的每个版本唯一
);

-- 索引
CREATE INDEX idx_submissions_homework_id ON submissions(homework_id);
CREATE INDEX idx_submissions_creator_id ON submissions(creator_id);
CREATE INDEX idx_submissions_status ON submissions(status);
CREATE INDEX idx_submissions_hw_creator ON submissions(homework_id, creator_id);
```

**字段说明**：

| 字段 | 类型 | 约束 | 说明 |
|------|------|------|------|
| version | INTEGER | NOT NULL | 版本号，同一学生同一作业递增 |
| status | TEXT | NOT NULL | `pending` / `graded` / `late` |
| is_late | BOOLEAN | NOT NULL | 迟交标记 |

**关键约束**：
- `UNIQUE(homework_id, creator_id, version)` - 版本唯一性

**状态转换**：
```
pending → graded（被评分后）
pending → late（提交时已超过截止时间）
late → graded（迟交被评分后，仍保留 is_late=true）
```

### 3.6 grades（评分表）⚠️ 重构

存储教师评分记录。

```sql
CREATE TABLE grades (
    id              TEXT PRIMARY KEY,           -- UUID v4
    submission_id   TEXT NOT NULL UNIQUE,       -- 所属提交（一对一）
    grader_id       TEXT NOT NULL,              -- 评分者（教师）
    score           REAL NOT NULL,              -- 分数
    comment         TEXT,                       -- 评语
    graded_at       INTEGER NOT NULL,           -- 首次评分时间
    updated_at      INTEGER NOT NULL,           -- 最后修改时间

    FOREIGN KEY (submission_id) REFERENCES submissions(id) ON DELETE CASCADE,
    FOREIGN KEY (grader_id) REFERENCES users(id) ON DELETE SET NULL,

    CHECK (score >= 0)                          -- 分数非负
);

-- 索引
CREATE INDEX idx_grades_submission_id ON grades(submission_id);
CREATE INDEX idx_grades_grader_id ON grades(grader_id);
```

**关键约束**：
- `UNIQUE(submission_id)` - 一个提交只能有一个评分
- `CHECK (score >= 0)` - 分数非负
- `grader_id ON DELETE SET NULL` - 删除评分者时保留评分记录

**业务约束**（应用层实现）：
- `score <= homework.max_score` - 分数不能超过满分

### 3.7 files（文件表）

存储上传文件的元数据。

```sql
CREATE TABLE files (
    id              TEXT PRIMARY KEY,           -- UUID v4
    user_id         TEXT,                       -- 上传者
    original_name   TEXT NOT NULL,              -- 原始文件名
    stored_name     TEXT NOT NULL,              -- 存储文件名（防冲突）
    file_type       TEXT NOT NULL,              -- MIME 类型
    file_size       INTEGER NOT NULL,           -- 文件大小（字节）
    file_path       TEXT NOT NULL,              -- 存储路径
    download_token  TEXT NOT NULL UNIQUE,       -- 下载令牌
    citation_count  INTEGER NOT NULL DEFAULT 0, -- 引用计数
    created_at      INTEGER NOT NULL,

    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL
);

-- 索引
CREATE INDEX idx_files_user_id ON files(user_id);
CREATE UNIQUE INDEX idx_files_download_token ON files(download_token);
```

### 3.8 homework_files（作业附件关联表）🆕

作业与文件的多对多关系表。

```sql
CREATE TABLE homework_files (
    homework_id     TEXT NOT NULL,
    file_id         TEXT NOT NULL,

    PRIMARY KEY (homework_id, file_id),
    FOREIGN KEY (homework_id) REFERENCES homeworks(id) ON DELETE CASCADE,
    FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE
);
```

**外键行为**：
- 删除作业或文件时自动删除关联记录

### 3.9 submission_files（提交附件关联表）🆕

提交与文件的多对多关系表。

```sql
CREATE TABLE submission_files (
    submission_id   TEXT NOT NULL,
    file_id         TEXT NOT NULL,

    PRIMARY KEY (submission_id, file_id),
    FOREIGN KEY (submission_id) REFERENCES submissions(id) ON DELETE CASCADE,
    FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE
);
```

### 3.10 notifications（通知表）🆕

存储站内通知消息。

```sql
CREATE TABLE notifications (
    id              TEXT PRIMARY KEY,           -- UUID v4
    user_id         TEXT NOT NULL,              -- 接收者
    type            TEXT NOT NULL,              -- 通知类型
    title           TEXT NOT NULL,              -- 通知标题
    content         TEXT,                       -- 通知内容
    reference_type  TEXT,                       -- 关联实体类型
    reference_id    TEXT,                       -- 关联实体ID
    is_read         BOOLEAN NOT NULL DEFAULT FALSE, -- 是否已读
    created_at      INTEGER NOT NULL,

    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- 索引
CREATE INDEX idx_notifications_user_id ON notifications(user_id);
CREATE INDEX idx_notifications_is_read ON notifications(user_id, is_read);
CREATE INDEX idx_notifications_created_at ON notifications(created_at DESC);
```

**字段说明**：

| 字段 | 类型 | 说明 |
|------|------|------|
| type | TEXT | 通知类型枚举（见下表） |
| reference_type | TEXT | `homework` / `submission` / `grade` / `class` |
| reference_id | TEXT | 关联实体的 UUID |

**通知类型枚举**：

| 类型 | 说明 | reference_type |
|------|------|----------------|
| homework_created | 新作业发布 | homework |
| homework_updated | 作业更新 | homework |
| homework_deadline | 作业即将截止 | homework |
| submission_received | 收到新提交 | submission |
| grade_received | 收到评分 | grade |
| grade_updated | 评分修改 | grade |
| class_joined | 加入班级 | class |
| class_role_changed | 班级角色变更 | class |

---

## 四、索引设计

### 4.1 索引清单

| 表 | 索引名 | 字段 | 类型 | 用途 |
|------|--------|------|------|------|
| users | idx_users_username | username | UNIQUE | 用户名查询 |
| users | idx_users_email | email | UNIQUE | 邮箱查询 |
| users | idx_users_role | role | NORMAL | 按角色筛选 |
| users | idx_users_status | status | NORMAL | 按状态筛选 |
| classes | idx_classes_teacher_id | teacher_id | NORMAL | 查询教师的班级 |
| classes | idx_classes_invite_code | invite_code | UNIQUE | 邀请码查询 |
| class_users | idx_class_users_class_id | class_id | NORMAL | 查询班级成员 |
| class_users | idx_class_users_user_id | user_id | NORMAL | 查询用户加入的班级 |
| homeworks | idx_homeworks_class_id | class_id | NORMAL | 查询班级的作业 |
| homeworks | idx_homeworks_created_by | created_by | NORMAL | 查询教师创建的作业 |
| homeworks | idx_homeworks_deadline | deadline | NORMAL | 按截止时间排序/筛选 |
| submissions | idx_submissions_homework_id | homework_id | NORMAL | 查询作业的提交 |
| submissions | idx_submissions_creator_id | creator_id | NORMAL | 查询学生的提交 |
| submissions | idx_submissions_status | status | NORMAL | 按状态筛选 |
| submissions | idx_submissions_hw_creator | (homework_id, creator_id) | COMPOSITE | 查询学生对某作业的提交 |
| grades | idx_grades_submission_id | submission_id | UNIQUE | 查询提交的评分 |
| grades | idx_grades_grader_id | grader_id | NORMAL | 查询教师的评分记录 |
| files | idx_files_user_id | user_id | NORMAL | 查询用户上传的文件 |
| files | idx_files_download_token | download_token | UNIQUE | 下载令牌查询 |
| notifications | idx_notifications_user_id | user_id | NORMAL | 查询用户通知 |
| notifications | idx_notifications_is_read | (user_id, is_read) | COMPOSITE | 查询未读通知 |
| notifications | idx_notifications_created_at | created_at DESC | NORMAL | 按时间排序 |

### 4.2 复合索引说明

**idx_submissions_hw_creator**：
- 用于查询"某学生对某作业的所有提交"
- 覆盖查询：`WHERE homework_id = ? AND creator_id = ?`

**idx_notifications_is_read**：
- 用于查询"某用户的未读通知"
- 覆盖查询：`WHERE user_id = ? AND is_read = false`

---

## 五、约束设计

### 5.1 唯一约束

| 表 | 约束 | 字段 |
|------|------|------|
| users | UK | username |
| users | UK | email |
| classes | UK | invite_code |
| class_users | UK | (class_id, user_id) |
| submissions | UK | (homework_id, creator_id, version) |
| grades | UK | submission_id |
| files | UK | download_token |

### 5.2 检查约束

| 表 | 约束 | 条件 |
|------|------|------|
| grades | CK | score >= 0 |

### 5.3 外键约束

| 表 | 外键 | 引用 | ON DELETE |
|------|------|------|-----------|
| classes | teacher_id | users.id | CASCADE |
| class_users | class_id | classes.id | CASCADE |
| class_users | user_id | users.id | CASCADE |
| homeworks | class_id | classes.id | CASCADE |
| homeworks | created_by | users.id | CASCADE |
| submissions | homework_id | homeworks.id | CASCADE |
| submissions | creator_id | users.id | CASCADE |
| grades | submission_id | submissions.id | CASCADE |
| grades | grader_id | users.id | SET NULL |
| files | user_id | users.id | SET NULL |
| homework_files | homework_id | homeworks.id | CASCADE |
| homework_files | file_id | files.id | CASCADE |
| submission_files | submission_id | submissions.id | CASCADE |
| submission_files | file_id | files.id | CASCADE |
| notifications | user_id | users.id | CASCADE |

---

## 六、枚举值定义

### 6.1 UserRole（系统角色）

```rust
pub enum UserRole {
    User,     // 普通用户
    Teacher,  // 教师
    Admin,    // 管理员
}
```

数据库存储：`"user"` / `"teacher"` / `"admin"`

### 6.2 UserStatus（用户状态）

```rust
pub enum UserStatus {
    Active,    // 正常
    Suspended, // 暂停
    Banned,    // 封禁
}
```

数据库存储：`"active"` / `"suspended"` / `"banned"`

### 6.3 ClassUserRole（班级角色）

```rust
pub enum ClassUserRole {
    Student,             // 学生
    ClassRepresentative, // 课代表
    Teacher,             // 班级教师
}
```

数据库存储：`"student"` / `"class_representative"` / `"teacher"`

### 6.4 SubmissionStatus（提交状态）

```rust
pub enum SubmissionStatus {
    Pending, // 待批改
    Graded,  // 已批改
    Late,    // 迟交
}
```

数据库存储：`"pending"` / `"graded"` / `"late"`

### 6.5 NotificationType（通知类型）

```rust
pub enum NotificationType {
    HomeworkCreated,     // 新作业发布
    HomeworkUpdated,     // 作业更新
    HomeworkDeadline,    // 作业即将截止
    SubmissionReceived,  // 收到新提交
    GradeReceived,       // 收到评分
    GradeUpdated,        // 评分修改
    ClassJoined,         // 加入班级
    ClassRoleChanged,    // 班级角色变更
}
```

数据库存储：`"homework_created"` / `"homework_updated"` / ...

---

## 七、迁移策略

### 7.1 破坏性重构

由于提交表和评分表需要重大修改，且无生产数据，采用破坏性重构：

1. 删除旧的迁移文件
2. 创建新的迁移文件 `m20250124_000001_redesign_tables.rs`
3. 重新建表

### 7.2 迁移文件

```rust
// migration/src/m20250124_000001_redesign_tables.rs

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. 创建 users 表
        // 2. 创建 classes 表
        // 3. 创建 class_users 表
        // 4. 创建 homeworks 表
        // 5. 创建 submissions 表
        // 6. 创建 grades 表
        // 7. 创建 files 表
        // 8. 创建 homework_files 表
        // 9. 创建 submission_files 表
        // 10. 创建 notifications 表
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 按依赖关系逆序删除
        Ok(())
    }
}
```

---

## 八、查询示例

### 8.1 查询某作业的所有提交（教师视图）

```sql
SELECT
    s.id,
    s.creator_id,
    u.username,
    u.display_name,
    s.version,
    s.status,
    s.is_late,
    s.submitted_at,
    g.score,
    g.comment
FROM submissions s
JOIN users u ON s.creator_id = u.id
LEFT JOIN grades g ON s.id = g.submission_id
WHERE s.homework_id = ?
  AND s.version = (
    SELECT MAX(version) FROM submissions
    WHERE homework_id = s.homework_id AND creator_id = s.creator_id
  )
ORDER BY s.submitted_at DESC;
```

### 8.2 查询某学生对某作业的提交历史

```sql
SELECT
    s.*,
    g.score,
    g.comment,
    g.graded_at
FROM submissions s
LEFT JOIN grades g ON s.id = g.submission_id
WHERE s.homework_id = ?
  AND s.creator_id = ?
ORDER BY s.version DESC;
```

### 8.3 查询作业提交统计

```sql
SELECT
    h.id AS homework_id,
    h.title,
    COUNT(DISTINCT cu.user_id) AS total_students,
    COUNT(DISTINCT s.creator_id) AS submitted_count,
    COUNT(DISTINCT g.id) AS graded_count,
    COUNT(DISTINCT CASE WHEN s.is_late THEN s.creator_id END) AS late_count,
    AVG(g.score) AS average_score,
    MAX(g.score) AS max_score,
    MIN(g.score) AS min_score
FROM homeworks h
JOIN class_users cu ON h.class_id = cu.class_id AND cu.role = 'student'
LEFT JOIN submissions s ON h.id = s.homework_id
LEFT JOIN grades g ON s.id = g.submission_id
WHERE h.id = ?
GROUP BY h.id;
```

### 8.4 查询未提交学生名单

```sql
SELECT
    u.id,
    u.username,
    u.display_name
FROM class_users cu
JOIN users u ON cu.user_id = u.id
WHERE cu.class_id = (SELECT class_id FROM homeworks WHERE id = ?)
  AND cu.role IN ('student', 'class_representative')
  AND NOT EXISTS (
    SELECT 1 FROM submissions s
    WHERE s.homework_id = ? AND s.creator_id = cu.user_id
  );
```

---

## 九、更新日志

| 版本 | 日期 | 变更内容 |
|------|------|----------|
| v2.0 | 2026-01-24 | 重构 submissions 和 grades 表；新增附件关联表和通知表 |
| v1.0 | 2025-01-23 | 初始版本 |
