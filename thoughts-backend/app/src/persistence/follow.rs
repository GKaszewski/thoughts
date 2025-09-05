use sea_orm::{ActiveModelTrait, ColumnTrait, DbConn, DbErr, EntityTrait, QueryFilter, Set};

use crate::{error::UserError, persistence::user::get_user_by_username};
use models::domains::follow;

pub async fn add_follower(
    db: &DbConn,
    followed_id: i32,
    follower_actor_id: &str,
) -> Result<(), UserError> {
    let follower_username = follower_actor_id
        .split('/')
        .last()
        .ok_or_else(|| UserError::Internal("Invalid follower actor ID".to_string()))?;

    let follower = get_user_by_username(db, follower_username)
        .await
        .map_err(|e| UserError::Internal(e.to_string()))?
        .ok_or(UserError::NotFound)?;

    follow_user(db, follower.id, followed_id)
        .await
        .map_err(|e| UserError::Internal(e.to_string()))?;

    Ok(())
}

pub async fn follow_user(db: &DbConn, follower_id: i32, followed_id: i32) -> Result<(), DbErr> {
    if follower_id == followed_id {
        return Err(DbErr::Custom("Users cannot follow themselves".to_string()));
    }

    let follow = follow::ActiveModel {
        follower_id: Set(follower_id),
        followed_id: Set(followed_id),
    };

    follow.insert(db).await?;
    Ok(())
}

pub async fn unfollow_user(
    db: &DbConn,
    follower_id: i32,
    followed_id: i32,
) -> Result<(), UserError> {
    let deleted_result = follow::Entity::delete_many()
        .filter(follow::Column::FollowerId.eq(follower_id))
        .filter(follow::Column::FollowedId.eq(followed_id))
        .exec(db)
        .await
        .map_err(|e| UserError::Internal(e.to_string()))?;

    if deleted_result.rows_affected == 0 {
        return Err(UserError::NotFollowing);
    }

    Ok(())
}

pub async fn get_followed_ids(db: &DbConn, user_id: i32) -> Result<Vec<i32>, DbErr> {
    let followed_users = follow::Entity::find()
        .filter(follow::Column::FollowerId.eq(user_id))
        .all(db)
        .await?;

    Ok(followed_users.into_iter().map(|f| f.followed_id).collect())
}
