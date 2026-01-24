use std::collections::HashMap;

use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};
use tracing::error;

use crate::{
    models::{
        ApiResponse, ErrorCode,
        class_users::{
            requests::{ClassUserListParams, ClassUserQuery},
            responses::{ClassUserDetail, ClassUserDetailListResponse, UserInfo},
        },
    },
    services::ClassUserService,
};

pub async fn list_class_users_with_pagination(
    service: &ClassUserService,
    request: &HttpRequest,
    class_id: i64,
    query: ClassUserListParams,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request);

    let list_query = ClassUserQuery {
        page: Some(query.pagination.page),
        size: Some(query.pagination.size),
        search: query.search,
    };

    match storage
        .list_class_users_with_pagination(class_id, list_query)
        .await
    {
        Ok(response) => {
            // 收集所有用户 ID
            let user_ids: Vec<i64> = response.items.iter().map(|cu| cu.user_id).collect();

            // 批量获取用户信息
            let mut user_map: HashMap<i64, UserInfo> = HashMap::new();
            for user_id in user_ids {
                if let Ok(Some(user)) = storage.get_user_by_id(user_id).await {
                    user_map.insert(
                        user_id,
                        UserInfo {
                            id: user.id,
                            username: user.username,
                            display_name: user.display_name,
                            avatar_url: user.avatar_url,
                        },
                    );
                }
            }

            // 组装详情列表
            let items: Vec<ClassUserDetail> = response
                .items
                .into_iter()
                .map(|cu| {
                    let user = user_map
                        .get(&cu.user_id)
                        .cloned()
                        .unwrap_or_else(|| UserInfo {
                            id: cu.user_id,
                            username: "未知用户".to_string(),
                            display_name: None,
                            avatar_url: None,
                        });

                    ClassUserDetail {
                        id: cu.id,
                        class_id: cu.class_id,
                        user_id: cu.user_id,
                        role: cu.role,
                        joined_at: cu.joined_at,
                        user,
                    }
                })
                .collect();

            let detail_response = ClassUserDetailListResponse {
                pagination: response.pagination,
                items,
            };

            Ok(HttpResponse::Ok().json(ApiResponse::success(
                detail_response,
                "Class users retrieved successfully",
            )))
        }
        Err(err) => {
            error!("Failed to retrieve class users: {}", err);
            Ok(
                HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                    ErrorCode::InternalServerError,
                    "Failed to retrieve class users",
                )),
            )
        }
    }
}
