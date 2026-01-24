use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};

use super::SubmissionService;
use crate::models::submissions::requests::CreateSubmissionRequest;
use crate::models::users::entities::UserRole;
use crate::models::{ApiResponse, ErrorCode};

pub async fn create_submission(
    service: &SubmissionService,
    request: &HttpRequest,
    creator_id: i64,
    creator_role: UserRole,
    req: CreateSubmissionRequest,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request);

    // 检查作业是否存在并获取班级信息
    let homework = match storage.get_homework_by_id(req.homework_id).await {
        Ok(Some(hw)) => hw,
        Ok(None) => {
            return Ok(HttpResponse::NotFound()
                .json(ApiResponse::error_empty(ErrorCode::NotFound, "作业不存在")));
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

    // 验证用户是否为该作业所属班级的成员（管理员除外）
    if creator_role != UserRole::Admin {
        match storage
            .get_class_user_by_user_id_and_class_id(creator_id, homework.class_id)
            .await
        {
            Ok(Some(_)) => {
                // 用户是班级成员，允许提交
            }
            Ok(None) => {
                return Ok(HttpResponse::Forbidden().json(ApiResponse::error_empty(
                    ErrorCode::ClassPermissionDenied,
                    "您不是该班级成员，无法提交作业",
                )));
            }
            Err(e) => {
                return Ok(
                    HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                        ErrorCode::InternalServerError,
                        format!("验证班级成员资格失败: {e}"),
                    )),
                );
            }
        }
    }

    match storage.create_submission(creator_id, req).await {
        Ok(submission) => {
            Ok(HttpResponse::Created().json(ApiResponse::success(submission, "提交成功")))
        }
        Err(e) => Ok(
            HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                ErrorCode::InternalServerError,
                format!("创建提交失败: {e}"),
            )),
        ),
    }
}
