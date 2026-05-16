use domain::{errors::DomainError, events::DomainEvent, ports::EventPublisher};
use postgres::failed_event::{PgFailedEventStore, DLQ_MAX_RETRIES, DLQ_POLL_INTERVAL_SECS};
use std::sync::Arc;

/// Background task: polls `failed_events` and republishes due rows to the event bus.
pub async fn run_dlq_processor(store: Arc<PgFailedEventStore>, publisher: Arc<dyn EventPublisher>) {
    let interval = std::time::Duration::from_secs(DLQ_POLL_INTERVAL_SECS);
    loop {
        tokio::time::sleep(interval).await;
        if let Err(e) = process_due(&store, &*publisher).await {
            tracing::error!("DLQ processor error: {e}");
        }
    }
}

async fn process_due(
    store: &PgFailedEventStore,
    publisher: &dyn EventPublisher,
) -> Result<(), sqlx::Error> {
    let due = store.poll_due().await?;
    if due.is_empty() {
        return Ok(());
    }
    tracing::info!(count = due.len(), "DLQ: processing due events");

    for row in due {
        if row.retry_count >= DLQ_MAX_RETRIES {
            tracing::error!(
                id = %row.id,
                event_type = %row.event_type,
                retry_count = row.retry_count,
                "DLQ: event permanently failed — parking",
            );
            store.park_permanently(row.id).await?;
            continue;
        }

        let republish_result = republish(&row.payload, publisher).await;

        match republish_result {
            Ok(()) => {
                tracing::info!(id = %row.id, "DLQ: republished successfully");
                store.advance(row.id, None).await?;
            }
            Err(e) => {
                tracing::warn!(id = %row.id, error = %e, "DLQ: republish failed");
                store.advance(row.id, Some(&e.to_string())).await?;
            }
        }
    }
    Ok(())
}

async fn republish(
    payload: &serde_json::Value,
    publisher: &dyn EventPublisher,
) -> Result<(), DomainError> {
    use event_payload::EventPayload;
    let ep: EventPayload = serde_json::from_value(payload.clone())
        .map_err(|e| DomainError::Internal(format!("DLQ deserialize: {e}")))?;
    let event = DomainEvent::try_from(ep)
        .map_err(|e| DomainError::Internal(format!("DLQ event conversion: {e}")))?;
    publisher.publish(&event).await
}
