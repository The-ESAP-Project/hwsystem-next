//! 文件存储操作

use super::SeaOrmStorage;
use crate::config::AppConfig;
use crate::entity::files::{ActiveModel, Column, Entity as Files};
use crate::errors::{HWSystemError, Result};
use crate::models::files::entities::File;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, ExprTrait, QueryFilter, Set};
use uuid::Uuid;

impl SeaOrmStorage {
    /// 上传文件（创建文件记录）
    pub async fn upload_file_impl(
        &self,
        original_name: &str,
        stored_name: &str,
        file_size: &i64,
        file_type: &str,
        user_id: i64,
    ) -> Result<File> {
        let now = chrono::Utc::now().timestamp();
        let config = AppConfig::get();
        let upload_dir = &config.upload.dir;
        let file_path = format!("{}/{}", upload_dir, stored_name);
        let download_token = Uuid::new_v4().to_string();

        let model = ActiveModel {
            original_name: Set(original_name.to_string()),
            stored_name: Set(stored_name.to_string()),
            file_size: Set(*file_size),
            file_type: Set(file_type.to_string()),
            file_path: Set(file_path),
            download_token: Set(download_token),
            citation_count: Set(0),
            user_id: Set(Some(user_id)),
            created_at: Set(now),
            ..Default::default()
        };

        let result = model
            .insert(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("上传文件记录失败: {e}")))?;

        Ok(result.into_file())
    }

    /// 通过 token 获取文件
    pub async fn get_file_by_token_impl(&self, token: &str) -> Result<Option<File>> {
        let result = Files::find()
            .filter(Column::DownloadToken.eq(token))
            .one(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询文件失败: {e}")))?;

        Ok(result.map(|m| m.into_file()))
    }

    /// 通过 ID 获取文件
    pub async fn get_file_by_id_impl(&self, id: i64) -> Result<Option<File>> {
        let result = Files::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("查询文件失败: {e}")))?;

        Ok(result.map(|m| m.into_file()))
    }

    /// 增加文件引用计数
    pub async fn increment_file_citation_impl(&self, file_id: i64) -> Result<bool> {
        use sea_orm::sea_query::Expr;

        let result = Files::update_many()
            .col_expr(
                Column::CitationCount,
                Expr::col(Column::CitationCount).add(1),
            )
            .filter(Column::Id.eq(file_id))
            .exec(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("增加文件引用计数失败: {e}")))?;

        Ok(result.rows_affected > 0)
    }

    /// 减少文件引用计数
    pub async fn decrement_file_citation_impl(&self, file_id: i64) -> Result<bool> {
        use sea_orm::sea_query::Expr;

        let result = Files::update_many()
            .col_expr(
                Column::CitationCount,
                Expr::col(Column::CitationCount).sub(1),
            )
            .filter(Column::Id.eq(file_id))
            .filter(Column::CitationCount.gt(0)) // 防止负数
            .exec(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("减少文件引用计数失败: {e}")))?;

        Ok(result.rows_affected > 0)
    }
}
