/// Test helpers for application-layer tests that need activitypub traits.
use activitypub::{ActivityPubRepository, ActorApUrls, OutboxEntry};
use async_trait::async_trait;
use domain::{
    errors::DomainError,
    models::user::User,
    testing::TestStore,
    value_objects::{Email, PasswordHash, ThoughtId, UserId, Username},
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Extends `TestStore` with AP-specific lookup maps.
#[derive(Default, Clone)]
pub struct TestApRepo {
    pub inner: TestStore,
    /// UserId → ActorApUrls (for get_actor_ap_urls)
    pub actor_ap_urls: Arc<Mutex<HashMap<UserId, ActorApUrls>>>,
}

impl TestApRepo {
    pub fn new(inner: TestStore) -> Self {
        Self {
            inner,
            actor_ap_urls: Default::default(),
        }
    }
}

#[async_trait]
impl ActivityPubRepository for TestApRepo {
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
        actor_ap_url: &str,
    ) -> Result<Option<UserId>, DomainError> {
        Ok(self
            .inner
            .actor_ap_ids
            .lock()
            .unwrap()
            .get(actor_ap_url)
            .cloned())
    }
    async fn intern_remote_actor(&self, actor_ap_url: &str) -> Result<UserId, DomainError> {
        if let Some(uid) = self.find_remote_actor_id(actor_ap_url).await? {
            return Ok(uid);
        }
        let uid = UserId::new();
        let handle = url::Url::parse(actor_ap_url)
            .map(|u| u.path().trim_start_matches('/').replace('/', "_"))
            .unwrap_or_else(|_| format!("remote_{}", &uid.to_string()[..8]));
        let user = User {
            id: uid.clone(),
            username: Username::from_trusted(handle),
            email: Email::from_trusted(format!("{}@remote", uid)),
            password_hash: PasswordHash("".into()),
            display_name: None,
            bio: None,
            avatar_url: None,
            header_url: None,
            custom_css: None,
            local: false,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        self.inner.users.lock().unwrap().push(user);
        self.inner
            .actor_ap_ids
            .lock()
            .unwrap()
            .insert(actor_ap_url.to_string(), uid.clone());
        Ok(uid)
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
        Ok(self
            .inner
            .thoughts
            .lock()
            .unwrap()
            .iter()
            .filter(|t| t.local)
            .count() as u64)
    }
    async fn get_thought_ap_id(
        &self,
        thought_id: &ThoughtId,
    ) -> Result<Option<String>, DomainError> {
        Ok(self
            .inner
            .thought_ap_ids
            .lock()
            .unwrap()
            .get(thought_id)
            .cloned())
    }
    async fn get_actor_ap_urls(
        &self,
        user_id: &UserId,
    ) -> Result<Option<ActorApUrls>, DomainError> {
        Ok(self.actor_ap_urls.lock().unwrap().get(user_id).cloned())
    }
}
