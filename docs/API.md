# API 文档

> 版本：v2.0
> 更新日期：2026-01-24
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
| page_size | number | 20 | 每页数量，最大 100 |

分页响应格式：

```json
{
    "items": [...],
    "total": 100,
    "page": 1,
    "page_size": 20,
    "total_pages": 5
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
| 1009 | 资源冲突 |
| 2000 | 认证失败 |
| 2001 | 注册失败 |
| 3000 | 文件不存在 |
| 3001 | 文件上传失败 |
| 3002 | 文件类型不允许 |
| 3003 | 文件大小超限 |
| 4000 | 用户不存在 |
| 4001 | 用户已存在 |
| 5000 | 班级不存在 |
| 5005 | 班级权限不足 |
| 5011 | 邀请码无效 |
| 5012 | 已加入该班级 |

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
    "token_type": "Bearer",
    "expires_in": 900,
    "user": {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "username": "john_doe",
        "email": "john@example.com",
        "display_name": "John Doe",
        "role": "user",
        "status": "active"
    }
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
    "token_type": "Bearer",
    "expires_in": 900
}
```

### 2.4 GET /auth/verify-token

验证 Token 有效性。

**权限**：JWT

**响应**：
```json
{
    "valid": true,
    "user_id": "550e8400-e29b-41d4-a716-446655440000",
    "role": "user",
    "expires_at": "2026-01-24T12:15:00Z"
}
```

### 2.5 GET /auth/me

获取当前用户信息。

**权限**：JWT

**响应**：
```json
{
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "username": "john_doe",
    "email": "john@example.com",
    "display_name": "John Doe",
    "role": "user",
    "status": "active",
    "created_at": "2026-01-01T00:00:00Z"
}
```

### 2.6 POST /auth/logout

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
| page_size | number | 每页数量 |
| role | string | 按角色筛选 |
| status | string | 按状态筛选 |
| search | string | 搜索用户名/邮箱 |

**响应**：
```json
{
    "items": [
        {
            "id": "...",
            "username": "...",
            "email": "...",
            "display_name": "...",
            "role": "user",
            "status": "active",
            "created_at": "..."
        }
    ],
    "total": 100,
    "page": 1,
    "page_size": 20
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

---

## 四、班级管理

### 4.1 GET /classes

获取班级列表。

**权限**：JWT

**说明**：
- 普通用户：返回自己加入的班级
- Admin：返回所有班级

**响应**：
```json
{
    "items": [
        {
            "id": "...",
            "name": "数据结构",
            "description": "2026春季班",
            "teacher": {
                "id": "...",
                "username": "...",
                "display_name": "..."
            },
            "member_count": 30,
            "my_role": "student",
            "created_at": "..."
        }
    ]
}
```

### 4.2 POST /classes

创建班级。

**权限**：Teacher+

**请求**：
```json
{
    "name": "数据结构",
    "description": "2026春季班"
}
```

**响应**：
```json
{
    "id": "...",
    "name": "数据结构",
    "description": "2026春季班",
    "invite_code": "ABC123",
    "teacher_id": "...",
    "created_at": "..."
}
```

### 4.3 GET /classes/code/{code}

通过邀请码查询班级。

**权限**：JWT

**响应**：
```json
{
    "id": "...",
    "name": "数据结构",
    "description": "2026春季班",
    "teacher": {
        "id": "...",
        "display_name": "张老师"
    },
    "member_count": 30
}
```

### 4.4 GET /classes/{class_id}

获取班级详情。

**权限**：班级成员 或 Admin

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
    "items": [
        {
            "id": "...",
            "user": {
                "id": "...",
                "username": "...",
                "display_name": "..."
            },
            "role": "student",
            "joined_at": "..."
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

### 6.1 GET /classes/{class_id}/homeworks

获取班级作业列表。

**权限**：班级成员

**查询参数**：
| 参数 | 类型 | 说明 |
|------|------|------|
| page | number | 页码 |
| page_size | number | 每页数量 |
| status | string | `upcoming`/`ongoing`/`ended` |

**响应**：
```json
{
    "items": [
        {
            "id": "...",
            "title": "链表实现",
            "description": "实现单链表的基本操作",
            "max_score": 100.0,
            "deadline": 1706140800,
            "allow_late": false,
            "attachment_count": 2,
            "my_submission": {
                "id": "...",
                "version": 2,
                "status": "graded",
                "score": 85.0
            },
            "created_at": "..."
        }
    ]
}
```

### 6.2 POST /classes/{class_id}/homeworks

创建作业。

**权限**：班级教师

**请求**：
```json
{
    "title": "链表实现",
    "description": "实现单链表的基本操作",
    "max_score": 100.0,
    "deadline": 1706140800,
    "allow_late": false,
    "attachments": ["file_id_1", "file_id_2"]
}
```

**响应**：
```json
{
    "id": "...",
    "class_id": "...",
    "title": "链表实现",
    "description": "...",
    "max_score": 100.0,
    "deadline": 1706140800,
    "allow_late": false,
    "attachments": [
        {
            "id": "...",
            "original_name": "要求.pdf",
            "file_size": 102400,
            "download_token": "..."
        }
    ],
    "created_by": "...",
    "created_at": "..."
}
```

### 6.3 GET /classes/{class_id}/homeworks/{homework_id}

获取作业详情。

**权限**：班级成员

### 6.4 PUT /classes/{class_id}/homeworks/{homework_id}

更新作业。

**权限**：班级教师

**请求**：
```json
{
    "title": "string",
    "description": "string",
    "max_score": 100.0,
    "deadline": 1706140800,
    "allow_late": true,
    "attachments": ["file_id_1"]
}
```

### 6.5 DELETE /classes/{class_id}/homeworks/{homework_id}

删除作业。

**权限**：班级教师

### 6.6 GET /classes/{class_id}/homeworks/{homework_id}/stats

获取作业统计。

**权限**：班级教师 或 课代表

**响应**：
```json
{
    "homework_id": "...",
    "total_students": 30,
    "submitted_count": 25,
    "graded_count": 20,
    "late_count": 3,
    "submission_rate": 83.33,
    "average_score": 85.5,
    "max_score_achieved": 98.0,
    "min_score_achieved": 62.0,
    "score_distribution": {
        "90-100": 5,
        "80-89": 8,
        "70-79": 4,
        "60-69": 2,
        "0-59": 1
    },
    "not_submitted": [
        {
            "user_id": "...",
            "username": "...",
            "display_name": "..."
        }
    ]
}
```

---

## 七、提交管理

### 7.1 GET /homeworks/{homework_id}/submissions

获取所有提交列表（教师/课代表视图）。

**权限**：班级教师 或 课代表

**查询参数**：
| 参数 | 类型 | 说明 |
|------|------|------|
| page | number | 页码 |
| page_size | number | 每页数量 |
| status | string | `pending`/`graded`/`late` |
| latest_only | boolean | 只显示最新版本（默认 true） |

**响应**：
```json
{
    "items": [
        {
            "id": "...",
            "creator": {
                "id": "...",
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

### 7.2 POST /homeworks/{homework_id}/submissions

提交作业。

**权限**：班级学生/课代表

**请求**：
```json
{
    "content": "这是我的作业内容...",
    "attachments": ["file_id_1"]
}
```

**响应**：
```json
{
    "id": "...",
    "homework_id": "...",
    "creator_id": "...",
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
            "id": "...",
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
            "id": "...",
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

### 7.5 GET /homeworks/{homework_id}/submissions/{submission_id}

获取提交详情。

**权限**：提交者 或 班级教师

### 7.6 DELETE /homeworks/{homework_id}/submissions/{submission_id}

撤回提交。

**权限**：提交者（截止前）

**错误**：
- 如果作业已截止，不允许撤回

---

## 八、评分管理

### 8.1 GET /submissions/{submission_id}/grade

获取评分。

**权限**：提交者 或 班级教师

**响应**：
```json
{
    "id": "...",
    "submission_id": "...",
    "grader": {
        "id": "...",
        "username": "...",
        "display_name": "..."
    },
    "score": 85.0,
    "max_score": 100.0,
    "comment": "Good work!",
    "graded_at": "2026-01-24T12:00:00Z",
    "updated_at": "2026-01-24T12:00:00Z"
}
```

### 8.2 POST /submissions/{submission_id}/grade

创建评分。

**权限**：班级教师（课代表不能评分）

**请求**：
```json
{
    "score": 85.0,
    "comment": "Good work!"
}
```

**验证**：
- `score` 必须 >= 0
- `score` 不能超过作业的 `max_score`

**错误**：
- 如果已存在评分，返回 409 冲突

### 8.3 PUT /submissions/{submission_id}/grade

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
- 最大 10MB
- 允许类型：`image/png`, `image/jpeg`, `application/pdf`, `text/plain`, `application/zip`

**响应**：
```json
{
    "id": "...",
    "original_name": "document.pdf",
    "file_type": "application/pdf",
    "file_size": 102400,
    "download_token": "abc123..."
}
```

### 9.2 GET /files/download/{token}

下载文件。

**权限**：JWT

### 9.3 DELETE /files/{file_id}

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
| page_size | number | 每页数量 |
| is_read | boolean | 筛选已读/未读 |
| type | string | 筛选通知类型 |

**响应**：
```json
{
    "items": [
        {
            "id": "...",
            "type": "homework_created",
            "title": "新作业发布",
            "content": "《数据结构》作业已发布",
            "reference_type": "homework",
            "reference_id": "...",
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
    "count": 5
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
        "id": "...",
        "type": "homework_created",
        "title": "新作业发布",
        "content": "《数据结构》作业已发布",
        "reference_type": "homework",
        "reference_id": "...",
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

---

## 十二、系统设置

### 12.1 GET /system/health

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

### 12.2 GET /system/settings

获取系统设置。

**权限**：Admin

### 12.3 GET /system/uptime

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
| v2.0 | 2026-01-24 | 新增提交版本控制、评分修改、通知系统、WebSocket |
| v1.0 | 2025-01-23 | 初始版本 |
