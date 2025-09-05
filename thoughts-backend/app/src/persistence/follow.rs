use sea_orm::{ActiveModelTrait, ColumnTrait, DbConn, DbErr, EntityTrait, QueryFilter, Set};

use crate::error::UserError;
use models::domains::follow;

pub async fn follow_user(db: &DbConn, follower_id: i32, followee_id: i32) -> Result<(), DbErr> {
    if follower_id == followee_id {
        return Err(DbErr::Custom("Users cannot follow themselves".to_string()));
    }

    let follow = follow::ActiveModel {
        follower_id: Set(follower_id),
        followed_id: Set(followee_id),
    };

    follow.save(db).await?;
    Ok(())
}

pub async fn unfollow_user(
    db: &DbConn,
    follower_id: i32,
    followee_id: i32,
) -> Result<(), UserError> {
    let deleted_result = follow::Entity::delete_many()
        .filter(follow::Column::FollowerId.eq(follower_id))
        .filter(follow::Column::FollowedId.eq(followee_id))
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
