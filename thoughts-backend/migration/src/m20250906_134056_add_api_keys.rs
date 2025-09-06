use super::m20240101_000001_init::User;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ApiKey::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ApiKey::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(uuid(ApiKey::UserId).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_api_key_user_id")
                            .from(ApiKey::Table, ApiKey::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(text(ApiKey::KeyHash).not_null().unique_key())
                    .col(string(ApiKey::Name).not_null())
                    .col(
                        timestamp_with_time_zone(ApiKey::CreatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(ApiKey::KeyPrefix).string_len(8).not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-api_keys-key_prefix")
                    .table(ApiKey::Table)
                    .col(ApiKey::KeyPrefix)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ApiKey::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ApiKey {
    Table,
    Id,
    UserId,
    KeyHash,
    Name,
    CreatedAt,
    KeyPrefix,
}
