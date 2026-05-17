use domain::{events::DomainEvent, ports::EventPublisher};
use event_payload::EventPayload;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;

pub struct OutboxRelay {
    pub pool: PgPool,
    pub publisher: Arc<dyn EventPublisher>,
    pub poll_interval: Duration,
}

#[derive(sqlx::FromRow)]
struct OutboxRow {
    seq: i64,
    event_type: String,
    payload: serde_json::Value,
}

impl OutboxRelay {
    pub async fn run(self) {
        loop {
            if let Err(e) = self.process_batch().await {
                tracing::error!("outbox relay error: {e}");
            }
            tokio::time::sleep(self.poll_interval).await;
        }
    }

    // NOTE: thoughts.save() and outbox.append() are not in the same DB transaction
    // (known architectural limitation — fixing requires transaction-sharing between
    // repositories, a larger refactor).
    async fn process_batch(&self) -> Result<(), sqlx::Error> {
        // Process one row at a time inside its own transaction so that
        // FOR UPDATE SKIP LOCKED actually holds the lock for the duration
        // of publish + mark_delivered. A batch SELECT without a surrounding
        // transaction releases locks immediately after autocommit.
        loop {
            let mut tx = self.pool.begin().await?;

            let row = sqlx::query_as::<_, OutboxRow>(
                "SELECT seq, event_type, payload \
                 FROM outbox_events \
                 WHERE delivered = false \
                 ORDER BY seq ASC \
                 LIMIT 1 \
                 FOR UPDATE SKIP LOCKED",
            )
            .fetch_optional(&mut *tx)
            .await?;

            let Some(row) = row else {
                tx.rollback().await?;
                break;
            };

            let payload: EventPayload = match serde_json::from_value(row.payload.clone()) {
                Ok(p) => p,
                Err(e) => {
                    tracing::error!(
                        seq = row.seq,
                        event_type = row.event_type,
                        "outbox: failed to deserialize payload: {e}"
                    );
                    // Mark delivered to avoid blocking; investigate manually.
                    sqlx::query(
                        "UPDATE outbox_events \
                         SET delivered = true, delivered_at = now() \
                         WHERE seq = $1",
                    )
                    .bind(row.seq)
                    .execute(&mut *tx)
                    .await?;
                    tx.commit().await?;
                    continue;
                }
            };

            let domain_event = match DomainEvent::try_from(payload) {
                Ok(ev) => ev,
                Err(e) => {
                    tracing::error!(
                        seq = row.seq,
                        "outbox: failed to convert to DomainEvent: {e}"
                    );
                    sqlx::query(
                        "UPDATE outbox_events \
                         SET delivered = true, delivered_at = now() \
                         WHERE seq = $1",
                    )
                    .bind(row.seq)
                    .execute(&mut *tx)
                    .await?;
                    tx.commit().await?;
                    continue;
                }
            };

            match self.publisher.publish(&domain_event).await {
                Ok(()) => {
                    sqlx::query(
                        "UPDATE outbox_events \
                         SET delivered = true, delivered_at = now() \
                         WHERE seq = $1",
                    )
                    .bind(row.seq)
                    .execute(&mut *tx)
                    .await?;
                    tx.commit().await?;
                    tracing::info!(
                        seq = row.seq,
                        event_type = row.event_type,
                        "outbox: delivered"
                    );
                }
                Err(e) => {
                    tracing::warn!(seq = row.seq, "outbox: publish failed (will retry): {e}");
                    tx.rollback().await?; // row stays undelivered, retried next poll
                }
            }
        }

        Ok(())
    }
}
