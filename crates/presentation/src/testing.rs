use crate::state::AppState;
use activitypub::{ActivityPubRepository, ActorApUrls, OutboxEntry};
use application::use_cases::profile::UploadConfig;
use async_trait::async_trait;
use domain::{
    errors::DomainError,
    ports::{AuthService, DataStream, GeneratedToken, MediaStore, PasswordHasher},
    testing::{NoOpOutboxWriter, TestStore},
    value_objects::{PasswordHash, ThoughtId, UserId},
};
use std::sync::Arc;

pub struct NoOpAuth;
impl AuthService for NoOpAuth {
    fn generate_token(&self, _uid: &UserId) -> Result<GeneratedToken, DomainError> {
        Err(DomainError::Internal("noop".into()))
    }
    fn validate_token(&self, _token: &str) -> Result<UserId, DomainError> {
        Err(DomainError::Unauthorized)
    }
}

pub struct NoOpHasher;
#[async_trait]
impl PasswordHasher for NoOpHasher {
    async fn hash(&self, _plain: &str) -> Result<PasswordHash, DomainError> {
        Err(DomainError::Internal("noop".into()))
    }
    async fn verify(&self, _plain: &str, _hash: &PasswordHash) -> Result<bool, DomainError> {
        Ok(false)
    }
}

pub struct NoOpApRepo;

#[async_trait]
impl ActivityPubRepository for NoOpApRepo {
    async fn outbox_entries_for_actor(&self, _: &UserId) -> Result<Vec<OutboxEntry>, DomainError> {
        Ok(vec![])
    }
    async fn outbox_page_for_actor(
        &self,
        _: &UserId,
        _: Option<chrono::DateTime<chrono::Utc>>,
        _: usize,
    ) -> Result<Vec<OutboxEntry>, DomainError> {
        Ok(vec![])
    }
    async fn find_remote_actor_id(&self, _: &str) -> Result<Option<UserId>, DomainError> {
        Ok(None)
    }
    async fn intern_remote_actor(&self, _: &str) -> Result<UserId, DomainError> {
        Err(DomainError::NotFound)
    }
    async fn update_remote_actor_display(
        &self,
        _: &UserId,
        _: Option<&str>,
        _: Option<&str>,
    ) -> Result<(), DomainError> {
        Ok(())
    }
    async fn accept_note(
        &self,
        _: activitypub::AcceptNoteInput<'_>,
    ) -> Result<ThoughtId, DomainError> {
        Ok(ThoughtId::from_uuid(uuid::Uuid::new_v4()))
    }
    async fn apply_note_update(&self, _: &str, _: &str) -> Result<(), DomainError> {
        Ok(())
    }
    async fn retract_note(&self, _: &str) -> Result<(), DomainError> {
        Ok(())
    }
    async fn retract_actor_notes(&self, _: &str) -> Result<(), DomainError> {
        Ok(())
    }
    async fn count_local_notes(&self) -> Result<u64, DomainError> {
        Ok(0)
    }
    async fn get_thought_ap_id(&self, _: &ThoughtId) -> Result<Option<String>, DomainError> {
        Ok(None)
    }
    async fn get_actor_ap_urls(&self, _: &UserId) -> Result<Option<ActorApUrls>, DomainError> {
        Ok(None)
    }
    async fn sync_remote_actor_to_user(&self, _: &str) -> Result<(), DomainError> {
        Ok(())
    }
}

pub struct NoOpMediaStore;

#[async_trait]
impl MediaStore for NoOpMediaStore {
    async fn put(&self, _key: &str, _data: DataStream) -> Result<(), DomainError> {
        Err(DomainError::Internal("noop".into()))
    }
    async fn get(&self, _key: &str) -> Result<DataStream, DomainError> {
        Err(DomainError::Internal("noop".into()))
    }
    async fn delete(&self, _key: &str) -> Result<(), DomainError> {
        Err(DomainError::Internal("noop".into()))
    }
}

pub fn make_state() -> AppState {
    let store = Arc::new(TestStore::default());
    AppState {
        users: store.clone(),
        thoughts: store.clone(),
        likes: store.clone(),
        boosts: store.clone(),
        follows: store.clone(),
        blocks: store.clone(),
        tags: store.clone(),
        api_keys: store.clone(),
        top_friends: store.clone(),
        notifications: store.clone(),
        remote_actors: store.clone(),
        feed: store.clone(),
        search: store.clone(),
        auth: Arc::new(NoOpAuth),
        hasher: Arc::new(NoOpHasher),
        events: store.clone(),
        outbox: Arc::new(NoOpOutboxWriter),
        federation: store.clone(),
        ap_repo: Arc::new(NoOpApRepo),
        remote_actor_connections: store.clone(),
        federation_scheduler: store.clone(),
        api_key_auth: store.clone(),
        engagement: store.clone(),
        media: Arc::new(NoOpMediaStore),
        upload_config: UploadConfig::default(),
        base_url: "http://localhost:3000".into(),
    }
}
