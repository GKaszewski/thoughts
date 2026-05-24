use super::*;
use domain::{
    errors::DomainError,
    models::user::User,
    testing::TestStore,
    value_objects::{Email, PasswordHash, UserId, Username},
};
use std::sync::{Arc, Mutex};

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

// ── upload tests ──────────────────────────────────────────────────────────────

use bytes::Bytes;
use domain::ports::{DataStream, MediaStore};
use std::collections::HashMap;

#[derive(Default, Clone)]
struct MockMedia {
    store: Arc<Mutex<HashMap<String, Bytes>>>,
    deleted: Arc<Mutex<Vec<String>>>,
}

#[async_trait::async_trait]
impl MediaStore for MockMedia {
    async fn put(&self, key: &str, mut data: DataStream) -> Result<(), DomainError> {
        use futures::stream::StreamExt;
        let mut buf = Vec::new();
        while let Some(chunk) = data.next().await {
            buf.extend_from_slice(&chunk?);
        }
        self.store
            .lock()
            .unwrap()
            .insert(key.to_string(), Bytes::from(buf));
        Ok(())
    }
    async fn get(&self, key: &str) -> Result<DataStream, DomainError> {
        let bytes = self
            .store
            .lock()
            .unwrap()
            .get(key)
            .cloned()
            .ok_or(DomainError::NotFound)?;
        Ok(Box::pin(futures::stream::once(async move { Ok(bytes) })))
    }
    async fn delete(&self, key: &str) -> Result<(), DomainError> {
        self.store.lock().unwrap().remove(key);
        self.deleted.lock().unwrap().push(key.to_string());
        Ok(())
    }
}

fn default_cfg() -> UploadConfig {
    UploadConfig::default()
}

#[tokio::test]
async fn upload_avatar_rejects_unsupported_mime() {
    let store = TestStore::default();
    let media = MockMedia::default();
    let user = make_user();
    store.users.lock().unwrap().push(user.clone());
    let err = upload_avatar(
        &store,
        &media,
        &store,
        &user.id,
        "http://localhost",
        &default_cfg(),
        "text/plain",
        Bytes::from("hi"),
    )
    .await
    .unwrap_err();
    assert!(matches!(err, DomainError::InvalidInput(_)));
}

#[tokio::test]
async fn upload_avatar_rejects_oversized_data() {
    let store = TestStore::default();
    let media = MockMedia::default();
    let user = make_user();
    store.users.lock().unwrap().push(user.clone());
    let big = Bytes::from(vec![0u8; 6 * 1024 * 1024]);
    let err = upload_avatar(
        &store,
        &media,
        &store,
        &user.id,
        "http://localhost",
        &default_cfg(),
        "image/jpeg",
        big,
    )
    .await
    .unwrap_err();
    assert!(matches!(err, DomainError::InvalidInput(_)));
}

#[tokio::test]
async fn upload_avatar_stores_file_and_updates_url() {
    let store = TestStore::default();
    let media = MockMedia::default();
    let user = make_user();
    store.users.lock().unwrap().push(user.clone());
    upload_avatar(
        &store,
        &media,
        &store,
        &user.id,
        "http://localhost",
        &default_cfg(),
        "image/jpeg",
        Bytes::from("img"),
    )
    .await
    .unwrap();
    let key = format!("users/{}/avatar.jpg", user.id.as_uuid());
    assert!(media.store.lock().unwrap().contains_key(&key));
    let saved = store
        .users
        .lock()
        .unwrap()
        .iter()
        .find(|u| u.id == user.id)
        .unwrap()
        .clone();
    assert_eq!(
        saved.avatar_url,
        Some(format!("http://localhost/media/{key}"))
    );
}

#[tokio::test]
async fn upload_avatar_deletes_old_file_on_reupload() {
    let store = TestStore::default();
    let media = MockMedia::default();
    let mut user = make_user();
    let old_key = format!("users/{}/avatar.png", user.id.as_uuid());
    user.avatar_url = Some(format!("http://localhost/media/{old_key}"));
    store.users.lock().unwrap().push(user.clone());
    media
        .store
        .lock()
        .unwrap()
        .insert(old_key.clone(), Bytes::from("old"));
    upload_avatar(
        &store,
        &media,
        &store,
        &user.id,
        "http://localhost",
        &default_cfg(),
        "image/jpeg",
        Bytes::from("new"),
    )
    .await
    .unwrap();
    assert!(!media.store.lock().unwrap().contains_key(&old_key));
    assert!(media.deleted.lock().unwrap().contains(&old_key));
}

#[tokio::test]
async fn upload_banner_stores_file_and_updates_header_url() {
    let store = TestStore::default();
    let media = MockMedia::default();
    let user = make_user();
    store.users.lock().unwrap().push(user.clone());
    upload_banner(
        &store,
        &media,
        &store,
        &user.id,
        "http://localhost",
        &default_cfg(),
        "image/png",
        Bytes::from("banner"),
    )
    .await
    .unwrap();
    let key = format!("users/{}/banner.png", user.id.as_uuid());
    assert!(media.store.lock().unwrap().contains_key(&key));
    let saved = store
        .users
        .lock()
        .unwrap()
        .iter()
        .find(|u| u.id == user.id)
        .unwrap()
        .clone();
    assert_eq!(
        saved.header_url,
        Some(format!("http://localhost/media/{key}"))
    );
}

#[tokio::test]
async fn upload_banner_deletes_old_file_on_reupload() {
    let store = TestStore::default();
    let media = MockMedia::default();
    let mut user = make_user();
    let old_key = format!("users/{}/banner.jpg", user.id.as_uuid());
    user.header_url = Some(format!("http://localhost/media/{old_key}"));
    store.users.lock().unwrap().push(user.clone());
    media
        .store
        .lock()
        .unwrap()
        .insert(old_key.clone(), Bytes::from("old"));
    upload_banner(
        &store,
        &media,
        &store,
        &user.id,
        "http://localhost",
        &default_cfg(),
        "image/png",
        Bytes::from("new"),
    )
    .await
    .unwrap();
    assert!(!media.store.lock().unwrap().contains_key(&old_key));
    assert!(media.deleted.lock().unwrap().contains(&old_key));
}
