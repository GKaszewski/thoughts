use async_trait::async_trait;
use domain::{
    errors::DomainError,
    models::thought::Thought,
    value_objects::{ThoughtId, UserId, Username},
};

/// AP-protocol endpoints for a locally-stored user (local or interned remote).
#[derive(Debug, Clone)]
pub struct ActorApUrls {
    pub ap_id: String,
    pub inbox_url: String,
}

/// A local thought ready for AP serialization, with the author's username
/// pre-joined so the handler can build AP URLs without a second query.
#[derive(Debug, Clone)]
pub struct OutboxEntry {
    pub thought: Thought,
    pub author_username: Username,
}

#[async_trait]
pub trait ActivityPubRepository: Send + Sync {
    // ── Outbox (local → remote) ──────────────────────────────────────

    /// All public local thoughts for this actor. Used for outbox totals
    /// and full-collection delivery.
    async fn outbox_entries_for_actor(
        &self,
        user_id: &UserId,
    ) -> Result<Vec<OutboxEntry>, DomainError>;

    /// Cursor page of public local thoughts, newest-first, before `before`.
    /// Used for OrderedCollectionPage responses.
    async fn outbox_page_for_actor(
        &self,
        user_id: &UserId,
        before: Option<chrono::DateTime<chrono::Utc>>,
        limit: usize,
    ) -> Result<Vec<OutboxEntry>, DomainError>;

    // ── Remote actor resolution ──────────────────────────────────────

    /// Find the local UserId for a remote actor by its AP URL.
    async fn find_remote_actor_id(&self, actor_ap_url: &str)
    -> Result<Option<UserId>, DomainError>;

    /// Ensure a remote actor placeholder exists; create one if absent.
    /// Idempotent — safe to call multiple times with the same URL.
    async fn intern_remote_actor(&self, actor_ap_url: &str) -> Result<UserId, DomainError>;

    /// Update display_name and avatar_url for an already-interned remote actor.
    async fn update_remote_actor_display(
        &self,
        user_id: &UserId,
        display_name: Option<&str>,
        avatar_url: Option<&str>,
    ) -> Result<(), DomainError>;

    // ── Inbox processing (remote → local) ───────────────────────────

    /// Persist an incoming remote Note. Idempotent on ap_id.
    #[allow(clippy::too_many_arguments)]
    async fn accept_note(
        &self,
        ap_id: &str,
        author_id: &UserId,
        content: &str,
        published: chrono::DateTime<chrono::Utc>,
        sensitive: bool,
        content_warning: Option<String>,
        visibility: &str,
        in_reply_to: Option<&str>,
    ) -> Result<ThoughtId, DomainError>;

    /// Apply an Update to a previously accepted remote Note.
    async fn apply_note_update(&self, ap_id: &str, new_content: &str) -> Result<(), DomainError>;

    /// Remove a specific remote Note (Delete activity). Only touches
    /// remotely-originated thoughts.
    async fn retract_note(&self, ap_id: &str) -> Result<(), DomainError>;

    /// Remove all Notes from a remote actor (actor-level Delete/Tombstone).
    async fn retract_actor_notes(&self, actor_ap_url: &str) -> Result<(), DomainError>;

    // ── Node-level stats ─────────────────────────────────────────────

    /// Total locally-authored thought count for NodeInfo responses.
    async fn count_local_notes(&self) -> Result<u64, DomainError>;

    /// Return the ActivityPub object URL for a thought, if one is stored.
    /// Returns None for local thoughts (caller constructs URL from base_url + thought_id).
    async fn get_thought_ap_id(
        &self,
        thought_id: &ThoughtId,
    ) -> Result<Option<String>, DomainError>;

    /// Return the AP actor URL and inbox URL for a user, if stored.
    /// Returns None for users that have not been federated.
    async fn get_actor_ap_urls(&self, user_id: &UserId)
    -> Result<Option<ActorApUrls>, DomainError>;
}

#[async_trait]
pub trait OutboundFederationPort: Send + Sync {
    /// Fan out a new local Note to all accepted followers.
    async fn broadcast_create(
        &self,
        author_user_id: &UserId,
        thought: &Thought,
        author_username: &str,
        in_reply_to_url: Option<&str>,
    ) -> Result<(), DomainError>;

    /// Fan out a Delete tombstone for a now-deleted local Note.
    /// `thought_ap_id` is pre-constructed by the caller because the thought
    /// has already been deleted from the DB when this fires.
    async fn broadcast_delete(
        &self,
        author_user_id: &UserId,
        thought_ap_id: &str,
    ) -> Result<(), DomainError>;

    /// Fan out an Update(Note) for an edited local thought.
    async fn broadcast_update(
        &self,
        author_user_id: &UserId,
        thought: &Thought,
        author_username: &str,
        in_reply_to_url: Option<&str>,
    ) -> Result<(), DomainError>;

    /// Fan out an Announce(object_ap_id) for a boost.
    async fn broadcast_announce(
        &self,
        booster_user_id: &UserId,
        object_ap_id: &str,
    ) -> Result<(), DomainError>;

    /// Fan out an Undo(Announce) to followers when a boost is removed.
    async fn broadcast_undo_announce(
        &self,
        booster_user_id: &UserId,
        object_ap_id: &str,
    ) -> Result<(), DomainError>;

    /// Send a Like activity to a remote thought author's inbox.
    /// Only called when a LOCAL user likes a REMOTE thought (one with an ap_id).
    async fn broadcast_like(
        &self,
        liker_user_id: &UserId,
        object_ap_id: &str,
        author_inbox_url: &str,
    ) -> Result<(), DomainError>;

    /// Send Undo(Like) to a remote thought author's inbox.
    async fn broadcast_undo_like(
        &self,
        liker_user_id: &UserId,
        object_ap_id: &str,
        author_inbox_url: &str,
    ) -> Result<(), DomainError>;

    /// Fan out an Update(Actor) to all accepted followers after a profile change.
    async fn broadcast_actor_update(&self, user_id: &UserId) -> Result<(), DomainError>;
}
