use super::*;
use domain::{
    models::{
        thought::{NewThought, Thought, Visibility},
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
    store
        .thoughts
        .lock()
        .unwrap()
        .push(Thought::new_local(NewThought {
            id: tid.clone(),
            user_id: alice.id.clone(),
            content: Content::new_local("hi").unwrap(),
            in_reply_to_id: None,
            visibility: Visibility::Public,
            content_warning: None,
            sensitive: false,
            mood: None,
        }));
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

#[tokio::test]
async fn get_local_friends_returns_mutual_follows() {
    use domain::models::feed::PageParams;
    let store = TestStore::default();
    let alice = user("alice");
    let bob = user("bob");
    let carol = user("carol");

    store
        .users
        .lock()
        .unwrap()
        .extend([alice.clone(), bob.clone(), carol.clone()]);

    // alice ↔ bob = friends; alice → carol but not back
    store.follows.lock().unwrap().extend([
        domain::models::social::Follow {
            follower_id: alice.id.clone(),
            following_id: bob.id.clone(),
            state: domain::models::social::FollowState::Accepted,
            ap_id: None,
            created_at: chrono::Utc::now(),
        },
        domain::models::social::Follow {
            follower_id: bob.id.clone(),
            following_id: alice.id.clone(),
            state: domain::models::social::FollowState::Accepted,
            ap_id: None,
            created_at: chrono::Utc::now(),
        },
        domain::models::social::Follow {
            follower_id: alice.id.clone(),
            following_id: carol.id.clone(),
            state: domain::models::social::FollowState::Accepted,
            ap_id: None,
            created_at: chrono::Utc::now(),
        },
    ]);

    let page = PageParams {
        page: 1,
        per_page: 20,
    };
    let result = get_local_friends(&store, &alice.id, &page).await.unwrap();
    assert_eq!(result.total, 1);
    assert_eq!(result.items[0].id, bob.id);
}
