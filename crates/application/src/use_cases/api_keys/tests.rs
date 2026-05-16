use super::*;
use domain::{testing::TestStore, value_objects::UserId};

#[tokio::test]
async fn create_key_saves_hashed_not_raw() {
    let store = TestStore::default();
    let uid = UserId::new();
    let (key, raw) = create_api_key(&store, &uid, "my-key".to_string())
        .await
        .unwrap();
    assert_ne!(key.key_hash, raw, "stored hash must differ from raw key");
    assert!(!key.key_hash.is_empty());
    assert_eq!(key.name, "my-key");
    assert_eq!(key.user_id, uid);
    assert_eq!(store.api_keys.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn raw_key_verifies_against_stored_hash() {
    use sha2::{Digest, Sha256};
    let store = TestStore::default();
    let uid = UserId::new();
    let (key, raw) = create_api_key(&store, &uid, "test".to_string())
        .await
        .unwrap();
    let expected_hash = hex::encode(Sha256::digest(raw.as_bytes()));
    assert_eq!(key.key_hash, expected_hash);
}

#[tokio::test]
async fn delete_key_removes_it() {
    let store = TestStore::default();
    let uid = UserId::new();
    let (key, _) = create_api_key(&store, &uid, "k".to_string()).await.unwrap();
    delete_api_key(&store, &uid, &key.id).await.unwrap();
    assert!(store.api_keys.lock().unwrap().is_empty());
}

#[tokio::test]
async fn list_keys_returns_only_own_keys() {
    let store = TestStore::default();
    let alice = UserId::new();
    let bob = UserId::new();
    create_api_key(&store, &alice, "a".to_string())
        .await
        .unwrap();
    create_api_key(&store, &bob, "b".to_string()).await.unwrap();
    let alice_keys = list_api_keys(&store, &alice).await.unwrap();
    assert_eq!(alice_keys.len(), 1);
    assert_eq!(alice_keys[0].user_id, alice);
}
