pub mod count;
pub mod delete;
pub mod list;
pub mod read;
pub mod trigger;

use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};
use std::sync::Arc;

use crate::models::notifications::requests::NotificationListQuery;
use crate::models::{ApiResponse, ErrorCode};
use crate::storage::Storage;

pub struct NotificationService {
    storage: Option<Arc<dyn Storage>>,
}

impl NotificationService {
    pub fn new_lazy() -> Self {
        Self { storage: None }
    }

    pub(crate) fn get_storage(
        &self,
        request: &HttpRequest,
    ) -> Result<Arc<dyn Storage>, actix_web::Error> {
        if let Some(storage) = &self.storage {
            Ok(storage.clone())
        } else {
            request
                .app_data::<actix_web::web::Data<Arc<dyn Storage>>>()
                .map(|data| data.get_ref().clone())
                .ok_or_else(|| {
                    actix_web::error::InternalError::from_response(
                        "Storage service unavailable",
                        HttpResponse::InternalServerError().json(ApiResponse::<()>::error_empty(
                            ErrorCode::InternalServerError,
                            "Storage service unavailable",
                        )),
                    )
                    .into()
                })
        }
    }

    /// 列出用户通知
    pub async fn list_notifications(
        &self,
        request: &HttpRequest,
        user_id: i64,
        query: NotificationListQuery,
    ) -> ActixResult<HttpResponse> {
        list::list_notifications(self, request, user_id, query).await
    }

    /// 获取未读通知数量
    pub async fn get_unread_count(
        &self,
        request: &HttpRequest,
        user_id: i64,
    ) -> ActixResult<HttpResponse> {
        count::get_unread_count(self, request, user_id).await
    }

    /// 标记通知为已读
    pub async fn mark_as_read(
        &self,
        request: &HttpRequest,
        notification_id: i64,
    ) -> ActixResult<HttpResponse> {
        read::mark_as_read(self, request, notification_id).await
    }

    /// 标记所有通知为已读
    pub async fn mark_all_as_read(
        &self,
        request: &HttpRequest,
        user_id: i64,
    ) -> ActixResult<HttpResponse> {
        read::mark_all_as_read(self, request, user_id).await
    }

    /// 删除通知
    pub async fn delete_notification(
        &self,
        request: &HttpRequest,
        notification_id: i64,
    ) -> ActixResult<HttpResponse> {
        delete::delete_notification(self, request, notification_id).await
    }
}
