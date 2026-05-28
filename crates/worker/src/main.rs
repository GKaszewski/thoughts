mod dlq;
mod factory;
mod handlers;
mod outbox_relay;

use domain::ports::EventConsumer;
use futures::StreamExt;
use nats::CONSUMER_MAX_DELIVER;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL required");
    let nats_url = std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".into());
    let base_url = std::env::var("BASE_URL").expect("BASE_URL required");

    tracing::info!("Building worker...");
    let infra = factory::build(&database_url, &base_url, &nats_url).await;

    // Spawn DLQ processor as a background task.
    tokio::spawn(dlq::run_dlq_processor(
        infra.dlq_store.clone(),
        infra.event_publisher.clone(),
    ));

    // Spawn outbox relay — polls DB for undelivered events and publishes them.
    tokio::spawn(
        outbox_relay::OutboxRelay {
            pool: infra.pool.clone(),
            publisher: infra.event_publisher.clone(),
            poll_interval: std::time::Duration::from_secs(5),
        }
        .run(),
    );

    tracing::info!("Worker started, consuming events...");
    let mut stream = infra.consumer.consume();
    while let Some(result) = stream.next().await {
        match result {
            Ok(envelope) => {
                let event = &envelope.event;
                let event_type = event_payload::EventPayload::from(event).subject();
                tracing::info!(
                    event_type,
                    delivery = envelope.delivery_count,
                    "received event"
                );

                let n = infra.handlers.notification.handle(event).await;
                let f = infra.handlers.federation.handle(event).await;
                let fm = infra.handlers.federation_management.handle(event).await;

                if n.is_ok() && f.is_ok() && fm.is_ok() {
                    (envelope.ack)();
                    tracing::info!(event_type, "event handled ok");
                } else {
                    if let Err(e) = &n {
                        tracing::error!("notification handler: {e}");
                    }
                    if let Err(e) = &f {
                        tracing::error!("federation handler: {e}");
                    }
                    if let Err(e) = &fm {
                        tracing::error!("federation management handler: {e}");
                    }

                    // Last delivery attempt -> move to DLQ then ack.
                    // Earlier attempts -> nack so NATS retries.
                    if envelope.delivery_count >= CONSUMER_MAX_DELIVER as u64 {
                        let error_msg = n
                            .err()
                            .or(f.err())
                            .or(fm.err())
                            .map(|e| e.to_string())
                            .unwrap_or_else(|| "unknown error".into());

                        // Serialize event back to payload for storage.
                        let ep = event_payload::EventPayload::from(event);
                        let event_type = ep.subject().to_string();
                        let payload = serde_json::to_value(&ep).unwrap_or(serde_json::Value::Null);

                        if let Err(e) = infra
                            .dlq_store
                            .insert(&event_type, &payload, &error_msg)
                            .await
                        {
                            tracing::error!("DLQ insert failed: {e} — message lost");
                        } else {
                            tracing::warn!(
                                event_type,
                                delivery_count = envelope.delivery_count,
                                "event exhausted — moved to DLQ"
                            );
                        }
                        (envelope.ack)(); // ack from NATS — DLQ owns it now
                    } else {
                        (envelope.nack)();
                    }
                }
            }
            Err(e) => tracing::error!("consumer error: {e}"),
        }
    }
}
