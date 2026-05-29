use super::*;
use domain::{
    models::{
        notification::NotificationKind,
        thought::{NewThought, Thought, Visibility},
        user::User,
    },
    testing::TestStore,
    value_objects::*,
};
use std::sync::Arc;

fn alice() -> User {
    User::new_local(
        UserId::new(),
        Username::new("alice").unwrap(),
        Email::new("alice@ex.com").unwrap(),
        PasswordHash("h".into()),
    )
}

#[tokio::test]
async fn like_creates_notification_for_thought_author() {
    let store = TestStore::default();
    let alice = alice();
    let bob_id = UserId::new();
    let thought = Thought::new_local(NewThought {
        id: ThoughtId::new(),
        user_id: alice.id.clone(),
        content: Content::new_local("hello").unwrap(),
        in_reply_to_id: None,
        visibility: Visibility::Public,
        content_warning: None,
        sensitive: false,
        mood: None,
    });
    store.thoughts.lock().unwrap().push(thought.clone());
    let svc = NotificationEventService {
        thoughts: Arc::new(store.clone()),
        notifications: Arc::new(store.clone()),
    };
    svc.process(&DomainEvent::LikeAdded {
        like_id: LikeId::new(),
        user_id: bob_id,
        thought_id: thought.id.clone(),
    })
    .await
    .unwrap();
    let notifs = store.notifications.lock().unwrap();
    assert_eq!(notifs.len(), 1);
    assert!(matches!(notifs[0].kind, NotificationKind::Like { .. }));
}

#[tokio::test]
async fn self_like_creates_no_notification() {
    let store = TestStore::default();
    let alice = alice();
    let thought = Thought::new_local(NewThought {
        id: ThoughtId::new(),
        user_id: alice.id.clone(),
        content: Content::new_local("hello").unwrap(),
        in_reply_to_id: None,
        visibility: Visibility::Public,
        content_warning: None,
        sensitive: false,
        mood: None,
    });
    store.thoughts.lock().unwrap().push(thought.clone());
    let svc = NotificationEventService {
        thoughts: Arc::new(store.clone()),
        notifications: Arc::new(store.clone()),
    };
    svc.process(&DomainEvent::LikeAdded {
        like_id: LikeId::new(),
        user_id: alice.id.clone(),
        thought_id: thought.id.clone(),
    })
    .await
    .unwrap();
    assert!(store.notifications.lock().unwrap().is_empty());
}

#[tokio::test]
async fn follow_accepted_creates_notification() {
    let store = TestStore::default();
    let alice = alice();
    let bob_id = UserId::new();
    let svc = NotificationEventService {
        thoughts: Arc::new(store.clone()),
        notifications: Arc::new(store.clone()),
    };
    svc.process(&DomainEvent::FollowAccepted {
        follower_id: bob_id,
        following_id: alice.id.clone(),
    })
    .await
    .unwrap();
    let notifs = store.notifications.lock().unwrap();
    assert_eq!(notifs.len(), 1);
    assert!(matches!(notifs[0].kind, NotificationKind::Follow { .. }));
}

#[tokio::test]
async fn reply_creates_notification_for_original_author() {
    let store = TestStore::default();
    let alice = alice();
    let bob_id = UserId::new();
    let original = Thought::new_local(NewThought {
        id: ThoughtId::new(),
        user_id: alice.id.clone(),
        content: Content::new_local("original").unwrap(),
        in_reply_to_id: None,
        visibility: Visibility::Public,
        content_warning: None,
        sensitive: false,
        mood: None,
    });
    store.thoughts.lock().unwrap().push(original.clone());
    let svc = NotificationEventService {
        thoughts: Arc::new(store.clone()),
        notifications: Arc::new(store.clone()),
    };
    svc.process(&DomainEvent::ThoughtCreated {
        thought_id: ThoughtId::new(),
        user_id: bob_id,
        in_reply_to_id: Some(original.id.clone()),
    })
    .await
    .unwrap();
    let notifs = store.notifications.lock().unwrap();
    assert_eq!(notifs.len(), 1);
    assert!(matches!(notifs[0].kind, NotificationKind::Reply { .. }));
}

#[tokio::test]
async fn self_reply_creates_no_notification() {
    let store = TestStore::default();
    let alice = alice();
    let original = Thought::new_local(NewThought {
        id: ThoughtId::new(),
        user_id: alice.id.clone(),
        content: Content::new_local("original").unwrap(),
        in_reply_to_id: None,
        visibility: Visibility::Public,
        content_warning: None,
        sensitive: false,
        mood: None,
    });
    store.thoughts.lock().unwrap().push(original.clone());
    let svc = NotificationEventService {
        thoughts: Arc::new(store.clone()),
        notifications: Arc::new(store.clone()),
    };
    svc.process(&DomainEvent::ThoughtCreated {
        thought_id: ThoughtId::new(),
        user_id: alice.id.clone(),
        in_reply_to_id: Some(original.id.clone()),
    })
    .await
    .unwrap();
    assert!(store.notifications.lock().unwrap().is_empty());
}

#[tokio::test]
async fn self_boost_creates_no_notification() {
    let store = TestStore::default();
    let alice = alice();
    let thought = Thought::new_local(NewThought {
        id: ThoughtId::new(),
        user_id: alice.id.clone(),
        content: Content::new_local("hello").unwrap(),
        in_reply_to_id: None,
        visibility: Visibility::Public,
        content_warning: None,
        sensitive: false,
        mood: None,
    });
    store.thoughts.lock().unwrap().push(thought.clone());
    let svc = NotificationEventService {
        thoughts: Arc::new(store.clone()),
        notifications: Arc::new(store.clone()),
    };
    svc.process(&DomainEvent::BoostAdded {
        boost_id: BoostId::new(),
        user_id: alice.id.clone(),
        thought_id: thought.id.clone(),
    })
    .await
    .unwrap();
    assert!(store.notifications.lock().unwrap().is_empty());
}
