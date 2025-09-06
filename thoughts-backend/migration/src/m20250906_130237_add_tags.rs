use super::m20250905_000001_init::Thought;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Tag::Table)
                    .if_not_exists()
                    .col(pk_auto(Tag::Id))
                    .col(string(Tag::Name).not_null().unique_key())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ThoughtTag::Table)
                    .if_not_exists()
                    .col(uuid(ThoughtTag::ThoughtId).not_null())
                    .col(integer(ThoughtTag::TagId).not_null())
                    .primary_key(
                        Index::create()
                            .col(ThoughtTag::ThoughtId)
                            .col(ThoughtTag::TagId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_thought_tag_thought_id")
                            .from(ThoughtTag::Table, ThoughtTag::ThoughtId)
                            .to(Thought::Table, Thought::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_thought_tag_tag_id")
                            .from(ThoughtTag::Table, ThoughtTag::TagId)
                            .to(Tag::Table, Tag::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ThoughtTag::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Tag::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Tag {
    Table,
    Id,
    Name,
}

#[derive(DeriveIden)]
enum ThoughtTag {
    Table,
    ThoughtId,
    TagId,
}
