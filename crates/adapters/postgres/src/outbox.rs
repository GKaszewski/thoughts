use async_trait::async_trait;
use domain::{errors::DomainError, events::DomainEvent, ports::OutboxWriter};
use event_payload::EventPayload;
use sqlx::PgPool;
use uuid::Uuid;

pub struct PgOutboxWriter {
    pool: PgPool,
}

impl PgOutboxWriter {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// Primary aggregate UUID for an event — used to populate `aggregate_id`.
fn aggregate_id(event: &DomainEvent) -> Uuid {
    match event {
        DomainEvent::ThoughtCreated { thought_id, .. } => thought_id.as_uuid(),
        DomainEvent::ThoughtDeleted { thought_id, .. } => thought_id.as_uuid(),
        DomainEvent::ThoughtUpdated { thought_id, .. } => thought_id.as_uuid(),
        DomainEvent::LikeAdded { thought_id, .. } => thought_id.as_uuid(),
        DomainEvent::LikeRemoved { thought_id, .. } => thought_id.as_uuid(),
        DomainEvent::BoostAdded { thought_id, .. } => thought_id.as_uuid(),
        DomainEvent::BoostRemoved { thought_id, .. } => thought_id.as_uuid(),
        DomainEvent::FollowRequested { follower_id, .. } => follower_id.as_uuid(),
        DomainEvent::FollowAccepted { follower_id, .. } => follower_id.as_uuid(),
        DomainEvent::FollowRejected { follower_id, .. } => follower_id.as_uuid(),
        DomainEvent::Unfollowed { follower_id, .. } => follower_id.as_uuid(),
        DomainEvent::UserBlocked { blocker_id, .. } => blocker_id.as_uuid(),
        DomainEvent::UserUnblocked { blocker_id, .. } => blocker_id.as_uuid(),
        DomainEvent::UserRegistered { user_id } => user_id.as_uuid(),
        DomainEvent::ProfileUpdated { user_id } => user_id.as_uuid(),
        DomainEvent::RemoteFollowAccepted { local_user_id, .. } => local_user_id.as_uuid(),
        DomainEvent::RemoteFollowRejected { local_user_id, .. } => local_user_id.as_uuid(),
        DomainEvent::ActorMoved { user_id, .. } => user_id.as_uuid(),
        DomainEvent::MentionReceived { thought_id, .. } => thought_id.as_uuid(),
    }
}

#[async_trait]
impl OutboxWriter for PgOutboxWriter {
    async fn append(&self, event: &DomainEvent) -> Result<(), DomainError> {
        let payload = EventPayload::from(event);
        let event_type = payload.subject();
        let payload_json =
            serde_json::to_value(&payload).map_err(|e| DomainError::Internal(e.to_string()))?;
        let agg_id = aggregate_id(event);

        sqlx::query(
            "INSERT INTO outbox_events (aggregate_id, event_type, payload) \
             VALUES ($1, $2, $3)",
        )
        .bind(agg_id)
        .bind(event_type)
        .bind(payload_json)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }
}
