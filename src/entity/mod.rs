//! SeaORM 实体定义
//!
//! 这些实体用于数据库操作，与 models 模块中的业务实体分离。
//! Storage 层使用这些实体进行 CRUD 操作，然后转换为 models 中的业务实体。

pub mod prelude;

pub mod class_users;
pub mod classes;
pub mod files;
pub mod grades;
pub mod homework_files;
pub mod homeworks;
pub mod notifications;
pub mod submission_files;
pub mod submissions;
pub mod users;
