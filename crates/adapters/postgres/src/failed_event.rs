use chrono::{DateTime, Utc};
use sqlx::PgPool;

/// How many times a failed event is retried by the DLQ processor.
pub const DLQ_MAX_RETRIES: i32 = 3;
/// Quarantine period for the first DLQ retry (seconds). Doubles each retry.
pub const DLQ_INITIAL_BACKOFF_SECS: i64 = 300; // 5 minutes
/// How often the DLQ processor polls for due retries (seconds).
pub const DLQ_POLL_INTERVAL_SECS: u64 = 60;

#[derive(sqlx::FromRow)]
pub struct FailedEvent {
    pub id: uuid::Uuid,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub failed_at: DateTime<Utc>,
    pub retry_at: DateTime<Utc>,
    pub retry_count: i32,
    pub last_error: String,
}

pub struct PgFailedEventStore {
    pool: PgPool,
}

impl PgFailedEventStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Insert a newly exhausted event into the DLQ.
    pub async fn insert(
        &self,
        event_type: &str,
        payload: &serde_json::Value,
        last_error: &str,
    ) -> Result<(), sqlx::Error> {
        let retry_at = Utc::now() + chrono::Duration::seconds(DLQ_INITIAL_BACKOFF_SECS);
        sqlx::query(
            "INSERT INTO failed_events \
             (event_type, payload, retry_at, last_error) \
             VALUES ($1, $2, $3, $4)",
        )
        .bind(event_type)
        .bind(payload)
        .bind(retry_at)
        .bind(last_error)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Fetch all events due for retry (retry_at <= now, retry_count < DLQ_MAX_RETRIES).
    pub async fn poll_due(&self) -> Result<Vec<FailedEvent>, sqlx::Error> {
        sqlx::query_as::<_, FailedEvent>(
            "SELECT id, event_type, payload, failed_at, retry_at, retry_count, last_error \
             FROM failed_events \
             WHERE retry_at <= now() AND retry_count < $1 \
             ORDER BY retry_at \
             LIMIT 100",
        )
        .bind(DLQ_MAX_RETRIES)
        .fetch_all(&self.pool)
        .await
    }

    /// Advance a row after a republish attempt using exponential backoff.
    /// next_retry = now + initial * 2^retry_count
    pub async fn advance(&self, id: uuid::Uuid, error: Option<&str>) -> Result<(), sqlx::Error> {
        let current: i32 =
            sqlx::query_scalar("SELECT retry_count FROM failed_events WHERE id = $1")
                .bind(id)
                .fetch_one(&self.pool)
                .await?;

        let new_count = current + 1;
        let backoff_secs = DLQ_INITIAL_BACKOFF_SECS * (1_i64 << new_count.min(10));
        let retry_at = Utc::now() + chrono::Duration::seconds(backoff_secs);
        let last_error = error.unwrap_or("republish succeeded");

        sqlx::query(
            "UPDATE failed_events \
             SET retry_count = $1, retry_at = $2, last_error = $3 \
             WHERE id = $4",
        )
        .bind(new_count)
        .bind(retry_at)
        .bind(last_error)
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Park a permanently failed event (retry_count >= DLQ_MAX_RETRIES).
    pub async fn park_permanently(&self, id: uuid::Uuid) -> Result<(), sqlx::Error> {
        let far_future = Utc::now() + chrono::Duration::days(365);
        sqlx::query("UPDATE failed_events SET retry_at = $1 WHERE id = $2")
            .bind(far_future)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
