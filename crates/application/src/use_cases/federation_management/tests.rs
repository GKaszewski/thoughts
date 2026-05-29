use super::*;
use chrono::Utc;
use domain::models::remote_actor::RemoteActor;
use domain::testing::TestStore;

fn remote_actor(url: &str, handle: &str) -> RemoteActor {
    RemoteActor {
        url: url.to_string(),
        handle: handle.to_string(),
        display_name: None,
        avatar_url: None,
        bio: None,
        banner_url: None,
        also_known_as: vec![],
        outbox_url: None,
        followers_url: None,
        following_url: None,
        inbox_url: None,
        shared_inbox_url: None,
        attachment: vec![],
        last_fetched_at: Utc::now(),
    }
}

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
    accept_follow_request(&store, &store, &uid, "https://mastodon.social/users/alice")
        .await
        .unwrap();
}

#[tokio::test]
async fn reject_follow_request_returns_ok() {
    let store = TestStore::default();
    let uid = UserId::new();
    reject_follow_request(&store, &store, &uid, "https://mastodon.social/users/alice")
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

#[tokio::test]
async fn get_remote_friends_returns_intersection() {
    let store = TestStore::default();
    let uid = UserId::new();

    let bob = remote_actor("https://bob.example.com/users/bob", "bob@bob.example.com");
    let carol = remote_actor(
        "https://carol.example.com/users/carol",
        "carol@carol.example.com",
    );

    // uid follows bob and carol
    store
        .remote_following
        .lock()
        .unwrap()
        .extend([bob.clone(), carol.clone()]);
    // only bob follows back
    store.remote_followers.lock().unwrap().push(bob.clone());

    let friends = get_remote_friends(&store, &uid).await.unwrap();
    assert_eq!(friends.len(), 1);
    assert_eq!(friends[0].url, bob.url);
}

#[tokio::test]
async fn get_remote_friends_empty_when_no_mutual() {
    let store = TestStore::default();
    let uid = UserId::new();

    let bob = remote_actor("https://bob.example.com/users/bob", "bob@bob.example.com");
    store.remote_following.lock().unwrap().push(bob.clone());
    // bob does NOT follow back

    let friends = get_remote_friends(&store, &uid).await.unwrap();
    assert!(friends.is_empty());
}
