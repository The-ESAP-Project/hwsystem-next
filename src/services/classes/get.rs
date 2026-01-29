use std::sync::Arc;

use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};

use super::ClassService;
use crate::services::StorageProvider;
use crate::{
    middlewares::RequireJWT,
    models::{
        ApiResponse, ErrorCode,
        class_users::entities::ClassUserRole,
        classes::{
            entities::Class,
            responses::{ClassDetail, TeacherInfo},
        },
        users::entities::UserRole,
    },
    storage::Storage,
};

pub async fn get_class(
    service: &ClassService,
    request: &HttpRequest,
    class_id: i64,
) -> ActixResult<HttpResponse> {
    let role = RequireJWT::extract_user_role(request);
    let storage = service.get_storage(request)?;

    let uid = match RequireJWT::extract_user_id(request) {
        Some(id) => id,
        None => {
            return Ok(HttpResponse::Unauthorized().json(ApiResponse::error_empty(
                ErrorCode::Unauthorized,
                "Unauthorized: missing user id",
            )));
        }
    };

    match storage.get_class_by_id(class_id).await {
        Ok(Some(class)) => {
            // 权限校验
            if let Err(resp) = check_class_access_permission(&storage, &role, uid, &class).await {
                return Ok(resp);
            }

            // 获取教师信息
            let teacher_info = match storage.get_user_by_id(class.teacher_id).await {
                Ok(Some(teacher)) => TeacherInfo {
                    id: teacher.id,
                    username: teacher.username,
                    display_name: teacher.display_name,
                },
                _ => TeacherInfo {
                    id: class.teacher_id,
                    username: "未知".to_string(),
                    display_name: None,
                },
            };

            // 获取成员数量
            let member_count = storage.count_class_members(class_id).await.unwrap_or(0);

            // 获取当前用户在班级中的角色
            let my_role: Option<ClassUserRole> = match role {
                Some(UserRole::Admin) => Some(ClassUserRole::Teacher), // 管理员视为教师权限
                Some(UserRole::Teacher) if class.teacher_id == uid => Some(ClassUserRole::Teacher),
                Some(UserRole::Teacher) | Some(UserRole::User) | None => {
                    // 查询用户在班级中的角色
                    storage
                        .get_class_user_by_user_id_and_class_id(uid, class_id)
                        .await
                        .ok()
                        .flatten()
                        .map(|cu| cu.role)
                }
            };

            let detail = ClassDetail {
                class,
                teacher: teacher_info,
                member_count,
                my_role,
            };

            Ok(HttpResponse::Ok().json(ApiResponse::success(
                detail,
                "Class information retrieved successfully",
            )))
        }
        Ok(None) => Ok(HttpResponse::NotFound().json(ApiResponse::error_empty(
            ErrorCode::ClassNotFound,
            "Class not found",
        ))),
        Err(e) => Ok(
            HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                ErrorCode::InternalServerError,
                format!("Failed to get class information: {e}"),
            )),
        ),
    }
}

/// 权限校验辅助函数
async fn check_class_access_permission(
    storage: &Arc<dyn Storage>,
    role: &Option<UserRole>,
    uid: i64,
    class: &Class,
) -> Result<(), HttpResponse> {
    match role {
        Some(UserRole::Admin) => Ok(()),
        Some(UserRole::Teacher) => {
            if class.teacher_id != uid {
                return Err(HttpResponse::Forbidden().json(ApiResponse::error_empty(
                    ErrorCode::ClassPermissionDenied,
                    "You do not have permission to view another teacher's class",
                )));
            }
            Ok(())
        }
        Some(UserRole::User) => {
            // 检查是否为班级成员
            match storage
                .get_class_user_by_user_id_and_class_id(uid, class.id)
                .await
            {
                Ok(Some(_)) => Ok(()),
                _ => Err(HttpResponse::Forbidden().json(ApiResponse::error_empty(
                    ErrorCode::ClassPermissionDenied,
                    "You are not a member of this class",
                ))),
            }
        }
        _ => Err(HttpResponse::Forbidden().json(ApiResponse::error_empty(
            ErrorCode::ClassPermissionDenied,
            "You do not have permission to view this class",
        ))),
    }
}

pub async fn get_class_by_code(
    service: &ClassService,
    request: &HttpRequest,
    code: String,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request)?;

    match storage.get_class_by_code(&code).await {
        Ok(Some(class)) => Ok(HttpResponse::Ok().json(ApiResponse::success(
            class,
            "Class information retrieved successfully",
        ))),
        Ok(None) => Ok(HttpResponse::NotFound().json(ApiResponse::error_empty(
            ErrorCode::ClassNotFound,
            "Class not found",
        ))),
        Err(e) => Ok(
            HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                ErrorCode::InternalServerError,
                format!("Failed to get class information: {e}"),
            )),
        ),
    }
}
