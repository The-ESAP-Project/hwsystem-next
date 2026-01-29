//! 系统设置存储实现

use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, Order, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};

use crate::entity::prelude::{SystemSettings, SystemSettingsAudit};
use crate::errors::{HWSystemError, Result};
use crate::models::{
    common::PaginationInfo,
    system::{
        entities::SystemSetting, requests::SettingAuditQuery, responses::SettingAuditListResponse,
    },
};

use super::SeaOrmStorage;

impl SeaOrmStorage {
    /// 获取所有设置
    pub(crate) async fn list_all_settings_impl(&self) -> Result<Vec<SystemSetting>> {
        let settings = SystemSettings::find()
            .order_by(crate::entity::system_settings::Column::Key, Order::Asc)
            .all(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("获取设置列表失败: {e}")))?;

        Ok(settings.into_iter().map(|s| s.into_setting()).collect())
    }

    /// 通过 key 获取设置
    pub(crate) async fn get_setting_by_key_impl(&self, key: &str) -> Result<Option<SystemSetting>> {
        let setting = SystemSettings::find_by_id(key.to_string())
            .one(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("获取设置失败: {e}")))?;

        Ok(setting.map(|s| s.into_setting()))
    }

    /// 更新设置
    pub(crate) async fn update_setting_impl(
        &self,
        key: &str,
        value: &str,
        user_id: i64,
        ip_address: Option<String>,
    ) -> Result<SystemSetting> {
        let now = chrono::Utc::now().timestamp();

        // 获取当前设置
        let existing = SystemSettings::find_by_id(key.to_string())
            .one(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("获取设置失败: {e}")))?
            .ok_or_else(|| HWSystemError::not_found(format!("配置项不存在: {key}")))?;

        let old_value = existing.value.clone();

        // 更新设置
        let mut active_model: crate::entity::system_settings::ActiveModel = existing.into();
        active_model.value = Set(value.to_string());
        active_model.updated_at = Set(now);
        active_model.updated_by = Set(Some(user_id));

        let updated = active_model
            .update(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("更新设置失败: {e}")))?;

        // 创建审计日志
        let audit = crate::entity::system_settings_audit::ActiveModel {
            id: Set(0), // auto increment
            setting_key: Set(key.to_string()),
            old_value: Set(Some(old_value)),
            new_value: Set(value.to_string()),
            changed_by: Set(user_id),
            changed_at: Set(now),
            ip_address: Set(ip_address),
        };

        audit
            .insert(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("创建审计日志失败: {e}")))?;

        Ok(updated.into_setting())
    }

    /// 批量更新设置
    pub(crate) async fn batch_update_settings_impl(
        &self,
        updates: Vec<(String, String)>,
        user_id: i64,
        ip_address: Option<String>,
    ) -> Result<Vec<SystemSetting>> {
        let mut results = Vec::new();

        for (key, value) in updates {
            let setting = self
                .update_setting_impl(&key, &value, user_id, ip_address.clone())
                .await?;
            results.push(setting);
        }

        Ok(results)
    }

    /// 获取审计日志
    pub(crate) async fn list_setting_audits_impl(
        &self,
        query: SettingAuditQuery,
    ) -> Result<SettingAuditListResponse> {
        let (page, page_size) = query.pagination.normalized();

        let mut find = SystemSettingsAudit::find();

        if let Some(key) = &query.key {
            find = find.filter(crate::entity::system_settings_audit::Column::SettingKey.eq(key));
        }

        // 获取总数
        let total = find
            .clone()
            .count(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("统计审计日志失败: {e}")))?
            as i64;

        // 获取分页数据
        let audits = find
            .order_by(
                crate::entity::system_settings_audit::Column::ChangedAt,
                Order::Desc,
            )
            .offset((page - 1) * page_size)
            .limit(page_size)
            .all(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("获取审计日志失败: {e}")))?;

        let total_pages = (total as u64).div_ceil(page_size);

        Ok(SettingAuditListResponse {
            audits: audits.into_iter().map(|a| a.into_audit()).collect(),
            pagination: PaginationInfo {
                page: page as i64,
                page_size: page_size as i64,
                total,
                total_pages: total_pages as i64,
            },
        })
    }
}
