use sea_orm::prelude::Uuid;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbConn, DbErr, EntityTrait, JoinType, QueryFilter, QueryOrder,
    QuerySelect, RelationTrait, Set, TransactionTrait,
};

use models::domains::{top_friends, user};
use models::params::user::{CreateUserParams, UpdateUserParams};
use models::queries::user::UserQuery;

use crate::error::UserError;
use crate::persistence::follow::{get_follower_ids, get_following_ids, get_friend_ids};

pub async fn create_user(
    db: &DbConn,
    params: CreateUserParams,
) -> Result<user::ActiveModel, DbErr> {
    user::ActiveModel {
        username: Set(params.username),
        ..Default::default()
    }
    .save(db)
    .await
}

pub async fn search_users(db: &DbConn, query: UserQuery) -> Result<Vec<user::Model>, DbErr> {
    user::Entity::find()
        .filter(user::Column::Username.contains(query.username.unwrap_or_default()))
        .all(db)
        .await
}

pub async fn get_user(db: &DbConn, id: Uuid) -> Result<Option<user::Model>, DbErr> {
    user::Entity::find_by_id(id).one(db).await
}

pub async fn get_user_by_username(
    db: &DbConn,
    username: &str,
) -> Result<Option<user::Model>, DbErr> {
    user::Entity::find()
        .filter(user::Column::Username.eq(username))
        .one(db)
        .await
}

pub async fn get_users_by_ids(db: &DbConn, ids: Vec<Uuid>) -> Result<Vec<user::Model>, DbErr> {
    user::Entity::find()
        .filter(user::Column::Id.is_in(ids))
        .all(db)
        .await
}

pub async fn update_user_profile(
    db: &DbConn,
    user_id: Uuid,
    params: UpdateUserParams,
) -> Result<user::Model, UserError> {
    let mut user: user::ActiveModel = get_user(db, user_id)
        .await
        .map_err(|e| UserError::Internal(e.to_string()))?
        .ok_or(UserError::NotFound)?
        .into();

    if let Some(display_name) = params.display_name {
        user.display_name = Set(Some(display_name));
    }
    if let Some(bio) = params.bio {
        user.bio = Set(Some(bio));
    }
    if let Some(avatar_url) = params.avatar_url {
        user.avatar_url = Set(Some(avatar_url));
    }
    if let Some(header_url) = params.header_url {
        user.header_url = Set(Some(header_url));
    }
    if let Some(custom_css) = params.custom_css {
        user.custom_css = Set(Some(custom_css));
    }

    if let Some(friend_usernames) = params.top_friends {
        let txn = db
            .begin()
            .await
            .map_err(|e| UserError::Internal(e.to_string()))?;

        top_friends::Entity::delete_many()
            .filter(top_friends::Column::UserId.eq(user_id))
            .exec(&txn)
            .await
            .map_err(|e| UserError::Internal(e.to_string()))?;

        let friends = user::Entity::find()
            .filter(user::Column::Username.is_in(friend_usernames.clone()))
            .all(&txn)
            .await
            .map_err(|e| UserError::Internal(e.to_string()))?;

        if friends.len() != friend_usernames.len() {
            return Err(UserError::Validation(
                "One or more usernames in top_friends do not exist".to_string(),
            ));
        }

        let new_top_friends: Vec<top_friends::ActiveModel> = friends
            .iter()
            .enumerate()
            .map(|(index, friend)| top_friends::ActiveModel {
                user_id: Set(user_id),
                friend_id: Set(friend.id),
                position: Set((index + 1) as i16),
                ..Default::default()
            })
            .collect();

        if !new_top_friends.is_empty() {
            top_friends::Entity::insert_many(new_top_friends)
                .exec(&txn)
                .await
                .map_err(|e| UserError::Internal(e.to_string()))?;
        }

        txn.commit()
            .await
            .map_err(|e| UserError::Internal(e.to_string()))?;
    }

    user.update(db)
        .await
        .map_err(|e| UserError::Internal(e.to_string()))
}

pub async fn get_top_friends(db: &DbConn, user_id: Uuid) -> Result<Vec<user::Model>, DbErr> {
    user::Entity::find()
        .join(
            JoinType::InnerJoin,
            top_friends::Relation::Friend.def().rev(),
        )
        .filter(top_friends::Column::UserId.eq(user_id))
        .order_by_asc(top_friends::Column::Position)
        .all(db)
        .await
}

pub async fn get_friends(db: &DbConn, user_id: Uuid) -> Result<Vec<user::Model>, DbErr> {
    let friend_ids = get_friend_ids(db, user_id).await?;
    if friend_ids.is_empty() {
        return Ok(vec![]);
    }
    get_users_by_ids(db, friend_ids).await
}

pub async fn get_following(db: &DbConn, user_id: Uuid) -> Result<Vec<user::Model>, DbErr> {
    let following_ids = get_following_ids(db, user_id).await?;
    if following_ids.is_empty() {
        return Ok(vec![]);
    }
    get_users_by_ids(db, following_ids).await
}

pub async fn get_followers(db: &DbConn, user_id: Uuid) -> Result<Vec<user::Model>, DbErr> {
    let follower_ids = get_follower_ids(db, user_id).await?;
    if follower_ids.is_empty() {
        return Ok(vec![]);
    }
    get_users_by_ids(db, follower_ids).await
}

pub async fn get_all_users(db: &DbConn) -> Result<Vec<user::Model>, DbErr> {
    user::Entity::find()
        .order_by_desc(user::Column::CreatedAt)
        .all(db)
        .await
}
