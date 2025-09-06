use super::m20240101_000001_init::User;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // --- Create Thought Table ---
        manager
            .create_table(
                Table::create()
                    .table(Thought::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Thought::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(uuid(Thought::AuthorId).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_thought_author_id")
                            .from(Thought::Table, Thought::AuthorId)
                            .to(User::Table, User::Id)
                            .on_update(ForeignKeyAction::NoAction)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(string(Thought::Content).not_null())
                    .col(
                        timestamp_with_time_zone(Thought::CreatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // --- Create Follow Table ---
        manager
            .create_table(
                Table::create()
                    .table(Follow::Table)
                    .if_not_exists()
                    .col(uuid(Follow::FollowerId).not_null())
                    .col(uuid(Follow::FollowedId).not_null())
                    // Composite Primary Key to ensure a user can only follow another once
                    .primary_key(
                        Index::create()
                            .col(Follow::FollowerId)
                            .col(Follow::FollowedId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_follow_follower_id")
                            .from(Follow::Table, Follow::FollowerId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_follow_followed_id")
                            .from(Follow::Table, Follow::FollowedId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Follow::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Thought::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Thought {
    Table,
    Id,
    AuthorId,
    Content,
    CreatedAt,
}

#[derive(DeriveIden)]
pub enum Follow {
    Table,
    // The user who is initiating the follow
    FollowerId,
    // The user who is being followed
    FollowedId,
}
