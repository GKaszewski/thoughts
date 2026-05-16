use chrono::Utc;
use domain::{
    errors::DomainError,
    events::DomainEvent,
    models::social::{Block, Boost, Follow, FollowState, Like},
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

#[cfg(test)]
mod tests {
    use super::*;
    use domain::{
        models::{
            thought::{Thought, Visibility},
            user::User,
        },
        testing::TestStore,
        value_objects::*,
    };

    fn user(name: &str) -> User {
        User::new_local(
            UserId::new(),
            Username::new(name).unwrap(),
            Email::new(format!("{name}@ex.com")).unwrap(),
            PasswordHash("h".into()),
        )
    }

    #[tokio::test]
    async fn like_and_unlike() {
        let store = TestStore::default();
        let alice = user("alice");
        let tid = ThoughtId::new();
        store.thoughts.lock().unwrap().push(Thought::new_local(
            tid.clone(),
            alice.id.clone(),
            Content::new_local("hi").unwrap(),
            None,
            Visibility::Public,
            None,
            false,
        ));
        like_thought(&store, &store, &alice.id, &tid).await.unwrap();
        assert_eq!(store.likes.lock().unwrap().len(), 1);
        unlike_thought(&store, &store, &alice.id, &tid)
            .await
            .unwrap();
        assert!(store.likes.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn follow_and_unfollow() {
        let store = TestStore::default();
        let alice = user("alice");
        let bob = user("bob");
        follow_user(&store, &store, &alice.id, &bob.id)
            .await
            .unwrap();
        assert_eq!(store.follows.lock().unwrap().len(), 1);
        unfollow_user(&store, &store, &alice.id, &bob.id)
            .await
            .unwrap();
        assert!(store.follows.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn cannot_follow_self() {
        let store = TestStore::default();
        let alice = user("alice");
        let err = follow_user(&store, &store, &alice.id, &alice.id)
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::InvalidInput(_)));
    }

    #[tokio::test]
    async fn unblock_user_publishes_event() {
        let store = TestStore::default();
        let alice = user("alice");
        let bob = user("bob");
        block_user(&store, &store, &alice.id, &bob.id)
            .await
            .unwrap();
        store.events.lock().unwrap().clear();
        unblock_user(&store, &store, &alice.id, &bob.id)
            .await
            .unwrap();
        let events = store.events.lock().unwrap();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], DomainEvent::UserUnblocked { .. }));
    }

    #[tokio::test]
    async fn block_user_saves_block_and_publishes_event() {
        let store = TestStore::default();
        let alice = user("alice");
        let bob = user("bob");
        block_user(&store, &store, &alice.id, &bob.id)
            .await
            .unwrap();
        assert_eq!(store.blocks.lock().unwrap().len(), 1);
        let events = store.events.lock().unwrap();
        assert!(events.iter().any(
            |e| matches!(e, DomainEvent::UserBlocked { blocker_id, .. } if blocker_id == &alice.id)
        ));
    }

    #[tokio::test]
    async fn cannot_block_self() {
        let store = TestStore::default();
        let alice = user("alice");
        let err = block_user(&store, &store, &alice.id, &alice.id)
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::InvalidInput(_)));
    }

    #[tokio::test]
    async fn follow_actor_local_routes_to_follow_user() {
        let store = TestStore::default();
        let alice = user("alice");
        let bob = user("bob");
        store.users.lock().unwrap().push(bob.clone());
        follow_actor(&store, &store, &store, &store, &alice.id, "bob")
            .await
            .unwrap();
        assert_eq!(store.follows.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn follow_actor_remote_routes_to_federation() {
        let store = TestStore::default();
        let alice = user("alice");
        follow_actor(
            &store,
            &store,
            &store,
            &store,
            &alice.id,
            "@bob@example.com",
        )
        .await
        .unwrap();
        // TestStore.follow_remote is a no-op that returns Ok(())
        // no local follow should be recorded
        assert!(store.follows.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn unfollow_actor_local_routes_to_unfollow_user() {
        let store = TestStore::default();
        let alice = user("alice");
        let bob = user("bob");
        store.users.lock().unwrap().push(bob.clone());
        // Create an existing follow first
        store
            .follows
            .lock()
            .unwrap()
            .push(domain::models::social::Follow {
                follower_id: alice.id.clone(),
                following_id: bob.id.clone(),
                state: domain::models::social::FollowState::Accepted,
                ap_id: None,
                created_at: chrono::Utc::now(),
            });
        unfollow_actor(&store, &store, &store, &store, &alice.id, "bob")
            .await
            .unwrap();
        assert!(store.follows.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn unfollow_actor_remote_routes_to_federation() {
        let store = TestStore::default();
        let alice = user("alice");
        unfollow_actor(
            &store,
            &store,
            &store,
            &store,
            &alice.id,
            "@bob@example.com",
        )
        .await
        .unwrap();
        // TestStore.unfollow_remote is a no-op — just verify it doesn't error
        assert!(store.follows.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn boost_and_unboost() {
        let store = TestStore::default();
        let alice = user("alice");
        let tid = ThoughtId::new();
        boost_thought(&store, &store, &alice.id, &tid)
            .await
            .unwrap();
        assert_eq!(store.boosts.lock().unwrap().len(), 1);
        unboost_thought(&store, &store, &alice.id, &tid)
            .await
            .unwrap();
        assert!(store.boosts.lock().unwrap().is_empty());
        let events = store.events.lock().unwrap();
        assert!(events
            .iter()
            .any(|e| matches!(e, DomainEvent::BoostAdded { .. })));
        assert!(events
            .iter()
            .any(|e| matches!(e, DomainEvent::BoostRemoved { .. })));
    }
}
