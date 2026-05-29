mod dlq;
mod factory;
mod handlers;
mod outbox_relay;

use domain::{errors::DomainError, events::DomainEvent};
use event_payload::EventPayload;
use event_transport::MessageSource;
use futures::StreamExt;
use nats::CONSUMER_MAX_DELIVER;
use url::Url;
use uuid::Uuid;

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

    tokio::spawn(dlq::run_dlq_processor(
        infra.dlq_store.clone(),
        infra.event_publisher.clone(),
    ));

    tokio::spawn(
        outbox_relay::OutboxRelay {
            pool: infra.pool.clone(),
            publisher: infra.event_publisher.clone(),
            poll_interval: std::time::Duration::from_secs(5),
        }
        .run(),
    );

    tracing::info!("Worker started, consuming events...");
    let mut stream = infra.message_source.messages();
    while let Some(result) = stream.next().await {
        match result {
            Err(e) => tracing::error!("consumer error: {e}"),
            Ok(raw) => {
                let payload = match serde_json::from_slice::<EventPayload>(&raw.payload) {
                    Ok(p) => p,
                    Err(e) => {
                        tracing::warn!("failed to deserialize event payload — acking: {e}");
                        (raw.ack)();
                        continue;
                    }
                };

                let event_type = payload.subject();
                tracing::info!(event_type, delivery = raw.delivery_count, "received event");

                let outcome: Result<(), DomainError> = match payload {
                    // ── k-ap federation events ────────────────────────────
                    EventPayload::FederationDeliveryRequested {
                        inbox,
                        activity,
                        signing_actor_id,
                    } => {
                        let result = async {
                            let inbox_url = Url::parse(&inbox)
                                .map_err(|e| DomainError::Internal(e.to_string()))?;
                            let actor_id = Uuid::parse_str(&signing_actor_id)
                                .map_err(|e| DomainError::Internal(e.to_string()))?;
                            infra
                                .raw_ap_service
                                .deliver_to_inbox(inbox_url, activity, actor_id)
                                .await
                                .map_err(|e| DomainError::Internal(e.to_string()))
                        }
                        .await;
                        result
                    }
                    EventPayload::FederationBackfillRequested {
                        owner_user_id,
                        follower_inbox_url,
                    } => {
                        let result = async {
                            let owner_id = Uuid::parse_str(&owner_user_id)
                                .map_err(|e| DomainError::Internal(e.to_string()))?;
                            infra
                                .raw_ap_service
                                .run_backfill_for_follower(owner_id, follower_inbox_url)
                                .await
                                .map_err(|e| DomainError::Internal(e.to_string()))
                        }
                        .await;
                        result
                    }

                    // ── domain events ──────────────────────────────────────
                    p => match DomainEvent::try_from(p) {
                        Err(e) => {
                            tracing::warn!("unknown event type — acking: {e}");
                            (raw.ack)();
                            continue;
                        }
                        Ok(event) => {
                            let n = infra.handlers.notification.handle(&event).await;
                            let f = infra.handlers.federation.handle(&event).await;
                            let fm = infra.handlers.federation_management.handle(&event).await;
                            match (n, f, fm) {
                                (Ok(()), Ok(()), Ok(())) => Ok(()),
                                (n, f, fm) => {
                                    if let Err(e) = &n {
                                        tracing::error!("notification handler: {e}");
                                    }
                                    if let Err(e) = &f {
                                        tracing::error!("federation handler: {e}");
                                    }
                                    if let Err(e) = &fm {
                                        tracing::error!("federation management handler: {e}");
                                    }
                                    Err(n.err().or(f.err()).or(fm.err()).unwrap())
                                }
                            }
                        }
                    },
                };

                match outcome {
                    Ok(()) => {
                        (raw.ack)();
                        tracing::info!(event_type, "event handled ok");
                    }
                    Err(e) => {
                        if raw.delivery_count >= CONSUMER_MAX_DELIVER as u64 {
                            // Rebuild payload from raw bytes for DLQ storage.
                            let payload_val =
                                serde_json::from_slice::<serde_json::Value>(&raw.payload)
                                    .unwrap_or(serde_json::Value::Null);
                            if let Err(dlq_err) = infra
                                .dlq_store
                                .insert(event_type, &payload_val, &e.to_string())
                                .await
                            {
                                tracing::error!("DLQ insert failed: {dlq_err} — message lost");
                            } else {
                                tracing::warn!(
                                    event_type,
                                    delivery_count = raw.delivery_count,
                                    "event exhausted — moved to DLQ"
                                );
                            }
                            (raw.ack)();
                        } else {
                            (raw.nack)();
                        }
                    }
                }
            }
        }
    }
}
