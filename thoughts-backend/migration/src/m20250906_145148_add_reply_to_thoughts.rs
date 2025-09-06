use sea_orm_migration::{prelude::*, schema::*};

use crate::m20250905_000001_init::Thought;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Thought::Table)
                    .add_column(uuid_null(ThoughtExtension::ReplyToId))
                    .add_foreign_key(
                        TableForeignKey::new()
                            .name("fk_thought_reply_to_id")
                            .from_tbl(Thought::Table)
                            .from_col(ThoughtExtension::ReplyToId)
                            .to_tbl(Thought::Table)
                            .to_col(Thought::Id)
                            .on_delete(ForeignKeyAction::SetNull),
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
                    .drop_foreign_key(Alias::new("fk_thought_reply_to_id"))
                    .drop_column(ThoughtExtension::ReplyToId)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum ThoughtExtension {
    ReplyToId,
}
