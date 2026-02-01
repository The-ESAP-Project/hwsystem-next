pub use sea_orm_migration::prelude::*;

mod m20250123_000001_create_tables;
mod m20250126_000001_create_system_settings;
mod m20260201_000001_add_image_compression_settings;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250123_000001_create_tables::Migration),
            Box::new(m20250126_000001_create_system_settings::Migration),
            Box::new(m20260201_000001_add_image_compression_settings::Migration),
        ]
    }
}
