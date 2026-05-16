use super::*;
use domain::testing::TestStore;

#[tokio::test]
async fn list_pending_returns_empty_by_default() {
    let store = TestStore::default();
    let uid = UserId::new();
    let result = list_pending_requests(&store, &uid).await.unwrap();
    assert!(result.is_empty());
}

#[tokio::test]
async fn accept_follow_request_returns_ok() {
    let store = TestStore::default();
    let uid = UserId::new();
    accept_follow_request(&store, &uid, "https://mastodon.social/users/alice")
        .await
        .unwrap();
}

#[tokio::test]
async fn reject_follow_request_returns_ok() {
    let store = TestStore::default();
    let uid = UserId::new();
    reject_follow_request(&store, &uid, "https://mastodon.social/users/alice")
        .await
        .unwrap();
}

#[tokio::test]
async fn list_remote_followers_returns_empty_by_default() {
    let store = TestStore::default();
    let uid = UserId::new();
    let result = list_remote_followers(&store, &uid).await.unwrap();
    assert!(result.is_empty());
}

#[tokio::test]
async fn remove_remote_follower_returns_ok() {
    let store = TestStore::default();
    let uid = UserId::new();
    remove_remote_follower(&store, &uid, "https://mastodon.social/users/alice")
        .await
        .unwrap();
}

#[tokio::test]
async fn list_remote_following_returns_empty_by_default() {
    let store = TestStore::default();
    let uid = UserId::new();
    let result = list_remote_following(&store, &uid).await.unwrap();
    assert!(result.is_empty());
}
