pub mod delete;
pub mod download;
pub mod upload;

use actix_multipart::Multipart;
use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};
use std::sync::Arc;

use crate::models::{ApiResponse, ErrorCode};
use crate::storage::Storage;

pub struct FileService {
    storage: Option<Arc<dyn Storage>>,
}

impl FileService {
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

    // Handle file upload
    pub async fn handle_upload(
        &self,
        request: &HttpRequest,
        payload: Multipart,
    ) -> ActixResult<HttpResponse> {
        upload::handle_upload(self, request, payload).await
    }

    // Handle file download
    pub async fn handle_download(
        &self,
        request: &HttpRequest,
        file_token: String,
    ) -> ActixResult<HttpResponse> {
        download::handle_download(self, request, file_token).await
    }

    // Handle file delete
    pub async fn handle_delete(
        &self,
        request: &HttpRequest,
        file_token: String,
    ) -> ActixResult<HttpResponse> {
        delete::handle_delete(self, request, file_token).await
    }
}
