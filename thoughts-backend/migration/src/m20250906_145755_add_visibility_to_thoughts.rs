use super::m20250905_000001_init::Thought;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TYPE thought_visibility AS ENUM ('public', 'friends_only', 'private')",
            )
            .await?;

        // 2. Add the new column to the thoughts table
        manager
            .alter_table(
                Table::alter()
                    .table(Thought::Table)
                    .add_column(
                        ColumnDef::new(ThoughtExtension::Visibility)
                            .enumeration(
                                "thought_visibility",
                                ["public", "friends_only", "private"],
                            )
                            .not_null()
                            .default("public"), // Default new thoughts to public
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Thought::Table)
                    .drop_column(ThoughtExtension::Visibility)
                    .to_owned(),
            )
            .await?;

        // Drop the ENUM type
        manager
            .get_connection()
            .execute_unprepared("DROP TYPE thought_visibility")
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum ThoughtExtension {
    Visibility,
}
