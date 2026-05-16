use async_trait::async_trait;
use chrono::{DateTime, Utc};
use url::Url;

#[async_trait]
pub trait ApObjectHandler: Send + Sync {
    /// Returns (ap_id, serialized object) for all local content owned by this user.
    /// Used by outbox (count) and backfill (delivery). Must only return locally-authored content.
    async fn get_local_objects_for_user(
        &self,
        user_id: uuid::Uuid,
    ) -> anyhow::Result<Vec<(Url, serde_json::Value)>>;

    /// Returns up to `limit` objects ordered newest-first, published before `before`.
    /// Returns (ap_id, object_json, published_at).
    async fn get_local_objects_page(
        &self,
        user_id: uuid::Uuid,
        before: Option<DateTime<Utc>>,
        limit: usize,
    ) -> anyhow::Result<Vec<(Url, serde_json::Value, DateTime<Utc>)>>;

    /// Incoming Create activity — persist remote content.
    async fn on_create(
        &self,
        ap_id: &Url,
        actor_url: &Url,
        object: serde_json::Value,
    ) -> anyhow::Result<()>;

    /// Incoming Update activity — update existing remote content.
    async fn on_update(
        &self,
        ap_id: &Url,
        actor_url: &Url,
        object: serde_json::Value,
    ) -> anyhow::Result<()>;

    /// Incoming Delete activity — remove specific remote content.
    async fn on_delete(&self, ap_id: &Url, actor_url: &Url) -> anyhow::Result<()>;

    /// Actor unfollowed/was removed — clean up all their remote content.
    async fn on_actor_removed(&self, actor_url: &Url) -> anyhow::Result<()>;

    /// Called when a remote actor likes a local thought.
    /// `object_url` is the AP URL of the liked note (e.g. `{base}/thoughts/{uuid}`).
    /// `actor_url` is the AP URL of the remote actor who sent the Like.
    async fn on_like(&self, object_url: &Url, actor_url: &Url) -> anyhow::Result<()>;

    /// Called when a remote actor boosts (Announce) a local thought.
    /// `object_url` is the AP URL of the announced note.
    /// `actor_url` is the AP URL of the remote actor who sent the Announce.
    async fn on_announce_received(&self, object_url: &Url, actor_url: &Url) -> anyhow::Result<()>;

    /// Called when a remote actor removes a Like from a local thought.
    async fn on_unlike(&self, object_url: &Url, actor_url: &Url) -> anyhow::Result<()>;

    /// Called when an inbound Note tags a local user with a Mention.
    async fn on_mention(
        &self,
        thought_ap_id: &Url,
        mentioned_user_uuid: uuid::Uuid,
        actor_url: &Url,
    ) -> anyhow::Result<()>;

    /// Total number of locally-authored posts across all users.
    async fn count_local_posts(&self) -> anyhow::Result<u64>;
}
