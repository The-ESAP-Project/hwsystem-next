//! 提交概览服务（按学生聚合）

use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};

use crate::middlewares::RequireJWT;
use crate::models::class_users::entities::ClassUserRole;
use crate::models::users::entities::UserRole;
use crate::models::{ApiResponse, ErrorCode};
use crate::services::submissions::SubmissionService;

/// 获取作业提交概览（按学生聚合）
///
/// 权限：班级教师、课代表和管理员
/// - 教师和管理员可以看到成绩
/// - 课代表只能看到提交状态，不能看到成绩
///
/// 参数：
/// - `graded`: 筛选是否已批改，true=已批改，false=待批改，None=全部
pub async fn get_submission_summary(
    service: &SubmissionService,
    request: &HttpRequest,
    homework_id: i64,
    page: Option<i64>,
    size: Option<i64>,
    graded: Option<bool>,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request);

    // 获取当前用户信息
    let current_user = match RequireJWT::extract_user_claims(request) {
        Some(user) => user,
        None => {
            return Ok(HttpResponse::Unauthorized()
                .json(ApiResponse::error_empty(ErrorCode::Unauthorized, "未登录")));
        }
    };

    // 验证作业存在
    let homework = storage
        .get_homework_by_id(homework_id)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("查询作业失败: {e}")))?;

    let homework = match homework {
        Some(hw) => hw,
        None => {
            return Ok(
                HttpResponse::NotFound().json(ApiResponse::<()>::error_empty(
                    ErrorCode::NotFound,
                    "作业不存在",
                )),
            );
        }
    };

    // 权限检查：Admin 直接放行，其他用户检查班级角色
    // 同时确定是否可以查看成绩（教师和管理员可以，课代表不可以）
    let include_grades = if current_user.role == UserRole::Admin {
        true // Admin 可以查看成绩
    } else {
        // 非 Admin 用户需要验证班级成员资格
        let class_user = match storage
            .get_class_user_by_user_id_and_class_id(current_user.id, homework.class_id)
            .await
        {
            Ok(Some(cu)) => cu,
            Ok(None) => {
                return Ok(
                    HttpResponse::Forbidden().json(ApiResponse::<()>::error_empty(
                        ErrorCode::ClassPermissionDenied,
                        "您不是该班级成员",
                    )),
                );
            }
            Err(e) => {
                return Ok(HttpResponse::InternalServerError().json(
                    ApiResponse::<()>::error_empty(
                        ErrorCode::InternalServerError,
                        format!("查询班级成员失败: {e}"),
                    ),
                ));
            }
        };

        // 验证是教师或课代表
        if class_user.role != ClassUserRole::Teacher
            && class_user.role != ClassUserRole::ClassRepresentative
        {
            return Ok(
                HttpResponse::Forbidden().json(ApiResponse::<()>::error_empty(
                    ErrorCode::ClassPermissionDenied,
                    "只有教师或课代表可以查看提交概览",
                )),
            );
        }

        // 教师可以查看成绩，课代表不可以
        class_user.role == ClassUserRole::Teacher
    };

    // 获取提交概览
    let page = page.unwrap_or(1);
    let size = size.unwrap_or(20);

    let summary = storage
        .get_submission_summary(homework_id, page, size, include_grades, graded)
        .await
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("查询提交概览失败: {e}"))
        })?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(summary, "查询成功")))
}

/// 获取某学生某作业的所有提交版本（教师视角）
///
/// 权限：班级教师、课代表和管理员
/// - 教师和管理员可以看到成绩
/// - 课代表只能看到提交状态，不能看到成绩
pub async fn list_user_submissions_for_teacher(
    service: &SubmissionService,
    request: &HttpRequest,
    homework_id: i64,
    user_id: i64,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request);

    // 获取当前用户信息
    let current_user = match RequireJWT::extract_user_claims(request) {
        Some(user) => user,
        None => {
            return Ok(HttpResponse::Unauthorized()
                .json(ApiResponse::error_empty(ErrorCode::Unauthorized, "未登录")));
        }
    };

    // 验证作业存在
    let homework = storage
        .get_homework_by_id(homework_id)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("查询作业失败: {e}")))?;

    let homework = match homework {
        Some(hw) => hw,
        None => {
            return Ok(
                HttpResponse::NotFound().json(ApiResponse::<()>::error_empty(
                    ErrorCode::NotFound,
                    "作业不存在",
                )),
            );
        }
    };

    // 权限检查：Admin 直接放行，其他用户检查班级角色
    // 同时确定是否可以查看成绩（教师和管理员可以，课代表不可以）
    let include_grades = if current_user.role == UserRole::Admin {
        true // Admin 可以查看成绩
    } else {
        // 非 Admin 用户需要验证班级成员资格
        let class_user = match storage
            .get_class_user_by_user_id_and_class_id(current_user.id, homework.class_id)
            .await
        {
            Ok(Some(cu)) => cu,
            Ok(None) => {
                return Ok(
                    HttpResponse::Forbidden().json(ApiResponse::<()>::error_empty(
                        ErrorCode::ClassPermissionDenied,
                        "您不是该班级成员",
                    )),
                );
            }
            Err(e) => {
                return Ok(HttpResponse::InternalServerError().json(
                    ApiResponse::<()>::error_empty(
                        ErrorCode::InternalServerError,
                        format!("查询班级成员失败: {e}"),
                    ),
                ));
            }
        };

        // 验证是教师或课代表
        if class_user.role != ClassUserRole::Teacher
            && class_user.role != ClassUserRole::ClassRepresentative
        {
            return Ok(
                HttpResponse::Forbidden().json(ApiResponse::<()>::error_empty(
                    ErrorCode::ClassPermissionDenied,
                    "只有教师或课代表可以查看学生提交历史",
                )),
            );
        }

        // 教师可以查看成绩，课代表不可以
        class_user.role == ClassUserRole::Teacher
    };

    // 获取学生提交历史
    let submissions = storage
        .list_user_submissions_for_teacher(homework_id, user_id, include_grades)
        .await
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("查询提交历史失败: {e}"))
        })?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(
        crate::models::submissions::responses::UserSubmissionHistoryResponse { items: submissions },
        "查询成功",
    )))
}
