use chrono::Utc;
use domain::{
    errors::DomainError,
    models::api_key::ApiKey,
    ports::ApiKeyRepository,
    value_objects::{ApiKeyId, UserId},
};

pub async fn list_api_keys(
    keys: &dyn ApiKeyRepository,
    user_id: &UserId,
) -> Result<Vec<ApiKey>, DomainError> {
    keys.list_for_user(user_id).await
}

pub async fn create_api_key(
    keys: &dyn ApiKeyRepository,
    user_id: &UserId,
    name: String,
) -> Result<(ApiKey, String), DomainError> {
    let raw_key = uuid::Uuid::new_v4().to_string().replace('-', "");
    let key_hash = sha256_hex(&raw_key);
    let key = ApiKey {
        id: ApiKeyId::new(),
        user_id: user_id.clone(),
        key_hash,
        name,
        created_at: Utc::now(),
    };
    keys.save(&key).await?;
    Ok((key, raw_key))
}

pub async fn delete_api_key(
    keys: &dyn ApiKeyRepository,
    user_id: &UserId,
    key_id: &ApiKeyId,
) -> Result<(), DomainError> {
    keys.delete(key_id, user_id).await
}

fn sha256_hex(s: &str) -> String {
    use sha2::{Digest, Sha256};
    let hash = Sha256::digest(s.as_bytes());
    hex::encode(hash)
}

#[cfg(test)]
mod tests {
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
}
