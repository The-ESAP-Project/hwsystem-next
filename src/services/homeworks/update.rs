use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};

use super::HomeworkService;
use crate::middlewares::RequireJWT;
use crate::models::homeworks::requests::UpdateHomeworkRequest;
use crate::models::notifications::entities::{NotificationType, ReferenceType};
use crate::models::users::entities::UserRole;
use crate::models::{ApiResponse, ErrorCode};
use crate::services::notifications::trigger::{get_class_student_ids, send_notifications};

pub async fn update_homework(
    service: &HomeworkService,
    request: &HttpRequest,
    homework_id: i64,
    req: UpdateHomeworkRequest,
    user_id: i64,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request);
    let user_role = RequireJWT::extract_user_role(request);

    // 获取作业信息
    let homework = match storage.get_homework_by_id(homework_id).await {
        Ok(Some(hw)) => hw,
        Ok(None) => {
            return Ok(HttpResponse::NotFound()
                .json(ApiResponse::error_empty(ErrorCode::HomeworkNotFound, "作业不存在")));
        }
        Err(e) => {
            return Ok(
                HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                    ErrorCode::InternalServerError,
                    format!("查询作业失败: {e}"),
                )),
            );
        }
    };

    // 权限检查：只有作业创建者或管理员才能更新
    match user_role {
        Some(UserRole::Admin) => {} // 管理员可以更新任何作业
        Some(UserRole::Teacher) => {
            if homework.created_by != user_id {
                return Ok(HttpResponse::Forbidden().json(ApiResponse::error_empty(
                    ErrorCode::Forbidden,
                    "只能更新自己创建的作业",
                )));
            }
        }
        _ => {
            return Ok(HttpResponse::Forbidden().json(ApiResponse::error_empty(
                ErrorCode::Forbidden,
                "没有更新作业的权限",
            )));
        }
    }

    match storage.update_homework(homework_id, req, user_id).await {
        Ok(Some(updated_homework)) => {
            // 异步发送通知给班级学生
            let storage_clone = storage.clone();
            let hw_id = updated_homework.id;
            let class_id = homework.class_id;
            let title = updated_homework.title.clone();

            tokio::spawn(async move {
                let student_ids = get_class_student_ids(&storage_clone, class_id).await;
                send_notifications(
                    storage_clone,
                    student_ids,
                    NotificationType::HomeworkUpdated,
                    format!("作业更新：{}", title),
                    Some(format!("作业「{}」已更新，请查看最新内容", title)),
                    Some(ReferenceType::Homework),
                    Some(hw_id),
                )
                .await;
            });

            Ok(HttpResponse::Ok().json(ApiResponse::success(updated_homework, "更新成功")))
        }
        Ok(None) => Ok(HttpResponse::NotFound()
            .json(ApiResponse::error_empty(ErrorCode::HomeworkNotFound, "作业不存在"))),
        Err(e) => Ok(
            HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                ErrorCode::HomeworkUpdateFailed,
                format!("更新作业失败: {e}"),
            )),
        ),
    }
}
