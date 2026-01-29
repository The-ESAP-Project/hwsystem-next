//! 通知存储操作

use super::SeaOrmStorage;
use crate::entity::notifications::{ActiveModel, Column, Entity as Notifications};
use crate::errors::{HWSystemError, Result};
use crate::models::{
    PaginationInfo,
    notifications::{
        entities::Notification,
        requests::{CreateNotificationRequest, NotificationListQuery},
        responses::NotificationListResponse,
    },
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Set,
};

impl SeaOrmStorage {
    /// 创建通知
    pub async fn create_notification_impl(
        &self,
        req: CreateNotificationRequest,
    ) -> Result<Notification> {
        let now = chrono::Utc::now().timestamp();

        let model = ActiveModel {
            user_id: Set(req.user_id),
            notification_type: Set(req.notification_type),
            title: Set(req.title),
            content: Set(req.content),
            reference_type: Set(req.reference_type),
            reference_id: Set(req.reference_id),
            is_read: Set(false),
            created_at: Set(now),
            ..Default::default()
        };

        let result = model
            .insert(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("创建通知失败: {e}")))?;

        Ok(result.into_notification())
    }

    /// 批量创建通知
    pub async fn create_notifications_batch_impl(
        &self,
        reqs: Vec<CreateNotificationRequest>,
    ) -> Result<Vec<Notification>> {
        let now = chrono::Utc::now().timestamp();
        let mut notifications = Vec::new();

        for req in reqs {
            let model = ActiveModel {
                user_id: Set(req.user_id),
                notification_type: Set(req.notification_type),
                title: Set(req.title),
                content: Set(req.content),
                reference_type: Set(req.reference_type),
                reference_id: Set(req.reference_id),
                is_read: Set(false),
                created_at: Set(now),
                ..Default::default()
            };

            let result = model
                .insert(&self.db)
                .await
                .map_err(|e| HWSystemError::database_operation(format!("批量创建通知失败: {e}")))?;

            notifications.push(result.into_notification());
        }

        Ok(notifications)
    }

    /// 通过 ID 获取通知
    pub async fn get_notification_by_id_impl(
        &self,
        notification_id: i64,
    ) -> Result<Option<Notification>> {
        let result = Notifications::find_by_id(notification_id)
            .one(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询通知失败: {e}")))?;

        Ok(result.map(|m| m.into_notification()))
    }

    /// 列出用户通知（分页）
    pub async fn list_notifications_with_pagination_impl(
        &self,
        user_id: i64,
        query: NotificationListQuery,
    ) -> Result<NotificationListResponse> {
        let (page, page_size) = query.pagination.normalized();

        let mut select = Notifications::find().filter(Column::UserId.eq(user_id));

        // 未读筛选
        if let Some(true) = query.unread_only {
            select = select.filter(Column::IsRead.eq(false));
        }

        // 排序
        select = select.order_by_desc(Column::CreatedAt);

        // 分页查询
        let paginator = select.paginate(&self.db, page_size);
        let total = paginator
            .num_items()
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询通知总数失败: {e}")))?;

        let pages = paginator
            .num_pages()
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询通知页数失败: {e}")))?;

        let notifications = paginator
            .fetch_page(page - 1)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询通知列表失败: {e}")))?;

        Ok(NotificationListResponse {
            items: notifications
                .into_iter()
                .map(|m| m.into_notification())
                .collect(),
            pagination: PaginationInfo {
                page: page as i64,
                page_size: page_size as i64,
                total: total as i64,
                total_pages: pages as i64,
            },
        })
    }

    /// 获取用户未读通知数量
    pub async fn get_unread_notification_count_impl(&self, user_id: i64) -> Result<i64> {
        let count = Notifications::find()
            .filter(Column::UserId.eq(user_id))
            .filter(Column::IsRead.eq(false))
            .count(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询未读通知数量失败: {e}")))?;

        Ok(count as i64)
    }

    /// 标记通知为已读
    pub async fn mark_notification_as_read_impl(&self, notification_id: i64) -> Result<bool> {
        let result = Notifications::update_many()
            .col_expr(Column::IsRead, sea_orm::sea_query::Expr::value(true))
            .filter(Column::Id.eq(notification_id))
            .exec(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("标记通知已读失败: {e}")))?;

        Ok(result.rows_affected > 0)
    }

    /// 标记用户所有通知为已读
    pub async fn mark_all_notifications_as_read_impl(&self, user_id: i64) -> Result<i64> {
        let result = Notifications::update_many()
            .col_expr(Column::IsRead, sea_orm::sea_query::Expr::value(true))
            .filter(Column::UserId.eq(user_id))
            .filter(Column::IsRead.eq(false))
            .exec(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("标记全部通知已读失败: {e}")))?;

        Ok(result.rows_affected as i64)
    }

    /// 删除通知
    pub async fn delete_notification_impl(&self, notification_id: i64) -> Result<bool> {
        let result = Notifications::delete_by_id(notification_id)
            .exec(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("删除通知失败: {e}")))?;

        Ok(result.rows_affected > 0)
    }
}
