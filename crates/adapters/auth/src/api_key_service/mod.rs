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
mod tests;
