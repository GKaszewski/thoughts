use async_trait::async_trait;
use domain::{
    errors::DomainError,
    ports::{ApiKeyRepository, ApiKeyService},
    value_objects::UserId,
};
use sha2::{Digest, Sha256};
use std::sync::Arc;

pub struct ApiKeyServiceImpl {
    repo: Arc<dyn ApiKeyRepository>,
}

impl ApiKeyServiceImpl {
    pub fn new(repo: Arc<dyn ApiKeyRepository>) -> Self {
        Self { repo }
    }

    fn hash(raw: &str) -> String {
        hex::encode(Sha256::digest(raw.as_bytes()))
    }
}

#[async_trait]
impl ApiKeyService for ApiKeyServiceImpl {
    async fn validate_key(&self, raw_key: &str) -> Result<Option<UserId>, DomainError> {
        let hash = Self::hash(raw_key);
        Ok(self.repo.find_by_hash(&hash).await?.map(|k| k.user_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::Utc;
    use domain::{
        errors::DomainError,
        models::api_key::ApiKey,
        ports::ApiKeyRepository,
        value_objects::{ApiKeyId, UserId},
    };
    use std::sync::{Arc, Mutex};

    struct FakeApiKeyRepo(Mutex<Vec<ApiKey>>);

    #[async_trait]
    impl ApiKeyRepository for FakeApiKeyRepo {
        async fn save(&self, key: &ApiKey) -> Result<(), DomainError> {
            self.0.lock().unwrap().push(key.clone());
            Ok(())
        }
        async fn find_by_hash(&self, hash: &str) -> Result<Option<ApiKey>, DomainError> {
            Ok(self.0.lock().unwrap().iter().find(|k| k.key_hash == hash).cloned())
        }
        async fn list_for_user(&self, _uid: &UserId) -> Result<Vec<ApiKey>, DomainError> {
            Ok(vec![])
        }
        async fn delete(&self, _id: &ApiKeyId, _uid: &UserId) -> Result<(), DomainError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn validate_known_key_returns_user_id() {
        let uid = UserId::new();
        let raw = "super-secret-key";
        let hash = ApiKeyServiceImpl::hash(raw);
        let key = ApiKey {
            id: ApiKeyId::new(),
            user_id: uid.clone(),
            key_hash: hash,
            name: "test".into(),
            created_at: Utc::now(),
        };
        let repo = Arc::new(FakeApiKeyRepo(Mutex::new(vec![key])));
        let svc = ApiKeyServiceImpl::new(repo);
        let result = svc.validate_key(raw).await.unwrap();
        assert_eq!(result.unwrap().as_uuid(), uid.as_uuid());
    }

    #[tokio::test]
    async fn validate_unknown_key_returns_none() {
        let repo = Arc::new(FakeApiKeyRepo(Mutex::new(vec![])));
        let svc = ApiKeyServiceImpl::new(repo);
        let result = svc.validate_key("unknown-key").await.unwrap();
        assert!(result.is_none());
    }
}
