use crate::state::AppState;
use activitypub::{ActivityPubRepository, ActorApUrls, OutboxEntry};
use async_trait::async_trait;
use domain::{
    errors::DomainError,
    ports::{AuthService, GeneratedToken, PasswordHasher},
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

/// No-op ActivityPubRepository for presentation layer tests.
pub struct NoOpApRepo;

#[async_trait]
impl ActivityPubRepository for NoOpApRepo {
    async fn outbox_entries_for_actor(
        &self,
        _uid: &UserId,
    ) -> Result<Vec<OutboxEntry>, DomainError> {
        Ok(vec![])
    }
    async fn outbox_page_for_actor(
        &self,
        _uid: &UserId,
        _before: Option<chrono::DateTime<chrono::Utc>>,
        _limit: usize,
    ) -> Result<Vec<OutboxEntry>, DomainError> {
        Ok(vec![])
    }
    async fn find_remote_actor_id(
        &self,
        _actor_ap_url: &str,
    ) -> Result<Option<UserId>, DomainError> {
        Ok(None)
    }
    async fn intern_remote_actor(&self, _actor_ap_url: &str) -> Result<UserId, DomainError> {
        Err(DomainError::NotFound)
    }
    async fn update_remote_actor_display(
        &self,
        _user_id: &UserId,
        _display_name: Option<&str>,
        _avatar_url: Option<&str>,
    ) -> Result<(), DomainError> {
        Ok(())
    }
    async fn accept_note(
        &self,
        _input: activitypub::AcceptNoteInput<'_>,
    ) -> Result<ThoughtId, DomainError> {
        Ok(ThoughtId::from_uuid(uuid::Uuid::new_v4()))
    }
    async fn apply_note_update(&self, _ap_id: &str, _new_content: &str) -> Result<(), DomainError> {
        Ok(())
    }
    async fn retract_note(&self, _ap_id: &str) -> Result<(), DomainError> {
        Ok(())
    }
    async fn retract_actor_notes(&self, _actor_ap_url: &str) -> Result<(), DomainError> {
        Ok(())
    }
    async fn count_local_notes(&self) -> Result<u64, DomainError> {
        Ok(0)
    }
    async fn get_thought_ap_id(
        &self,
        _thought_id: &ThoughtId,
    ) -> Result<Option<String>, DomainError> {
        Ok(None)
    }
    async fn get_actor_ap_urls(
        &self,
        _user_id: &UserId,
    ) -> Result<Option<ActorApUrls>, DomainError> {
        Ok(None)
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
    }
}
