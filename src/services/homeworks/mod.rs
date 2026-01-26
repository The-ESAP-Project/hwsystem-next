pub mod create;
pub mod delete;
pub mod detail;
pub mod list;
pub mod list_all;
pub mod my_stats;
pub mod stats;
pub mod stats_export;
pub mod teacher_stats;
pub mod update;

use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};
use std::sync::Arc;

use crate::models::homeworks::requests::{
    AllHomeworksQuery, CreateHomeworkRequest, HomeworkListQuery, UpdateHomeworkRequest,
};
use crate::storage::Storage;

pub struct HomeworkService {
    storage: Option<Arc<dyn Storage>>,
}

impl HomeworkService {
    pub fn new_lazy() -> Self {
        Self { storage: None }
    }

    pub(crate) fn get_storage(&self, request: &HttpRequest) -> Arc<dyn Storage> {
        if let Some(storage) = &self.storage {
            storage.clone()
        } else {
            request
                .app_data::<actix_web::web::Data<Arc<dyn Storage>>>()
                .expect("Storage not found in app data")
                .get_ref()
                .clone()
        }
    }

    pub async fn list_homeworks(
        &self,
        request: &HttpRequest,
        query: HomeworkListQuery,
    ) -> ActixResult<HttpResponse> {
        list::list_homeworks(self, request, query).await
    }

    pub async fn create_homework(
        &self,
        request: &HttpRequest,
        created_by: i64,
        req: CreateHomeworkRequest,
    ) -> ActixResult<HttpResponse> {
        create::create_homework(self, request, created_by, req).await
    }

    pub async fn get_homework(
        &self,
        request: &HttpRequest,
        homework_id: i64,
    ) -> ActixResult<HttpResponse> {
        detail::get_homework(self, request, homework_id).await
    }

    pub async fn update_homework(
        &self,
        request: &HttpRequest,
        homework_id: i64,
        req: UpdateHomeworkRequest,
        user_id: i64,
    ) -> ActixResult<HttpResponse> {
        update::update_homework(self, request, homework_id, req, user_id).await
    }

    pub async fn delete_homework(
        &self,
        request: &HttpRequest,
        homework_id: i64,
        user_id: i64,
    ) -> ActixResult<HttpResponse> {
        delete::delete_homework(self, request, homework_id, user_id).await
    }

    pub async fn get_homework_stats(
        &self,
        request: &HttpRequest,
        homework_id: i64,
    ) -> ActixResult<HttpResponse> {
        stats::get_homework_stats(self, request, homework_id).await
    }

    pub async fn export_homework_stats(
        &self,
        request: &HttpRequest,
        homework_id: i64,
    ) -> ActixResult<HttpResponse> {
        stats_export::export_homework_stats(self, request, homework_id).await
    }

    pub async fn get_my_homework_stats(&self, request: &HttpRequest) -> ActixResult<HttpResponse> {
        my_stats::get_my_homework_stats(self, request).await
    }

    pub async fn get_teacher_homework_stats(
        &self,
        request: &HttpRequest,
    ) -> ActixResult<HttpResponse> {
        teacher_stats::get_teacher_homework_stats(self, request).await
    }

    pub async fn list_all_homeworks(
        &self,
        request: &HttpRequest,
        query: AllHomeworksQuery,
    ) -> ActixResult<HttpResponse> {
        list_all::list_all_homeworks(self, request, query).await
    }
}
