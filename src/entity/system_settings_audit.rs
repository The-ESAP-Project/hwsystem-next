//! 系统设置审计日志实体

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "system_settings_audit")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    pub setting_key: String,
    pub old_value: Option<String>,
    pub new_value: String,
    pub changed_by: i64,
    pub changed_at: i64,
    pub ip_address: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

// 从数据库模型转换为业务模型
impl Model {
    pub fn into_audit(self) -> crate::models::system::entities::SettingAudit {
        use crate::models::system::entities::SettingAudit;
        use chrono::{DateTime, Utc};

        SettingAudit {
            id: self.id,
            setting_key: self.setting_key,
            old_value: self.old_value,
            new_value: self.new_value,
            changed_by: self.changed_by,
            changed_at: DateTime::<Utc>::from_timestamp(self.changed_at, 0).unwrap_or_default(),
            ip_address: self.ip_address,
        }
    }
}
