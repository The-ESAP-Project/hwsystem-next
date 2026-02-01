use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // ==================== 系统设置表 ====================
        manager
            .create_table(
                Table::create()
                    .table(SystemSettings::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SystemSettings::Key)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(SystemSettings::Value).text().not_null())
                    .col(
                        ColumnDef::new(SystemSettings::ValueType)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(SystemSettings::Description).text().null())
                    .col(
                        ColumnDef::new(SystemSettings::UpdatedAt)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SystemSettings::UpdatedBy)
                            .big_integer()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // ==================== 系统设置审计日志表 ====================
        manager
            .create_table(
                Table::create()
                    .table(SystemSettingsAudit::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SystemSettingsAudit::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(SystemSettingsAudit::SettingKey)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(SystemSettingsAudit::OldValue).text().null())
                    .col(
                        ColumnDef::new(SystemSettingsAudit::NewValue)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SystemSettingsAudit::ChangedBy)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SystemSettingsAudit::ChangedAt)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SystemSettingsAudit::IpAddress)
                            .string()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // 审计日志索引
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_system_settings_audit_setting_key")
                    .table(SystemSettingsAudit::Table)
                    .col(SystemSettingsAudit::SettingKey)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_system_settings_audit_changed_at")
                    .table(SystemSettingsAudit::Table)
                    .col(SystemSettingsAudit::ChangedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_system_settings_audit_changed_by")
                    .table(SystemSettingsAudit::Table)
                    .col(SystemSettingsAudit::ChangedBy)
                    .to_owned(),
            )
            .await?;

        // ==================== 插入默认配置 ====================
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let default_settings = [
            ("app.system_name", "作业管理系统", "string", "系统名称"),
            (
                "jwt.access_token_expiry",
                "60",
                "integer",
                "Access Token 有效期（分钟）",
            ),
            (
                "jwt.refresh_token_expiry",
                "7",
                "integer",
                "Refresh Token 有效期（天）",
            ),
            (
                "jwt.refresh_token_remember_me_expiry",
                "30",
                "integer",
                "记住我 Refresh Token 有效期（天）",
            ),
            (
                "upload.max_size",
                "52428800",
                "integer",
                "单文件最大大小（字节）",
            ),
            (
                "upload.allowed_types",
                r#"[".pdf",".doc",".docx",".xls",".xlsx",".ppt",".pptx",".txt",".zip",".rar",".7z",".jpg",".jpeg",".png",".gif",".bmp",".webp",".csv"]"#,
                "json_array",
                "允许上传的文件类型",
            ),
            (
                "cors.allowed_origins",
                r#"["http://localhost:3000","http://localhost:5173"]"#,
                "json_array",
                "允许的跨域来源",
            ),
            ("cors.max_age", "86400", "integer", "预检请求缓存时间（秒）"),
        ];

        for (key, value, value_type, description) in default_settings {
            let insert = Query::insert()
                .into_table(SystemSettings::Table)
                .columns([
                    SystemSettings::Key,
                    SystemSettings::Value,
                    SystemSettings::ValueType,
                    SystemSettings::Description,
                    SystemSettings::UpdatedAt,
                ])
                .values_panic([
                    key.into(),
                    value.into(),
                    value_type.into(),
                    description.into(),
                    now.into(),
                ])
                .to_owned();

            manager.exec_stmt(insert).await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SystemSettingsAudit::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(SystemSettings::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum SystemSettings {
    #[sea_orm(iden = "system_settings")]
    Table,
    Key,
    Value,
    ValueType,
    Description,
    UpdatedAt,
    UpdatedBy,
}

#[derive(DeriveIden)]
enum SystemSettingsAudit {
    #[sea_orm(iden = "system_settings_audit")]
    Table,
    Id,
    SettingKey,
    OldValue,
    NewValue,
    ChangedBy,
    ChangedAt,
    IpAddress,
}
