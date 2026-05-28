use chrono::Utc;
use domain::{
    errors::DomainError,
    events::DomainEvent,
    models::{
        feed::{PageParams, Paginated},
        social::{Block, Boost, Follow, FollowState, Like},
        user::User,
    },
    ports::{
        BlockRepository, BoostRepository, EventPublisher, FederationFollowPort, FollowRepository,
        LikeRepository, UserReader,
    },
    value_objects::{BoostId, LikeId, ThoughtId, UserId, Username},
};

pub async fn like_thought(
    likes: &dyn LikeRepository,
    events: &dyn EventPublisher,
    user_id: &UserId,
    thought_id: &ThoughtId,
) -> Result<(), DomainError> {
    let like = Like {
        id: LikeId::new(),
        user_id: user_id.clone(),
        thought_id: thought_id.clone(),
        ap_id: None,
        created_at: Utc::now(),
    };
    likes.save(&like).await?;
    events
        .publish(&DomainEvent::LikeAdded {
            like_id: like.id,
            user_id: user_id.clone(),
            thought_id: thought_id.clone(),
        })
        .await?;
    Ok(())
}

pub async fn unlike_thought(
    likes: &dyn LikeRepository,
    events: &dyn EventPublisher,
    user_id: &UserId,
    thought_id: &ThoughtId,
) -> Result<(), DomainError> {
    likes.delete(user_id, thought_id).await?;
    events
        .publish(&DomainEvent::LikeRemoved {
            user_id: user_id.clone(),
            thought_id: thought_id.clone(),
        })
        .await?;
    Ok(())
}

pub async fn boost_thought(
    boosts: &dyn BoostRepository,
    events: &dyn EventPublisher,
    user_id: &UserId,
    thought_id: &ThoughtId,
) -> Result<(), DomainError> {
    let boost = Boost {
        id: BoostId::new(),
        user_id: user_id.clone(),
        thought_id: thought_id.clone(),
        ap_id: None,
        created_at: Utc::now(),
    };
    boosts.save(&boost).await?;
    events
        .publish(&DomainEvent::BoostAdded {
            boost_id: boost.id,
            user_id: user_id.clone(),
            thought_id: thought_id.clone(),
        })
        .await?;
    Ok(())
}

pub async fn unboost_thought(
    boosts: &dyn BoostRepository,
    events: &dyn EventPublisher,
    user_id: &UserId,
    thought_id: &ThoughtId,
) -> Result<(), DomainError> {
    boosts.delete(user_id, thought_id).await?;
    events
        .publish(&DomainEvent::BoostRemoved {
            user_id: user_id.clone(),
            thought_id: thought_id.clone(),
        })
        .await?;
    Ok(())
}

pub async fn follow_actor(
    follows: &dyn FollowRepository,
    users: &dyn UserReader,
    federation: &dyn FederationFollowPort,
    events: &dyn EventPublisher,
    follower_id: &UserId,
    username: &str,
) -> Result<(), DomainError> {
    if username.contains('@') {
        federation.follow_remote(follower_id, username).await
    } else {
        let uname = Username::new(username)
            .map_err(|_| DomainError::InvalidInput("invalid username".into()))?;
        let target = users
            .find_by_username(&uname)
            .await?
            .ok_or(DomainError::NotFound)?;
        follow_user(follows, events, follower_id, &target.id).await
    }
}

pub async fn follow_user(
    follows: &dyn FollowRepository,
    events: &dyn EventPublisher,
    follower_id: &UserId,
    following_id: &UserId,
) -> Result<(), DomainError> {
    if follower_id == following_id {
        return Err(DomainError::InvalidInput("cannot follow yourself".into()));
    }
    let follow = Follow {
        follower_id: follower_id.clone(),
        following_id: following_id.clone(),
        state: FollowState::Accepted,
        ap_id: None,
        created_at: Utc::now(),
    };
    follows.save(&follow).await?;
    events
        .publish(&DomainEvent::FollowAccepted {
            follower_id: follower_id.clone(),
            following_id: following_id.clone(),
        })
        .await?;
    Ok(())
}

pub async fn unfollow_actor(
    follows: &dyn FollowRepository,
    users: &dyn UserReader,
    federation: &dyn FederationFollowPort,
    events: &dyn EventPublisher,
    follower_id: &UserId,
    username: &str,
) -> Result<(), DomainError> {
    if username.contains('@') {
        federation.unfollow_remote(follower_id, username).await
    } else {
        let uname = Username::new(username)
            .map_err(|_| DomainError::InvalidInput("invalid username".into()))?;
        let target = users
            .find_by_username(&uname)
            .await?
            .ok_or(DomainError::NotFound)?;
        unfollow_user(follows, events, follower_id, &target.id).await
    }
}

pub async fn unfollow_user(
    follows: &dyn FollowRepository,
    events: &dyn EventPublisher,
    follower_id: &UserId,
    following_id: &UserId,
) -> Result<(), DomainError> {
    follows.delete(follower_id, following_id).await?;
    events
        .publish(&DomainEvent::Unfollowed {
            follower_id: follower_id.clone(),
            following_id: following_id.clone(),
        })
        .await?;
    Ok(())
}

pub async fn accept_follow(
    follows: &dyn FollowRepository,
    events: &dyn EventPublisher,
    follower_id: &UserId,
    following_id: &UserId,
) -> Result<(), DomainError> {
    follows
        .update_state(follower_id, following_id, &FollowState::Accepted)
        .await?;
    events
        .publish(&DomainEvent::FollowAccepted {
            follower_id: follower_id.clone(),
            following_id: following_id.clone(),
        })
        .await?;
    Ok(())
}

pub async fn reject_follow(
    follows: &dyn FollowRepository,
    events: &dyn EventPublisher,
    follower_id: &UserId,
    following_id: &UserId,
) -> Result<(), DomainError> {
    follows
        .update_state(follower_id, following_id, &FollowState::Rejected)
        .await?;
    events
        .publish(&DomainEvent::FollowRejected {
            follower_id: follower_id.clone(),
            following_id: following_id.clone(),
        })
        .await?;
    Ok(())
}

pub async fn block_by_username(
    blocks: &dyn BlockRepository,
    users: &dyn UserReader,
    events: &dyn EventPublisher,
    blocker_id: &UserId,
    username: &str,
) -> Result<(), DomainError> {
    let uname = Username::new(username).map_err(|_| DomainError::NotFound)?;
    let target = users
        .find_by_username(&uname)
        .await?
        .ok_or(DomainError::NotFound)?;
    block_user(blocks, events, blocker_id, &target.id).await
}

pub async fn unblock_by_username(
    blocks: &dyn BlockRepository,
    users: &dyn UserReader,
    events: &dyn EventPublisher,
    blocker_id: &UserId,
    username: &str,
) -> Result<(), DomainError> {
    let uname = Username::new(username).map_err(|_| DomainError::NotFound)?;
    let target = users
        .find_by_username(&uname)
        .await?
        .ok_or(DomainError::NotFound)?;
    unblock_user(blocks, events, blocker_id, &target.id).await
}

pub async fn block_user(
    blocks: &dyn BlockRepository,
    events: &dyn EventPublisher,
    blocker_id: &UserId,
    blocked_id: &UserId,
) -> Result<(), DomainError> {
    if blocker_id == blocked_id {
        return Err(DomainError::InvalidInput("cannot block yourself".into()));
    }
    let block = Block {
        blocker_id: blocker_id.clone(),
        blocked_id: blocked_id.clone(),
        created_at: Utc::now(),
    };
    blocks.save(&block).await?;
    events
        .publish(&DomainEvent::UserBlocked {
            blocker_id: blocker_id.clone(),
            blocked_id: blocked_id.clone(),
        })
        .await?;
    Ok(())
}

pub async fn unblock_user(
    blocks: &dyn BlockRepository,
    events: &dyn EventPublisher,
    blocker_id: &UserId,
    blocked_id: &UserId,
) -> Result<(), DomainError> {
    blocks.delete(blocker_id, blocked_id).await?;
    events
        .publish(&DomainEvent::UserUnblocked {
            blocker_id: blocker_id.clone(),
            blocked_id: blocked_id.clone(),
        })
        .await?;
    Ok(())
}

pub async fn get_local_friends(
    follows: &dyn FollowRepository,
    user_id: &UserId,
    page: &PageParams,
) -> Result<Paginated<User>, DomainError> {
    follows.list_mutual(user_id, page).await
}

#[cfg(test)]
mod tests;
