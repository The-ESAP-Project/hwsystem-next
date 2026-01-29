use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};

use super::GradeService;
use crate::middlewares::RequireJWT;
use crate::models::grades::requests::CreateGradeRequest;
use crate::models::notifications::entities::{NotificationType, ReferenceType};
use crate::models::users::entities::UserRole;
use crate::models::{ApiResponse, ErrorCode};
use crate::services::StorageProvider;
use crate::services::notifications::trigger::send_notification;

pub async fn create_grade(
    service: &GradeService,
    request: &HttpRequest,
    grader_id: i64,
    req: CreateGradeRequest,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request)?;
    let user_role = RequireJWT::extract_user_role(request);

    // 检查提交是否存在
    let submission = match storage.get_submission_by_id(req.submission_id).await {
        Ok(Some(sub)) => sub,
        Ok(None) => {
            return Ok(HttpResponse::NotFound().json(ApiResponse::error_empty(
                ErrorCode::SubmissionNotFound,
                "提交不存在",
            )));
        }
        Err(e) => {
            return Ok(
                HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                    ErrorCode::InternalServerError,
                    format!("查询提交失败: {e}"),
                )),
            );
        }
    };

    // 获取作业信息以确定班级
    let homework = match storage.get_homework_by_id(submission.homework_id).await {
        Ok(Some(hw)) => hw,
        Ok(None) => {
            return Ok(HttpResponse::NotFound().json(ApiResponse::error_empty(
                ErrorCode::HomeworkNotFound,
                "作业不存在",
            )));
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

    // 获取班级信息
    let class = match storage.get_class_by_id(homework.class_id).await {
        Ok(Some(cls)) => cls,
        Ok(None) => {
            return Ok(HttpResponse::NotFound().json(ApiResponse::error_empty(
                ErrorCode::ClassNotFound,
                "班级不存在",
            )));
        }
        Err(e) => {
            return Ok(
                HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                    ErrorCode::InternalServerError,
                    format!("查询班级失败: {e}"),
                )),
            );
        }
    };

    // 权限检查：只有该班级的教师或管理员才能评分
    match user_role {
        Some(UserRole::Admin) => {} // 管理员可以评任何提交
        Some(UserRole::Teacher) => {
            if class.teacher_id != grader_id {
                return Ok(HttpResponse::Forbidden().json(ApiResponse::error_empty(
                    ErrorCode::Forbidden,
                    "只能对自己班级的提交进行评分",
                )));
            }
        }
        _ => {
            return Ok(HttpResponse::Forbidden().json(ApiResponse::error_empty(
                ErrorCode::Forbidden,
                "没有评分权限",
            )));
        }
    }

    // 检查是否已评分
    match storage.get_grade_by_submission_id(req.submission_id).await {
        Ok(Some(_)) => {
            return Ok(HttpResponse::Conflict().json(ApiResponse::error_empty(
                ErrorCode::Conflict,
                "该提交已评分，请使用更新接口",
            )));
        }
        Err(e) => {
            return Ok(
                HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                    ErrorCode::InternalServerError,
                    format!("查询评分失败: {e}"),
                )),
            );
        }
        _ => {}
    }

    match storage.create_grade(grader_id, req).await {
        Ok(grade) => {
            // 异步通知学生
            let storage_clone = storage.clone();
            let grade_id = grade.id;
            let student_id = submission.creator_id;
            let score = grade.score;
            let hw_title = homework.title.clone();

            tokio::spawn(async move {
                send_notification(
                    storage_clone,
                    student_id,
                    NotificationType::GradeReceived,
                    format!("作业已评分：{}", hw_title),
                    Some(format!("您的作业「{}」已评分，得分：{}", hw_title, score)),
                    Some(ReferenceType::Grade),
                    Some(grade_id),
                )
                .await;
            });

            Ok(HttpResponse::Created().json(ApiResponse::success(grade, "评分成功")))
        }
        Err(e) => Ok(
            HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                ErrorCode::GradeCreateFailed,
                format!("创建评分失败: {e}"),
            )),
        ),
    }
}
