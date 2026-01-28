# API 文档

> 版本：v2.10
> 更新日期：2026-01-28
> 基础路径：`/api/v1`

---

## 一、通用约定

### 1.1 响应格式

所有 API 返回统一的 JSON 格式：

```json
{
    "code": 0,
    "message": "Success",
    "data": { ... },
    "timestamp": "2026-01-24T12:00:00Z"
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| code | number | 错误码，0 表示成功 |
| message | string | 消息文本 |
| data | object | 响应数据，可选 |
| timestamp | string | ISO 8601 时间戳 |

### 1.2 认证方式

需要认证的 API 在请求头中携带 JWT Token：

```
Authorization: Bearer <access_token>
```

### 1.3 分页参数

支持分页的接口使用以下查询参数：

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| page | number | 1 | 页码，从 1 开始 |
| size | number | 20 | 每页数量，最大 100 |

分页响应格式：

```json
{
    "items": [...],
    "pagination": {
        "page": 1,
        "page_size": 20,
        "total": 100,
        "total_pages": 5
    }
}
```

### 1.4 错误码

| 错误码 | 说明 |
|--------|------|
| 0 | 成功 |
| 1000 | 请求参数错误 |
| 1001 | 未授权（未登录） |
| 1003 | 权限不足 |
| 1004 | 资源不存在 |
| 1005 | 服务器内部错误 |
| 1006 | 未实现的功能 |
| 1009 | 资源冲突 |
| 1029 | 请求过于频繁（速率限制） |
| 2000 | 认证失败 |
| 2001 | 注册失败 |
| 2002 | 密码不符合策略要求 |
| 3000 | 文件不存在 |
| 3001 | 文件上传失败 |
| 3002 | 文件类型不允许 |
| 3003 | 文件大小超限 |
| 3004 | 不允许多文件上传 |
| 4000 | 用户不存在 |
| 4001 | 用户已存在 |
| 4002 | 用户更新失败 |
| 4003 | 用户删除失败 |
| 4004 | 用户创建失败 |
| 4005 | 不能删除当前用户 |
| 4010 | 用户名无效 |
| 4011 | 用户名已存在 |
| 4012 | 用户邮箱无效 |
| 4013 | 用户邮箱已存在 |
| 4014 | 密码不符合策略要求 |
| 5000 | 班级不存在 |
| 5001 | 班级已存在 |
| 5002 | 班级创建失败 |
| 5003 | 班级更新失败 |
| 5004 | 班级删除失败 |
| 5005 | 班级权限不足 |
| 5010 | 加入班级失败 |
| 5011 | 邀请码无效 |
| 5012 | 已加入该班级 |
| 5013 | 加入班级被禁止 |
| 5014 | 班级用户未找到 |
| 6000 | 权限被拒绝 |
| 7000 | 导入文件解析失败 |
| 7001 | 导入文件格式无效 |
| 7002 | 导入文件缺少必需列 |
| 7003 | 导入文件数据无效 |
| 7010 | 导出失败 |

---

## 二、认证模块

### 2.1 POST /auth/login

用户登录。

**权限**：公开

**请求**：
```json
{
    "username": "string",      // 用户名或邮箱
    "password": "string",
    "remember_me": false       // 可选，延长 refresh token 有效期
}
```

**响应**：
```json
{
    "access_token": "eyJhbGci...",
    "expires_in": 900,
    "user": {
        "id": 1,
        "username": "john_doe",
        "email": "john@example.com",
        "display_name": "John Doe",
        "role": "user",
        "status": "active"
    },
    "created_at": "2026-01-24T12:00:00Z"
}
```

**说明**：
- Refresh Token 通过 HttpOnly Cookie 返回
- `remember_me=true` 时 Refresh Token 有效期 30 天，否则 7 天

### 2.2 POST /auth/register

用户注册。

**权限**：公开

**请求**：
```json
{
    "username": "string",      // 3-32字符，字母数字下划线
    "email": "string",         // 有效邮箱
    "password": "string",      // 8位以上，含大小写和数字
    "display_name": "string"   // 可选
}
```

**响应**：同登录响应

### 2.3 POST /auth/refresh

刷新 Access Token。

**权限**：公开（需携带 Refresh Token Cookie）

**响应**：
```json
{
    "access_token": "eyJhbGci...",
    "expires_in": 900
}
```

### 2.4 GET /auth/verify-token

验证 Token 有效性。

**权限**：JWT

**响应**：
```json
{
    "is_valid": true
}
```

### 2.5 GET /auth/me

获取当前用户信息。

**权限**：JWT

**响应**：
```json
{
    "id": 1,
    "username": "john_doe",
    "email": "john@example.com",
    "display_name": "John Doe",
    "role": "user",
    "status": "active",
    "created_at": "2026-01-01T00:00:00Z"
}
```

### 2.6 PUT /auth/me

更新当前用户个人资料。

**权限**：JWT

**请求**：
```json
{
    "display_name": "新名称"
}
```

**响应**：
```json
{
    "user": {
        "id": 1,
        "username": "john_doe",
        "email": "john@example.com",
        "display_name": "新名称",
        "role": "user",
        "status": "active"
    }
}
```

### 2.7 POST /auth/logout ⚠️ 未实现

用户登出，清除 Refresh Token。

**权限**：JWT

**响应**：
```json
{
    "message": "Logged out successfully"
}
```

---

## 三、用户管理

### 3.1 GET /users

获取用户列表（分页）。

**权限**：Admin

**查询参数**：
| 参数 | 类型 | 说明 |
|------|------|------|
| page | number | 页码 |
| size | number | 每页数量 |
| role | string | 按角色筛选 |
| status | string | 按状态筛选 |
| search | string | 搜索用户名/邮箱 |

**响应**：
```json
{
    "items": [
        {
            "id": 1,
            "username": "...",
            "email": "...",
            "display_name": "...",
            "role": "user",
            "status": "active",
            "created_at": "..."
        }
    ],
    "pagination": {
        "page": 1,
        "page_size": 20,
        "total": 100,
        "total_pages": 5
    }
}
```

### 3.2 POST /users

创建用户。

**权限**：Admin

**请求**：
```json
{
    "username": "string",
    "email": "string",
    "password": "string",
    "display_name": "string",
    "role": "user"             // user/teacher/admin
}
```

### 3.3 GET /users/{id}

获取用户详情。

**权限**：Admin

### 3.4 PUT /users/{id}

更新用户信息。

**权限**：Admin

**请求**：
```json
{
    "display_name": "string",
    "role": "string",
    "status": "string"
}
```

### 3.5 DELETE /users/{id}

删除用户。

**权限**：Admin

### 3.6 GET /users/export

导出用户列表。

**权限**：Admin

**查询参数**：
| 参数 | 类型 | 说明 |
|------|------|------|
| format | string | 导出格式：`csv` / `xlsx`（默认 csv） |
| role | string | 按角色筛选 |
| status | string | 按状态筛选 |

**响应**：文件下载（Content-Type: text/csv 或 application/vnd.openxmlformats-officedocument.spreadsheetml.sheet）

### 3.7 POST /users/import

导入用户。

**权限**：Admin

**请求**：`multipart/form-data`
- `file`：CSV 或 XLSX 文件

**响应**：
```json
{
    "total": 12,
    "success": 10,
    "skipped": 0,
    "failed": 2,
    "errors": [
        {"row": 3, "field": "username", "message": "用户名已存在"},
        {"row": 7, "field": "email", "message": "邮箱格式无效"}
    ]
}
```

### 3.8 GET /users/import/template

下载用户导入模板。

**权限**：Admin

**查询参数**：
| 参数 | 类型 | 说明 |
|------|------|------|
| format | string | 模板格式：`csv` / `xlsx`（默认 csv） |

**响应**：文件下载

### 3.9 GET /users/me/stats

获取当前用户的综合统计。

**权限**：JWT

**响应**：
```json
{
    "class_count": 3,
    "total_students": 90,
    "homework_pending": 5,
    "homework_submitted": 3,
    "homework_graded": 12,
    "pending_review": 15,
    "server_time": "2026-01-26T12:00:00Z"
}
```

**说明**：
- `class_count`：用户加入/创建的班级数量
- `total_students`：教师视角下的学生总数（学生视角为 0）
- `homework_pending`：学生视角下待完成的作业数
- `homework_submitted`：学生视角下已提交待批改的作业数
- `homework_graded`：学生视角下已批改的作业数
- `pending_review`：教师视角下待批改的提交数（学生视角为 0）
- `server_time`：服务器时间（ISO 8601），用于前端统一时间判断

---

## 四、班级管理

### 4.1 GET /classes

获取班级列表。

**权限**：JWT

**说明**：
- 普通用户：返回自己加入的班级
- 教师：返回自己创建的班级
- Admin：返回所有班级

**响应**：
```json
{
    "pagination": {
        "page": 1,
        "page_size": 20,
        "total": 5,
        "total_pages": 1
    },
    "items": [
        {
            "id": 1,
            "name": "数据结构",
            "description": "2026春季班",
            "teacher_id": 2,
            "invite_code": "ABC123",
            "created_at": "2026-01-24T00:00:00Z",
            "updated_at": "2026-01-24T00:00:00Z",
            "teacher": {
                "id": 2,
                "username": "teacher1",
                "display_name": "张老师"
            },
            "member_count": 30
        }
    ]
}
```

### 4.2 POST /classes

创建班级。

**权限**：Teacher+

**说明**：
- 教师创建：自动使用当前登录教师的 ID，无需指定 `teacher_id`
- 管理员创建：必须指定 `teacher_id` 来绑定负责该班级的教师

**请求（教师创建）**：
```json
{
    "name": "数据结构",
    "description": "2026春季班"
}
```

**请求（管理员创建）**：
```json
{
    "teacher_id": 2,
    "name": "数据结构",
    "description": "2026春季班"
}
```

**响应**：
```json
{
    "id": 1,
    "name": "数据结构",
    "description": "2026春季班",
    "invite_code": "ABC123",
    "teacher_id": 2,
    "created_at": "..."
}
```

**错误码**：
- 1000：管理员未指定 teacher_id
- 4000：指定的教师不存在
- 5005：无权限创建班级（班级权限被拒绝）。典型触发场景：指定的用户不是教师角色、教师在请求中填写了非本人 `teacher_id`、非 Teacher/Admin 角色尝试创建班级

### 4.3 GET /classes/code/{code}

通过邀请码查询班级。

**权限**：JWT

**响应**：
```json
{
    "id": 1,
    "name": "数据结构",
    "description": "2026春季班",
    "teacher": {
        "id": 2,
        "display_name": "张老师"
    },
    "member_count": 30
}
```

### 4.4 GET /classes/{class_id}

获取班级详情。

**权限**：班级成员 或 班级教师 或 Admin

**响应**：
```json
{
    "id": 1,
    "name": "数据结构",
    "description": "2026春季班",
    "teacher_id": 2,
    "invite_code": "ABC123",
    "created_at": "2026-01-24T00:00:00Z",
    "updated_at": "2026-01-24T00:00:00Z",
    "teacher": {
        "id": 2,
        "username": "teacher1",
        "display_name": "张老师"
    },
    "member_count": 30
}
```

### 4.5 PUT /classes/{class_id}

更新班级信息。

**权限**：班级教师 或 Admin

**请求**：
```json
{
    "name": "string",
    "description": "string"
}
```

### 4.6 DELETE /classes/{class_id}

删除班级。

**权限**：班级教师 或 Admin

### 4.7 GET /classes/{class_id}/export

导出班级报表。

**权限**：班级教师 或 课代表 或 Admin

**响应**：文件下载（Excel 格式），包含班级成员列表、作业完成情况等

---

## 五、班级成员

### 5.1 POST /classes/{class_id}/students

加入班级。

**权限**：JWT

**请求**：
```json
{
    "invite_code": "ABC123"
}
```

**错误码**：
- 5011：邀请码无效
- 5012：已加入该班级

### 5.2 GET /classes/{class_id}/students

获取班级成员列表。

**权限**：课代表+ 或 Admin

**响应**：
```json
{
    "pagination": {
        "page": 1,
        "page_size": 20,
        "total": 30,
        "total_pages": 2
    },
    "items": [
        {
            "id": 1,
            "class_id": 1,
            "user_id": 3,
            "role": "student",
            "joined_at": "2026-01-24T00:00:00Z",
            "user": {
                "id": 3,
                "username": "student1",
                "display_name": "张三",
                "avatar_url": null
            }
        }
    ]
}
```

### 5.3 GET /classes/{class_id}/students/{user_id}

获取成员详情。

**权限**：班级成员

### 5.4 PUT /classes/{class_id}/students/{user_id}

修改成员角色。

**权限**：班级教师

**请求**：
```json
{
    "role": "class_representative"
}
```

### 5.5 DELETE /classes/{class_id}/students/{user_id}

移除成员。

**权限**：班级教师 或 自己（退出班级）

---

## 六、作业管理

### 6.1 GET /homeworks

获取作业列表。

**权限**：JWT

**查询参数**：
| 参数 | 类型 | 说明 |
|------|------|------|
| class_id | number | 班级 ID（必填） |
| page | number | 页码 |
| size | number | 每页数量 |
| status | string | `upcoming`/`ongoing`/`ended` |

**响应**：
```json
{
    "items": [
        {
            "id": 1,
            "title": "链表实现",
            "description": "实现单链表的基本操作",
            "max_score": 100.0,
            "deadline": "2026-01-25T00:00:00Z",
            "allow_late": false,
            "attachment_count": 2,
            "my_submission": {
                "id": 1,
                "version": 2,
                "status": "graded",
                "score": 85.0
            },
            "created_at": "..."
        }
    ]
}
```

### 6.2 POST /homeworks

创建作业。

**权限**：班级教师

**请求**：
```json
{
    "class_id": 1,
    "title": "链表实现",
    "description": "实现单链表的基本操作",
    "max_score": 100.0,
    "deadline": "2026-01-25T00:00:00Z",
    "allow_late": false,
    "attachments": ["download_token_1", "download_token_2"]
}
```

**说明**：
- `deadline` 使用 ISO 8601 格式（如 `"2026-01-25T00:00:00Z"`）
- `attachments` 使用文件上传后返回的 `download_token`
- 只能使用当前用户上传的文件，否则返回 403 权限错误

**响应**：
```json
{
    "id": 1,
    "class_id": 1,
    "title": "链表实现",
    "description": "...",
    "max_score": 100.0,
    "deadline": "2026-01-25T00:00:00Z",
    "allow_late": false,
    "attachments": [
        {
            "id": 1,
            "original_name": "要求.pdf",
            "file_size": 102400,
            "download_token": "..."
        }
    ],
    "created_by": 2,
    "created_at": "..."
}
```

### 6.3 GET /homeworks/{id}

获取作业详情。

**权限**：班级成员

**响应**：
```json
{
    "id": 1,
    "class_id": 3,
    "title": "链表实现",
    "description": "实现单链表的基本操作",
    "max_score": 100.0,
    "deadline": "2026-01-25T00:00:00Z",
    "allow_late": false,
    "created_by": 2,
    "created_at": "2026-01-24T00:00:00Z",
    "updated_at": "2026-01-24T00:00:00Z",
    "attachments": [
        {
            "download_token": "abc123...",
            "original_name": "要求.pdf",
            "file_size": 102400,
            "file_type": "application/pdf"
        }
    ]
}
```

### 6.4 PUT /homeworks/{id}

更新作业。

**权限**：班级教师

**请求**：
```json
{
    "title": "string",
    "description": "string",
    "max_score": 100.0,
    "deadline": "2026-01-25T00:00:00Z",
    "allow_late": true,
    "attachments": ["download_token_1"]
}
```

**说明**：
- `deadline` 使用 ISO 8601 格式（如 `"2026-01-25T00:00:00Z"`）
- `attachments` 使用文件上传后返回的 `download_token`
- 只能使用当前用户上传的文件，否则返回 403 权限错误

### 6.5 DELETE /homeworks/{id}

删除作业。

**权限**：班级教师

### 6.6 GET /homeworks/{id}/stats

获取作业统计。

**权限**：班级教师 或 课代表

**响应**：
```json
{
    "homework_id": 1,
    "total_students": 30,
    "submitted_count": 25,
    "graded_count": 20,
    "late_count": 3,
    "submission_rate": 83.33,
    "score_stats": {
        "average": 85.5,
        "max": 98.0,
        "min": 62.0
    },
    "score_distribution": [
        { "range": "90-100", "count": 5 },
        { "range": "80-89", "count": 8 },
        { "range": "70-79", "count": 4 },
        { "range": "60-69", "count": 2 },
        { "range": "0-59", "count": 1 }
    ],
    "unsubmitted_students": [
        {
            "id": 1,
            "username": "student1",
            "display_name": "张三",
            "avatar_url": null
        }
    ]
}
```

### 6.7 GET /homeworks/{id}/stats/export

导出作业统计报表。

**权限**：班级教师 或 课代表

**响应**：文件下载（Excel 格式），包含提交情况、成绩分布等

### 6.8 GET /homeworks/my/stats

获取当前学生的作业统计。

**权限**：JWT

**响应**：
```json
{
    "total": 10,
    "pending": 3,
    "submitted": 5,
    "graded": 2
}
```

### 6.9 GET /homeworks/teacher/stats

获取教师的作业统计。

**权限**：Teacher+

**响应**：
```json
{
    "total_homeworks": 15,
    "pending_review": 25,
    "total_submissions": 120,
    "graded_submissions": 95
}
```

### 6.10 GET /homeworks/all

获取跨班级作业列表。

**权限**：JWT

**查询参数**：
| 参数 | 类型 | 说明 |
|------|------|------|
| page | number | 页码 |
| size | number | 每页数量 |
| status | string | 按用户提交状态筛选：`pending`/`submitted`/`graded` |
| deadline_filter | string | 按截止时间筛选：`active`/`expired`/`all` |
| search | string | 搜索作业标题 |
| include_stats | boolean | 是否包含统计摘要（教师视角） |

**响应**：
```json
{
    "items": [
        {
            "id": 1,
            "class_id": 1,
            "title": "链表实现",
            "description": "...",
            "max_score": 100.0,
            "deadline": "2026-01-25T00:00:00Z",
            "allow_late": false,
            "created_by": 2,
            "created_at": "...",
            "updated_at": "...",
            "creator": {
                "id": 2,
                "username": "teacher1",
                "display_name": "张老师",
                "avatar_url": null
            },
            "my_submission": {
                "id": 1,
                "version": 2,
                "status": "graded",
                "is_late": false,
                "score": 85.0
            },
            "stats_summary": null
        }
    ],
    "pagination": {
        "page": 1,
        "page_size": 20,
        "total": 50,
        "total_pages": 3
    },
    "server_time": "2026-01-26T12:00:00Z"
}
```

**说明**：
- `my_submission`：当前用户的最新提交（仅学生视角有值）
- `stats_summary`：作业统计摘要（仅教师/管理员视角且 `include_stats=true` 时有值）
- `server_time`：服务器时间，用于前端统一时间判断

---

## 七、提交管理

### 7.1 GET /submissions

获取提交列表（教师/课代表视图）。

**权限**：班级教师 或 课代表

**查询参数**：
| 参数 | 类型 | 说明 |
|------|------|------|
| homework_id | number | 作业 ID（必填） |
| page | number | 页码 |
| size | number | 每页数量 |
| status | string | `pending`/`graded`/`late` |
| latest_only | boolean | 只显示最新版本（默认 true） |

**响应**：
```json
{
    "items": [
        {
            "id": 1,
            "creator": {
                "id": 3,
                "username": "...",
                "display_name": "..."
            },
            "version": 2,
            "content": "...",
            "status": "graded",
            "is_late": false,
            "attachment_count": 1,
            "grade": {
                "score": 85.0,
                "comment": "Good work!",
                "graded_at": "..."
            },
            "submitted_at": "..."
        }
    ]
}
```

### 7.2 POST /submissions

提交作业。

**权限**：班级学生/课代表

**请求**：
```json
{
    "homework_id": 1,
    "content": "这是我的作业内容...",
    "attachments": ["file_id_1"]
}
```

**响应**：
```json
{
    "id": 1,
    "homework_id": 1,
    "creator_id": 3,
    "version": 2,
    "content": "...",
    "status": "pending",
    "is_late": false,
    "attachments": [...],
    "submitted_at": "..."
}
```

**错误**：
- 如果作业已截止且不允许迟交，返回错误

### 7.3 GET /homeworks/{homework_id}/submissions/my

获取我的提交历史。

**权限**：班级学生/课代表

**响应**：
```json
{
    "items": [
        {
            "id": 1,
            "version": 1,
            "content": "...",
            "status": "graded",
            "is_late": false,
            "attachments": [...],
            "grade": {
                "score": 75.0,
                "comment": "需要改进"
            },
            "submitted_at": "..."
        },
        {
            "id": 2,
            "version": 2,
            "content": "...",
            "status": "graded",
            "is_late": false,
            "attachments": [...],
            "grade": {
                "score": 85.0,
                "comment": "Good work!"
            },
            "submitted_at": "..."
        }
    ]
}
```

### 7.4 GET /homeworks/{homework_id}/submissions/my/latest

获取我的最新提交。

**权限**：班级学生/课代表

**响应**：

成功且有提交记录时返回提交详情：
```json
{
    "code": 0,
    "data": {
        "id": 1,
        "homework_id": 1,
        "creator_id": 3,
        "version": 1,
        "content": "作业内容",
        "attachments": [
            {
                "download_token": "abc123...",
                "original_name": "作业.pdf",
                "file_size": 102400,
                "file_type": "application/pdf"
            }
        ],
        "submitted_at": "2026-01-24T12:00:00Z",
        "is_late": false,
        "grade": null
    },
    "message": "查询成功"
}
```

成功但尚未提交时返回 null：
```json
{
    "code": 0,
    "data": null,
    "message": "暂无提交"
}
```

### 7.5 GET /homeworks/{homework_id}/submissions/summary

获取提交概览（教师视图）。

**权限**：班级教师 或 课代表

**响应**：
```json
{
    "homework_id": 1,
    "total_students": 30,
    "submitted_count": 25,
    "graded_count": 20,
    "pending_count": 5,
    "late_count": 3
}
```

### 7.6 GET /homeworks/{homework_id}/submissions/user/{user_id}

获取指定学生的提交历史（教师视角）。

**权限**：班级教师

### 7.7 GET /submissions/{id}

获取提交详情。

**权限**：提交者 或 班级教师

### 7.8 DELETE /submissions/{id}

撤回提交。

**权限**：提交者（截止前）

**错误**：
- 如果作业已截止，不允许撤回

---

## 八、评分管理

### 8.1 GET /submissions/{submission_id}/grade

获取提交的评分。

**权限**：提交者 或 班级教师

**响应**：
```json
{
    "id": 1,
    "submission_id": 1,
    "grader": {
        "id": 2,
        "username": "...",
        "display_name": "..."
    },
    "score": 85.0,
    "comment": "Good work!",
    "graded_at": "2026-01-24T12:00:00Z"
}
```

### 8.2 POST /grades

创建评分。

**权限**：班级教师（课代表不能评分）

**请求**：
```json
{
    "submission_id": 1,
    "score": 85.0,
    "comment": "Good work!"
}
```

**验证**：
- `score` 必须 >= 0
- `score` 不能超过作业的 `max_score`

**错误**：
- 如果已存在评分，返回 409 冲突

### 8.3 PUT /grades/{id}

修改评分。

**权限**：班级教师

**请求**：
```json
{
    "score": 90.0,
    "comment": "重新审核后调整分数"
}
```

---

## 九、文件管理

### 9.1 POST /files/upload

上传文件。

**权限**：JWT

**请求**：`multipart/form-data`
- `file`：文件内容

**限制**：
- 最大 10MB（可通过系统设置调整）
- 允许类型：`.png`, `.jpg`, `.jpeg`, `.gif`, `.pdf`, `.txt`, `.zip`（可通过系统设置调整）

**响应**：
```json
{
    "download_token": "abc123...",
    "file_name": "document.pdf",
    "size": 102400,
    "content_type": "application/pdf",
    "created_at": "2026-01-26T12:00:00Z"
}
```

### 9.2 GET /files/download/{token}

下载文件。

**权限**：JWT

### 9.3 DELETE /files/{file_id} ⚠️ 未实现

删除文件。

**权限**：上传者 或 Admin

---

## 十、通知系统

### 10.1 GET /notifications

获取通知列表。

**权限**：JWT

**查询参数**：
| 参数 | 类型 | 说明 |
|------|------|------|
| page | number | 页码 |
| size | number | 每页数量 |
| is_read | boolean | 筛选已读/未读 |
| type | string | 筛选通知类型 |

**响应**：
```json
{
    "items": [
        {
            "id": 1,
            "type": "homework_created",
            "title": "新作业发布",
            "content": "《数据结构》作业已发布",
            "reference_type": "homework",
            "reference_id": 1,
            "is_read": false,
            "created_at": "..."
        }
    ]
}
```

### 10.2 GET /notifications/unread-count

获取未读通知数量。

**权限**：JWT

**响应**：
```json
{
    "unread_count": 5
}
```

### 10.3 PUT /notifications/{id}/read

标记通知为已读。

**权限**：JWT

### 10.4 PUT /notifications/read-all

标记所有通知为已读。

**权限**：JWT

### 10.5 DELETE /notifications/{id}

删除通知。

**权限**：JWT

---

## 十一、WebSocket

### 11.1 连接

**路径**：`/api/v1/ws?token=<access_token>`

**协议**：WebSocket

### 11.2 消息格式

**服务端推送**：
```json
{
    "type": "notification",
    "payload": {
        "id": 1,
        "type": "homework_created",
        "title": "新作业发布",
        "content": "《数据结构》作业已发布",
        "reference_type": "homework",
        "reference_id": 1,
        "created_at": "..."
    }
}
```

**心跳**：
```json
// 客户端发送
{"type": "ping"}

// 服务端响应
{"type": "pong"}
```

**建议**：客户端每 30 秒发送一次 ping

### 11.3 GET /ws/status

获取 WebSocket 服务状态。

**权限**：JWT

**响应**：
```json
{
    "online_users": 123,
    "status": "ok"
}
```

---

## 十二、系统设置

### 12.1 GET /system/settings

获取公开系统设置（只读）。

**权限**：JWT

**响应**：
```json
{
    "system_name": "作业管理系统",
    "max_file_size": 10485760,
    "allowed_file_types": [".png", ".jpg", ".jpeg", ".gif", ".pdf", ".txt", ".zip"],
    "environment": "development",
    "log_level": "info"
}
```

### 12.2 GET /system/admin/settings

获取所有系统设置（管理员视图）。

**权限**：Admin

**响应**：
```json
{
    "settings": [
        {
            "key": "upload_max_size",
            "value": "10485760",
            "value_type": "integer",
            "description": "文件上传大小限制（字节）",
            "updated_at": "2026-01-24T12:00:00Z",
            "updated_by": 1
        }
    ]
}
```

### 12.3 PUT /system/admin/settings/{key}

更新系统设置。

**权限**：Admin

**请求**：
```json
{
    "value": "20971520"
}
```

**响应**：
```json
{
    "key": "upload_max_size",
    "value": "20971520",
    "value_type": "integer",
    "updated_at": "2026-01-24T12:00:00Z"
}
```

### 12.4 GET /system/admin/settings/audit

获取设置变更审计日志。

**权限**：Admin

**响应**：
```json
{
    "audits": [
        {
            "id": 1,
            "setting_key": "upload_max_size",
            "old_value": "10485760",
            "new_value": "20971520",
            "changed_by": 1,
            "changed_at": "2026-01-24T12:00:00Z",
            "ip_address": "192.168.1.1"
        }
    ],
    "pagination": {
        "page": 1,
        "page_size": 20,
        "total": 10,
        "total_pages": 1
    }
}
```

### 12.5 GET /system/health ⚠️ 未实现

健康检查。

**权限**：公开

**响应**：
```json
{
    "status": "ok",
    "database": "connected",
    "cache": "connected"
}
```

### 12.6 GET /system/uptime ⚠️ 未实现

获取系统运行时间。

**权限**：JWT

**响应**：
```json
{
    "uptime_seconds": 86400,
    "started_at": "2026-01-23T00:00:00Z"
}
```

---

## 十三、更新日志

| 版本 | 日期 | 变更内容 |
|------|------|----------|
| v2.10 | 2026-01-28 | 修正分页默认值：`size` 默认值改为 20（之前错误为 10）；统一 HTTP 入口参数命名为 `*Params`，存储层为 `*Query`；优化 HomeworkListParams、AllHomeworksParams 等结构；所有分页 API 已验证符合文档规范 |
| v2.9 | 2026-01-26 | 新增端点：`/users/me/stats`、`/homeworks/all`、`/ws/status`；修正响应格式：文件上传、系统设置、用户导入、教师统计、审计日志 |
| v2.8 | 2026-01-26 | 补充缺失端点（用户导入导出、作业统计、班级导出、系统设置管理）；补全错误码；修正响应字段 |
| v2.7 | 2026-01-24 | 修正文档与代码一致：ID 改为数字类型；分页参数 `page_size` → `size`；标注未实现端点；新增提交概览端点 |
| v2.6 | 2026-01-24 | 统一 API 响应格式：通知列表 `notifications` → `items`；提交历史返回 `{ items: [...] }` 结构 |
| v2.5 | 2026-01-24 | 班级详情允许班级成员访问；作业详情返回附件列表 |
| v2.4 | 2026-01-24 | 班级成员列表返回用户详情（用户名、头像等） |
| v2.3 | 2026-01-24 | 班级列表和详情返回教师信息和成员数量 |
| v2.2 | 2026-01-24 | 作业 deadline 改用 ISO 8601 格式 |
| v2.1 | 2026-01-24 | 作业附件改用 download_token，增加文件所有权校验 |
| v2.0 | 2026-01-24 | 新增提交版本控制、评分修改、通知系统、WebSocket |
| v1.0 | 2025-01-23 | 初始版本 |
