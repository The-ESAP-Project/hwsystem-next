use serde_repr::{Deserialize_repr, Serialize_repr};
use ts_rs::TS;

// ErrorCode 使用 serde_repr 序列化为数字
// 使用 #[ts(repr(enum))] 导出为 TypeScript 数字枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/error_code.ts")]
#[ts(rename = "ErrorCode")]
#[ts(repr(enum))]
#[repr(i32)]
pub enum ErrorCode {
    // 成功
    Success = 0, // 成功

    // 通用错误
    BadRequest = 1000,          // 错误的请求
    Unauthorized = 1001,        // 未授权访问
    Forbidden = 1003,           // 禁止访问
    NotFound = 1004,            // 未找到资源
    InternalServerError = 1005, // 内部服务器错误
    NotImplemented = 1006,      // 未实现的功能
    Conflict = 1009,            // 冲突 (资源已存在)
    RateLimitExceeded = 1029,   // 请求过于频繁

    // Auth 错误
    AuthFailed = 2000,              // 身份验证失败
    RegisterFailed = 2001,          // 注册失败
    PasswordPolicyViolation = 2002, // 密码不符合策略要求

    // 文件相关错误
    FileNotFound = 3000,              // 文件未找到
    FileUploadFailed = 3001,          // 文件上传失败
    FileTypeNotAllowed = 3002,        // 文件类型不被允许
    FileSizeExceeded = 3003,          // 文件大小超出限制
    MultifileUploadNotAllowed = 3004, // 不允许多文件上传

    // 用户相关错误
    UserNotFound = 4000,            // 用户未找到
    UserAlreadyExists = 4001,       // 用户已存在
    UserUpdateFailed = 4002,        // 用户更新失败
    UserDeleteFailed = 4003,        // 用户删除失败
    UserCreationFailed = 4004,      // 用户创建失败
    CanNotDeleteCurrentUser = 4005, // 不能删除当前用户

    UserNameInvalid = 4010,        // 用户名无效
    UserNameAlreadyExists = 4011,  // 用户名已存在
    UserEmailInvalid = 4012,       // 用户邮箱无效
    UserEmailAlreadyExists = 4013, // 用户邮箱已存在
    UserPasswordInvalid = 4014,    // 密码不符合策略要求

    // 班级相关错误
    ClassNotFound = 5000,          // 班级未找到
    ClassAlreadyExists = 5001,     // 班级已存在
    ClassCreationFailed = 5002,    // 班级创建失败
    ClassUpdateFailed = 5003,      // 班级更新失败
    ClassDeleteFailed = 5004,      // 班级删除失败
    ClassPermissionDenied = 5005,  // 班级权限被拒绝
    ClassJoinFailed = 5010,        // 加入班级失败
    ClassInviteCodeInvalid = 5011, // 班级邀请码无效
    ClassAlreadyJoined = 5012,     // 已经加入该班级
    ClassJoinForbidden = 5013,     // 加入班级被禁止
    ClassUserNotFound = 5014,      // 班级用户未找到

    // 通用权限错误
    PermissionDenied = 6000, // 权限被拒绝

    // 导入/导出相关错误
    ImportFileParseFailed = 7000,   // 导入文件解析失败
    ImportFileFormatInvalid = 7001, // 导入文件格式无效
    ImportFileMissingColumn = 7002, // 导入文件缺少必需列
    ImportFileDataInvalid = 7003,   // 导入文件数据无效
    ExportFailed = 7010,            // 导出失败

    // 作业相关错误
    HomeworkNotFound = 8000,     // 作业未找到
    HomeworkCreateFailed = 8001, // 作业创建失败
    HomeworkUpdateFailed = 8002, // 作业更新失败
    HomeworkDeleteFailed = 8003, // 作业删除失败

    // 提交相关错误
    SubmissionNotFound = 9000,     // 提交未找到
    SubmissionCreateFailed = 9001, // 提交创建失败
    SubmissionDeleteFailed = 9002, // 提交删除失败

    // 成绩相关错误
    GradeNotFound = 10000,     // 成绩未找到
    GradeCreateFailed = 10001, // 成绩创建失败
    GradeUpdateFailed = 10002, // 成绩更新失败

    // 通知相关错误
    NotificationNotFound = 11000, // 通知未找到
}
