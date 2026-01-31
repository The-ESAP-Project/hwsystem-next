//! 文件存储操作

use super::SeaOrmStorage;
use crate::config::AppConfig;
use crate::entity::files::{ActiveModel, Column, Entity as Files};
use crate::errors::{HWSystemError, Result};
use crate::models::files::entities::File;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, ExprTrait, QueryFilter, Set,
};
use std::collections::HashMap;
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
        self.increment_file_citation_txn(&self.db, file_id).await
    }

    /// 增加文件引用计数（事务版本）
    pub async fn increment_file_citation_txn<C: ConnectionTrait>(
        &self,
        conn: &C,
        file_id: i64,
    ) -> Result<bool> {
        use sea_orm::sea_query::Expr;

        let result = Files::update_many()
            .col_expr(
                Column::CitationCount,
                Expr::col(Column::CitationCount).add(1),
            )
            .filter(Column::Id.eq(file_id))
            .exec(conn)
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

    /// 删除文件
    /// - 只有上传者可以删除自己的文件
    /// - 如果文件有引用（citation_count > 0），只删除数据库记录
    /// - 如果没有引用，同时删除物理文件
    pub async fn delete_file_impl(&self, token: &str, user_id: i64) -> Result<bool> {
        // 先查询文件
        let file = self.get_file_by_token_impl(token).await?;

        let file = match file {
            Some(f) => f,
            None => return Ok(false), // 文件不存在
        };

        // 检查权限：只有上传者可以删除
        if file.user_id != Some(user_id) {
            return Err(HWSystemError::authorization("只有上传者可以删除文件"));
        }

        // 删除数据库记录
        let result = Files::delete_by_id(file.id)
            .exec(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("删除文件记录失败: {e}")))?;

        if result.rows_affected == 0 {
            return Ok(false);
        }

        // 如果没有引用，删除物理文件
        if file.citation_count == 0 {
            let config = AppConfig::get();
            let file_path = format!("{}/{}", config.upload.dir, file.stored_name);
            if std::path::Path::new(&file_path).exists()
                && let Err(e) = std::fs::remove_file(&file_path)
            {
                // 物理文件删除失败只记录日志，不影响返回结果
                tracing::warn!("删除物理文件失败: {} - {}", file_path, e);
            }
        }

        Ok(true)
    }

    /// 批量获取文件信息
    pub async fn get_files_by_ids_impl(&self, ids: &[i64]) -> Result<HashMap<i64, File>> {
        if ids.is_empty() {
            return Ok(HashMap::new());
        }

        let files = Files::find()
            .filter(Column::Id.is_in(ids.to_vec()))
            .all(&self.db)
            .await
            .map_err(|e| HWSystemError::database_operation(format!("批量查询文件失败: {e}")))?;

        Ok(files.into_iter().map(|f| (f.id, f.into_file())).collect())
    }
}
