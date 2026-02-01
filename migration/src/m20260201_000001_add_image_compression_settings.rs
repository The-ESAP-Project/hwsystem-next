use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // 添加图片压缩配置项
        let compression_settings = [
            (
                "upload.client_compress_enabled",
                "false",
                "boolean",
                "是否启用前端图片压缩",
            ),
            (
                "upload.compress_threshold",
                "2097152",
                "integer",
                "触发压缩的文件大小阈值（字节，默认2MB）",
            ),
            (
                "upload.compress_quality",
                "0.85",
                "string",
                "压缩质量（0-1，推荐0.7-0.9）",
            ),
            (
                "upload.compress_max_width",
                "1920",
                "integer",
                "压缩后最大宽度（像素）",
            ),
            (
                "upload.compress_max_height",
                "1920",
                "integer",
                "压缩后最大高度（像素）",
            ),
        ];

        for (key, value, value_type, description) in compression_settings {
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
        // 删除图片压缩配置项
        let keys = [
            "upload.client_compress_enabled",
            "upload.compress_threshold",
            "upload.compress_quality",
            "upload.compress_max_width",
            "upload.compress_max_height",
        ];

        for key in keys {
            let delete = Query::delete()
                .from_table(SystemSettings::Table)
                .and_where(Expr::col(SystemSettings::Key).eq(key))
                .to_owned();

            manager.exec_stmt(delete).await?;
        }

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
}
