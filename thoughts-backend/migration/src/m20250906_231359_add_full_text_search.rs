use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // --- Users Table ---
        // Add the tsvector column for users
        manager.get_connection().execute_unprepared(
            "ALTER TABLE \"user\" ADD COLUMN \"search_document\" tsvector \
            GENERATED ALWAYS AS (to_tsvector('english', username || ' ' || coalesce(display_name, ''))) STORED"
        ).await?;
        // Add the GIN index for users
        manager.get_connection().execute_unprepared(
            "CREATE INDEX \"user_search_document_idx\" ON \"user\" USING GIN(\"search_document\")"
        ).await?;

        // --- Thoughts Table ---
        // Add the tsvector column for thoughts
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE \"thought\" ADD COLUMN \"search_document\" tsvector \
            GENERATED ALWAYS AS (to_tsvector('english', content)) STORED",
            )
            .await?;
        // Add the GIN index for thoughts
        manager.get_connection().execute_unprepared(
            "CREATE INDEX \"thought_search_document_idx\" ON \"thought\" USING GIN(\"search_document\")"
        ).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("ALTER TABLE \"user\" DROP COLUMN \"search_document\"")
            .await?;
        manager
            .get_connection()
            .execute_unprepared("ALTER TABLE \"thought\" DROP COLUMN \"search_document\"")
            .await?;
        Ok(())
    }
}
