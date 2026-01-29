use std::collections::HashMap;
use std::sync::Arc;

use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};

use super::ClassService;
use crate::{
    middlewares::RequireJWT,
    models::{
        ApiResponse, ErrorCode,
        classes::{
            requests::ClassListQuery,
            responses::{ClassDetail, ClassDetailListResponse, ClassListResponse, TeacherInfo},
        },
        users::entities::UserRole,
    },
    storage::Storage,
};

pub async fn list_classes(
    service: &ClassService,
    request: &HttpRequest,
    query: ClassListQuery,
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

    let mut list_query = ClassListQuery {
        pagination: query.pagination.clone(),
        teacher_id: None,
        search: query.search,
    };

    // 权限校验 - 学生走特殊路径
    match role {
        Some(UserRole::Admin) => {
            // 管理员可查全部班级
        }
        Some(UserRole::Teacher) => {
            // 教师只能查询自己的班级
            list_query.teacher_id = Some(uid);
        }
        Some(UserRole::User) => {
            // 学生查询自己加入的班级
            return list_user_classes_with_details(&storage, uid, list_query).await;
        }
        _ => {
            return Ok(HttpResponse::Unauthorized().json(ApiResponse::error_empty(
                ErrorCode::Unauthorized,
                "Unauthorized: missing required role",
            )));
        }
    }

    // 教师/管理员路径
    let result = storage.list_classes_with_pagination(list_query).await;
    match result {
        Ok(response) => {
            let detail_response = enrich_class_list(&storage, response).await;
            Ok(HttpResponse::Ok().json(ApiResponse::success(
                detail_response,
                "Class list retrieved successfully",
            )))
        }
        Err(e) => Ok(
            HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                ErrorCode::InternalServerError,
                format!("Failed to retrieve class list: {e}"),
            )),
        ),
    }
}

/// 学生获取自己加入的班级列表（带详情）
async fn list_user_classes_with_details(
    storage: &Arc<dyn Storage>,
    user_id: i64,
    query: ClassListQuery,
) -> ActixResult<HttpResponse> {
    let result = storage
        .list_user_classes_with_pagination(user_id, query)
        .await;

    match result {
        Ok(response) => {
            let detail_response = enrich_class_list(storage, response).await;
            Ok(HttpResponse::Ok().json(ApiResponse::success(
                detail_response,
                "User class list retrieved successfully",
            )))
        }
        Err(e) => Ok(
            HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                ErrorCode::InternalServerError,
                format!("Failed to retrieve user class list: {e}"),
            )),
        ),
    }
}

/// 为班级列表添加教师信息和成员数量
async fn enrich_class_list(
    storage: &Arc<dyn Storage>,
    response: ClassListResponse,
) -> ClassDetailListResponse {
    // 收集所有教师 ID
    let teacher_ids: Vec<i64> = response
        .items
        .iter()
        .map(|c| c.teacher_id)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    // 批量获取教师信息
    let mut teacher_map: HashMap<i64, TeacherInfo> = HashMap::new();
    for teacher_id in teacher_ids {
        if let Ok(Some(teacher)) = storage.get_user_by_id(teacher_id).await {
            teacher_map.insert(
                teacher_id,
                TeacherInfo {
                    id: teacher.id,
                    username: teacher.username,
                    display_name: teacher.display_name,
                },
            );
        }
    }

    // 组装详情列表
    let mut items = Vec::with_capacity(response.items.len());
    for class in response.items {
        let teacher = teacher_map
            .get(&class.teacher_id)
            .cloned()
            .unwrap_or_else(|| TeacherInfo {
                id: class.teacher_id,
                username: "未知".to_string(),
                display_name: None,
            });

        let member_count = storage.count_class_members(class.id).await.unwrap_or(0);

        items.push(ClassDetail {
            class,
            teacher,
            member_count,
            my_role: None, // 列表接口不填充 my_role，获取详情时才查询
        });
    }

    ClassDetailListResponse {
        pagination: response.pagination,
        items,
    }
}
