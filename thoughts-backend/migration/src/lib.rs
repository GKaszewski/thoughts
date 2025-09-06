pub use sea_orm_migration::prelude::*;

mod m20240101_000001_init;
mod m20250905_000001_init;
mod m20250906_100000_add_profile_fields;
mod m20250906_130237_add_tags;
mod m20250906_134056_add_api_keys;
mod m20250906_145148_add_reply_to_thoughts;
mod m20250906_145755_add_visibility_to_thoughts;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240101_000001_init::Migration),
            Box::new(m20250905_000001_init::Migration),
            Box::new(m20250906_100000_add_profile_fields::Migration),
            Box::new(m20250906_130237_add_tags::Migration),
            Box::new(m20250906_134056_add_api_keys::Migration),
            Box::new(m20250906_145148_add_reply_to_thoughts::Migration),
            Box::new(m20250906_145755_add_visibility_to_thoughts::Migration),
        ]
    }
}
