use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};

use super::HomeworkService;
use crate::middlewares::RequireJWT;
use crate::models::files::responses::FileInfo;
use crate::models::homeworks::responses::HomeworkCreator;
use crate::models::users::entities::UserRole;
use crate::models::{ApiResponse, ErrorCode, homeworks::responses::HomeworkDetail};
use crate::services::StorageProvider;

pub async fn get_homework(
    service: &HomeworkService,
    request: &HttpRequest,
    homework_id: i64,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request)?;

    // 获取当前用户信息
    let current_user = match RequireJWT::extract_user_claims(request) {
        Some(user) => user,
        None => {
            return Ok(HttpResponse::Unauthorized()
                .json(ApiResponse::error_empty(ErrorCode::Unauthorized, "未登录")));
        }
    };

    match storage.get_homework_by_id(homework_id).await {
        Ok(Some(homework)) => {
            // 权限验证：管理员直接放行，否则验证班级成员资格
            if current_user.role != UserRole::Admin {
                match storage
                    .get_class_user_by_user_id_and_class_id(current_user.id, homework.class_id)
                    .await
                {
                    Ok(Some(_)) => {
                        // 用户是班级成员，允许访问
                    }
                    Ok(None) => {
                        return Ok(HttpResponse::Forbidden().json(ApiResponse::error_empty(
                            ErrorCode::ClassPermissionDenied,
                            "您不是该班级成员，无权查看此作业",
                        )));
                    }
                    Err(e) => {
                        return Ok(HttpResponse::InternalServerError().json(
                            ApiResponse::error_empty(
                                ErrorCode::InternalServerError,
                                format!("验证班级成员资格失败: {e}"),
                            ),
                        ));
                    }
                }
            }

            // 批量获取附件完整信息
            let file_ids = storage
                .get_homework_file_ids(homework_id)
                .await
                .unwrap_or_default();

            let files_map = storage
                .get_files_by_ids(&file_ids)
                .await
                .unwrap_or_default();

            let attachments: Vec<FileInfo> = file_ids
                .iter()
                .filter_map(|id| {
                    files_map.get(id).map(|file| FileInfo {
                        download_token: file.download_token.clone(),
                        original_name: file.original_name.clone(),
                        file_size: file.file_size,
                        file_type: file.file_type.clone(),
                    })
                })
                .collect();

            // 获取创建者信息
            let creator = match storage.get_user_by_id(homework.created_by).await {
                Ok(Some(user)) => Some(HomeworkCreator {
                    id: user.id,
                    username: user.username,
                    display_name: user.display_name,
                    avatar_url: user.avatar_url,
                }),
                _ => None,
            };

            let detail = HomeworkDetail {
                homework,
                attachments,
                creator,
            };
            Ok(HttpResponse::Ok().json(ApiResponse::success(detail, "查询成功")))
        }
        Ok(None) => Ok(HttpResponse::NotFound().json(ApiResponse::error_empty(
            ErrorCode::HomeworkNotFound,
            "作业不存在",
        ))),
        Err(e) => Ok(
            HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                ErrorCode::InternalServerError,
                format!("查询作业失败: {e}"),
            )),
        ),
    }
}
