use super::m20240101_000001_init::User;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(User::Table)
                    .add_column(string_null(UserExtension::Email).unique_key())
                    .add_column(string_null(UserExtension::DisplayName))
                    .add_column(string_null(UserExtension::Bio))
                    .add_column(text_null(UserExtension::AvatarUrl))
                    .add_column(text_null(UserExtension::HeaderUrl))
                    .add_column(text_null(UserExtension::CustomCss))
                    .add_column(
                        timestamp_with_time_zone(UserExtension::CreatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .add_column(
                        timestamp_with_time_zone(UserExtension::UpdatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(TopFriends::Table)
                    .if_not_exists()
                    .col(uuid(TopFriends::UserId).not_null())
                    .col(uuid(TopFriends::FriendId).not_null())
                    .col(small_integer(TopFriends::Position).not_null())
                    .primary_key(
                        Index::create()
                            .col(TopFriends::UserId)
                            .col(TopFriends::FriendId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_top_friends_user_id")
                            .from(TopFriends::Table, TopFriends::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_top_friends_friend_id")
                            .from(TopFriends::Table, TopFriends::FriendId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TopFriends::Table).to_owned())
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(User::Table)
                    .drop_column(UserExtension::Email)
                    .drop_column(UserExtension::DisplayName)
                    .drop_column(UserExtension::Bio)
                    .drop_column(UserExtension::AvatarUrl)
                    .drop_column(UserExtension::HeaderUrl)
                    .drop_column(UserExtension::CustomCss)
                    .drop_column(UserExtension::CreatedAt)
                    .drop_column(UserExtension::UpdatedAt)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum UserExtension {
    Email,
    DisplayName,
    Bio,
    AvatarUrl,
    HeaderUrl,
    CustomCss,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum TopFriends {
    Table,
    UserId,
    FriendId,
    Position,
}
