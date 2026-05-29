use super::*;
use domain::{
    models::user::User,
    testing::{NoOpEventPublisher, NoOpOutboxWriter, TestOutbox, TestStore},
    value_objects::*,
};

fn user() -> User {
    User::new_local(
        UserId::new(),
        Username::new("alice").unwrap(),
        Email::new("alice@ex.com").unwrap(),
        PasswordHash("h".into()),
    )
}

fn input(uid: UserId) -> CreateThoughtInput {
    CreateThoughtInput {
        user_id: uid,
        content: "hello".into(),
        in_reply_to_id: None,
        visibility: None,
        content_warning: None,
        sensitive: false,
        mood: None,
    }
}

#[tokio::test]
async fn create_thought_saves_and_stages_outbox_event() {
    let store = TestStore::default();
    let outbox = TestOutbox::default();
    let u = user();
    store.users.lock().unwrap().push(u.clone());
    let out = create_thought(
        &store,
        &store,
        &store,
        &NoOpEventPublisher,
        &outbox,
        input(u.id.clone()),
    )
    .await
    .unwrap();
    assert_eq!(out.thought.content.as_str(), "hello");
    let staged = outbox.staged();
    assert_eq!(staged.len(), 1);
    assert!(matches!(staged[0], DomainEvent::ThoughtCreated { .. }));
}

#[tokio::test]
async fn delete_thought_stages_outbox_event() {
    let store = TestStore::default();
    let outbox = TestOutbox::default();
    let u = user();
    store.users.lock().unwrap().push(u.clone());
    let out = create_thought(
        &store,
        &store,
        &store,
        &NoOpEventPublisher,
        &NoOpOutboxWriter,
        input(u.id.clone()),
    )
    .await
    .unwrap();
    let tid = out.thought.id.clone();

    delete_thought(&store, &NoOpEventPublisher, &outbox, &tid, &u.id)
        .await
        .unwrap();

    let staged = outbox.staged();
    assert_eq!(staged.len(), 1);
    assert!(
        matches!(&staged[0], DomainEvent::ThoughtDeleted { thought_id, .. } if *thought_id == tid)
    );
}

#[tokio::test]
async fn delete_own_thought_succeeds() {
    let store = TestStore::default();
    let u = user();
    store.users.lock().unwrap().push(u.clone());
    let out = create_thought(
        &store,
        &store,
        &store,
        &NoOpEventPublisher,
        &NoOpOutboxWriter,
        input(u.id.clone()),
    )
    .await
    .unwrap();
    delete_thought(
        &store,
        &NoOpEventPublisher,
        &NoOpOutboxWriter,
        &out.thought.id,
        &u.id,
    )
    .await
    .unwrap();
    assert!(store.thoughts.lock().unwrap().is_empty());
}

#[tokio::test]
async fn delete_other_thought_returns_not_found() {
    let store = TestStore::default();
    let alice = user();
    let bob = User::new_local(
        UserId::new(),
        Username::new("bob").unwrap(),
        Email::new("bob@ex.com").unwrap(),
        PasswordHash("h".into()),
    );
    store
        .users
        .lock()
        .unwrap()
        .extend([alice.clone(), bob.clone()]);
    let out = create_thought(
        &store,
        &store,
        &store,
        &NoOpEventPublisher,
        &NoOpOutboxWriter,
        input(alice.id.clone()),
    )
    .await
    .unwrap();
    let err = delete_thought(
        &store,
        &NoOpEventPublisher,
        &NoOpOutboxWriter,
        &out.thought.id,
        &bob.id,
    )
    .await
    .unwrap_err();
    assert!(matches!(err, DomainError::NotFound));
}

#[tokio::test]
async fn edit_thought_changes_content_and_emits_event() {
    let store = TestStore::default();
    let alice = user();
    store.users.lock().unwrap().push(alice.clone());
    let out = create_thought(
        &store,
        &store,
        &store,
        &NoOpEventPublisher,
        &NoOpOutboxWriter,
        input(alice.id.clone()),
    )
    .await
    .unwrap();
    let tid = out.thought.id.clone();

    edit_thought(&store, &store, &tid, &alice.id, "updated".to_string())
        .await
        .unwrap();

    let saved = store
        .thoughts
        .lock()
        .unwrap()
        .iter()
        .find(|t| t.id == tid)
        .unwrap()
        .clone();
    assert_eq!(saved.content.as_str(), "updated");

    let events = store.events.lock().unwrap();
    assert!(events.iter().any(
        |e| matches!(e, DomainEvent::ThoughtUpdated { thought_id, .. } if thought_id == &tid)
    ));
}

#[tokio::test]
async fn create_reply_sets_in_reply_to_id() {
    let store = TestStore::default();
    let alice = user();
    store.users.lock().unwrap().push(alice.clone());
    let original = create_thought(
        &store,
        &store,
        &store,
        &NoOpEventPublisher,
        &NoOpOutboxWriter,
        input(alice.id.clone()),
    )
    .await
    .unwrap()
    .thought;

    create_thought(
        &store,
        &store,
        &store,
        &NoOpEventPublisher,
        &NoOpOutboxWriter,
        CreateThoughtInput {
            user_id: alice.id.clone(),
            content: "reply".into(),
            in_reply_to_id: Some(original.id.clone()),
            visibility: None,
            content_warning: None,
            sensitive: false,
            mood: None,
        },
    )
    .await
    .unwrap();

    let thoughts = store.thoughts.lock().unwrap();
    let reply = thoughts
        .iter()
        .find(|t| t.content.as_str() == "reply")
        .unwrap();
    assert_eq!(reply.in_reply_to_id, Some(original.id.clone()));
}

// enrichment_tests (combined from second cfg(test) block)

use domain::models::thought::{NewThought, Thought, Visibility};
use domain::ports::{ThoughtRepository, UserWriter};

fn make_user() -> User {
    User::new_local(
        UserId::new(),
        Username::new("alice").unwrap(),
        Email::new("a@a.com").unwrap(),
        PasswordHash("h".into()),
    )
}

fn make_thought(user_id: UserId) -> Thought {
    Thought::new_local(NewThought {
        id: ThoughtId::new(),
        user_id,
        content: Content::new_local(String::from("hello")).unwrap(),
        in_reply_to_id: None,
        visibility: Visibility::Public,
        content_warning: None,
        sensitive: false,
        mood: None,
    })
}

#[tokio::test]
async fn get_thought_view_returns_feed_entry() {
    let store = TestStore::default();
    let user = make_user();
    <TestStore as UserWriter>::save(&store, &user)
        .await
        .unwrap();
    let thought = make_thought(user.id.clone());
    <TestStore as ThoughtRepository>::save(&store, &thought)
        .await
        .unwrap();

    let entry = get_thought_view(&store, &store, &store, &thought.id, None)
        .await
        .unwrap();
    assert_eq!(entry.thought.id, thought.id);
    assert_eq!(entry.author.id, user.id);
    assert_eq!(entry.stats.like_count, 0);
    assert!(entry.viewer.is_none());
}

#[tokio::test]
async fn get_thought_view_returns_not_found_for_missing_thought() {
    let store = TestStore::default();
    let err = get_thought_view(&store, &store, &store, &ThoughtId::new(), None)
        .await
        .unwrap_err();
    assert!(matches!(err, DomainError::NotFound));
}

#[tokio::test]
async fn get_thread_views_batches_correctly() {
    let store = TestStore::default();
    let user = make_user();
    <TestStore as UserWriter>::save(&store, &user)
        .await
        .unwrap();
    let root = make_thought(user.id.clone());
    <TestStore as ThoughtRepository>::save(&store, &root)
        .await
        .unwrap();
    let reply = Thought::new_local(NewThought {
        id: ThoughtId::new(),
        user_id: user.id.clone(),
        content: Content::new_local(String::from("reply")).unwrap(),
        in_reply_to_id: Some(root.id.clone()),
        visibility: Visibility::Public,
        content_warning: None,
        sensitive: false,
        mood: None,
    });
    <TestStore as ThoughtRepository>::save(&store, &reply)
        .await
        .unwrap();

    let entries = get_thread_views(&store, &store, &store, &root.id, None)
        .await
        .unwrap();
    assert_eq!(entries.len(), 2);
}
