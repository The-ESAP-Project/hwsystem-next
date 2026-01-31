use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};

use crate::middlewares::require_class_role::class_user_cache_key;
use crate::models::class_users::entities::ClassUserRole;
use crate::models::notifications::entities::{NotificationType, ReferenceType};
use crate::services::notifications::trigger::send_notification;
use crate::{
    middlewares::RequireJWT,
    models::{
        ApiResponse, ErrorCode,
        class_users::requests::UpdateClassUserRequest,
        classes::entities::Class,
        users::entities::{User, UserRole},
    },
    services::{CacheProvider, ClassUserService, StorageProvider},
};

pub async fn update_class_user(
    service: &ClassUserService,
    request: &HttpRequest,
    class_id: i64,
    user_id: i64,
    update_data: UpdateClassUserRequest,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request)?;

    let user = match RequireJWT::extract_user_claims(request) {
        Some(user) => user,
        _ => {
            return Ok(HttpResponse::Unauthorized().json(ApiResponse::error_empty(
                ErrorCode::Unauthorized,
                "Unauthorized: missing user claims",
            )));
        }
    };

    // 查询班级信息
    let class = match storage.get_class_by_id(class_id).await {
        Ok(Some(class)) => class,
        Ok(None) => {
            return Ok(HttpResponse::NotFound().json(ApiResponse::error_empty(
                ErrorCode::ClassNotFound,
                "Class not found",
            )));
        }
        Err(e) => {
            return Ok(
                HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                    ErrorCode::InternalServerError,
                    format!("Failed to get class information: {e}"),
                )),
            );
        }
    };

    // 权限校验
    if let Err(resp) = check_update_class_user_permissions(&user, &class) {
        return Ok(resp);
    }

    // 获取原角色用于比较
    let old_role = match storage
        .get_class_user_by_user_id_and_class_id(user_id, class_id)
        .await
    {
        Ok(Some(cu)) => Some(cu.role),
        _ => None,
    };

    match storage
        .update_class_user(class_id, user_id, update_data.clone())
        .await
    {
        Ok(Some(class_user)) => {
            // 失效缓存
            if let Some(cache) = service.get_cache(request) {
                cache.remove(&class_user_cache_key(user_id, class_id)).await;
            }

            // 检查角色是否变化，如果变化则发送通知
            if let Some(new_role) = &update_data.role
                && old_role.as_ref() != Some(new_role)
            {
                let storage_clone = storage.clone();
                let class_name = class.name.clone();
                let role_name = match new_role {
                    ClassUserRole::Student => "学生",
                    ClassUserRole::ClassRepresentative => "课代表",
                    ClassUserRole::Teacher => "教师",
                };

                tokio::spawn(async move {
                    send_notification(
                        storage_clone,
                        user_id,
                        NotificationType::ClassRoleChanged,
                        format!("班级角色变更：{}", class_name),
                        Some(format!(
                            "您在班级「{}」的角色已变更为：{}",
                            class_name, role_name
                        )),
                        Some(ReferenceType::Class),
                        Some(class_id),
                    )
                    .await;
                });
            }

            Ok(HttpResponse::Ok().json(ApiResponse::success(
                class_user,
                "Class user updated successfully",
            )))
        }
        Ok(None) => Ok(HttpResponse::NotFound().json(ApiResponse::error_empty(
            ErrorCode::ClassUserNotFound,
            "Class user not found",
        ))),
        Err(e) => Ok(
            HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                ErrorCode::InternalServerError,
                format!("Failed to get class user: {e}"),
            )),
        ),
    }
}

fn check_update_class_user_permissions(user: &User, class: &Class) -> Result<(), HttpResponse> {
    match user.role {
        UserRole::Admin => Ok(()),
        UserRole::Teacher if class.teacher_id == user.id => Ok(()),
        _ => Err(HttpResponse::Forbidden().json(ApiResponse::error_empty(
            ErrorCode::ClassPermissionDenied,
            "You do not have permission to update class users",
        ))),
    }
}
