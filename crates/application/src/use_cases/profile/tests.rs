use super::*;
use domain::{
    errors::DomainError,
    models::user::User,
    testing::TestStore,
    value_objects::{Email, PasswordHash, UserId, Username},
};

fn make_user() -> User {
    User::new_local(
        UserId::new(),
        Username::new("alice").unwrap(),
        Email::new("alice@ex.com").unwrap(),
        PasswordHash("h".into()),
    )
}

#[tokio::test]
async fn set_top_friends_rejects_more_than_eight() {
    let store = TestStore::default();
    let uid = UserId::new();
    let friends: Vec<UserId> = (0..9).map(|_| UserId::new()).collect();
    let err = set_top_friends(&store, &uid, friends).await.unwrap_err();
    assert!(matches!(err, DomainError::InvalidInput(_)));
}

#[tokio::test]
async fn set_top_friends_assigns_sequential_positions() {
    let store = TestStore::default();
    let uid = UserId::new();
    let f1 = UserId::new();
    let f2 = UserId::new();
    let f3 = UserId::new();
    set_top_friends(&store, &uid, vec![f1.clone(), f2.clone(), f3.clone()])
        .await
        .unwrap();
    let tf = store.top_friends.lock().unwrap();
    assert_eq!(tf.len(), 3);
    let pos_f1 = tf
        .iter()
        .find(|t| t.friend_id == f1)
        .map(|t| t.position)
        .unwrap();
    let pos_f2 = tf
        .iter()
        .find(|t| t.friend_id == f2)
        .map(|t| t.position)
        .unwrap();
    assert!(pos_f1 < pos_f2, "f1 should come before f2");
}

#[tokio::test]
async fn get_user_by_username_returns_not_found_for_missing_user() {
    let store = TestStore::default();
    let err = get_user_by_username(&store, "nobody").await.unwrap_err();
    assert!(matches!(err, DomainError::NotFound));
}

#[tokio::test]
async fn get_user_by_username_returns_correct_user() {
    let store = TestStore::default();
    let user = make_user();
    store.users.lock().unwrap().push(user.clone());
    let found = get_user_by_username(&store, "alice").await.unwrap();
    assert_eq!(found.id, user.id);
}
