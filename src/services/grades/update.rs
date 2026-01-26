use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};

use super::GradeService;
use crate::middlewares::RequireJWT;
use crate::models::grades::requests::UpdateGradeRequest;
use crate::models::notifications::entities::{NotificationType, ReferenceType};
use crate::models::users::entities::UserRole;
use crate::models::{ApiResponse, ErrorCode};
use crate::services::notifications::trigger::send_notification;

pub async fn update_grade(
    service: &GradeService,
    request: &HttpRequest,
    grade_id: i64,
    req: UpdateGradeRequest,
    user_id: i64,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request);
    let user_role = RequireJWT::extract_user_role(request);

    // 获取评分信息
    let grade = match storage.get_grade_by_id(grade_id).await {
        Ok(Some(g)) => g,
        Ok(None) => {
            return Ok(HttpResponse::NotFound()
                .json(ApiResponse::error_empty(ErrorCode::GradeNotFound, "评分不存在")));
        }
        Err(e) => {
            return Ok(
                HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                    ErrorCode::InternalServerError,
                    format!("查询评分失败: {e}"),
                )),
            );
        }
    };

    // 权限检查：只有原评分者或管理员才能更新
    match user_role {
        Some(UserRole::Admin) => {} // 管理员可以更新任何评分
        Some(UserRole::Teacher) => {
            if grade.grader_id != user_id {
                return Ok(HttpResponse::Forbidden().json(ApiResponse::error_empty(
                    ErrorCode::Forbidden,
                    "只能更新自己创建的评分",
                )));
            }
        }
        _ => {
            return Ok(HttpResponse::Forbidden().json(ApiResponse::error_empty(
                ErrorCode::Forbidden,
                "没有更新评分的权限",
            )));
        }
    }

    match storage.update_grade(grade_id, req).await {
        Ok(Some(updated_grade)) => {
            // 异步通知学生
            let storage_clone = storage.clone();
            let g_id = updated_grade.id;
            let new_score = updated_grade.score;
            let submission_id = grade.submission_id;

            tokio::spawn(async move {
                // 获取提交和作业信息
                if let Ok(Some(submission)) =
                    storage_clone.get_submission_by_id(submission_id).await
                    && let Ok(Some(homework)) = storage_clone
                        .get_homework_by_id(submission.homework_id)
                        .await
                {
                    send_notification(
                        storage_clone,
                        submission.creator_id,
                        NotificationType::GradeUpdated,
                        format!("评分已更新：{}", homework.title),
                        Some(format!(
                            "您的作业「{}」评分已更新，新得分：{}",
                            homework.title, new_score
                        )),
                        Some(ReferenceType::Grade),
                        Some(g_id),
                    )
                    .await;
                }
            });

            Ok(HttpResponse::Ok().json(ApiResponse::success(updated_grade, "更新成功")))
        }
        Ok(None) => Ok(HttpResponse::NotFound()
            .json(ApiResponse::error_empty(ErrorCode::GradeNotFound, "评分不存在"))),
        Err(e) => Ok(
            HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                ErrorCode::GradeUpdateFailed,
                format!("更新评分失败: {e}"),
            )),
        ),
    }
}
