use async_nats::jetstream::{self, stream::Config as StreamConfig, AckKind};
use async_trait::async_trait;
use domain::errors::DomainError;
use event_transport::{MessageSource, RawMessage, Transport};
use futures::stream::BoxStream;
use std::sync::Arc;

const STREAM_NAME: &str = "THOUGHTS_EVENTS";
const STREAM_SUBJECT: &str = "thoughts-events.>";
const CONSUMER_NAME: &str = "worker";
const MAX_MESSAGES: i64 = 100_000;

/// Maximum NATS delivery attempts before a message is considered exhausted.
pub const CONSUMER_MAX_DELIVER: i64 = 5;
/// How long NATS waits for an ack before redelivering.
const CONSUMER_ACK_WAIT_SECS: u64 = 30;
/// Timeout for spawned ack/nack async tasks.
const ACK_TASK_TIMEOUT_SECS: u64 = 5;

fn stream_config() -> StreamConfig {
    StreamConfig {
        name: STREAM_NAME.to_string(),
        subjects: vec![STREAM_SUBJECT.to_string()],
        max_messages: MAX_MESSAGES,
        ..Default::default()
    }
}

/// Ensure the JetStream stream exists with the current config.
/// If an incompatible stream exists (e.g. wrong retention policy), deletes and recreates it.
pub async fn ensure_stream(client: &async_nats::Client) -> Result<(), DomainError> {
    let js = jetstream::new(client.clone());

    // Happy path: stream exists and config is compatible.
    if js.update_stream(stream_config()).await.is_ok() {
        tracing::info!(subject = STREAM_SUBJECT, "JetStream stream updated");
        return Ok(());
    }

    // Update failed — retention policy mismatch or other incompatibility.
    // Delete the old stream and recreate with current config.
    tracing::warn!(
        "JetStream stream update failed (incompatible config), deleting '{STREAM_NAME}' and recreating"
    );
    let _ = js.delete_stream(STREAM_NAME).await;

    js.create_stream(stream_config())
        .await
        .map(|_| ())
        .map_err(|e| DomainError::Internal(format!("JetStream stream create failed: {e}")))
}

// ── NatsTransport — JetStream publish ──────────────────────────────────────

pub struct NatsTransport {
    jetstream: jetstream::Context,
}

impl NatsTransport {
    pub fn new(client: async_nats::Client) -> Self {
        Self {
            jetstream: jetstream::new(client),
        }
    }
}

#[async_trait]
impl Transport for NatsTransport {
    async fn publish_bytes(&self, subject: &str, bytes: &[u8]) -> Result<(), DomainError> {
        // Prefix all subjects so they land inside the stream's subject filter.
        let full_subject = format!("thoughts-events.{subject}");
        self.jetstream
            .publish(full_subject, bytes.to_vec().into())
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?
            .await // wait for server ack — confirms message is durably stored
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        Ok(())
    }
}

// ── NatsMessageSource — JetStream durable push consumer ────────────────────

pub struct NatsMessageSource {
    jetstream: jetstream::Context,
}

impl NatsMessageSource {
    pub fn new(client: async_nats::Client) -> Self {
        Self {
            jetstream: jetstream::new(client),
        }
    }
}

impl MessageSource for NatsMessageSource {
    fn messages(&self) -> BoxStream<'_, Result<RawMessage, DomainError>> {
        use futures::stream;
        use tokio::sync::{mpsc, Mutex as TokioMutex};

        let js = self.jetstream.clone();
        let (tx, rx) = mpsc::channel::<Result<RawMessage, DomainError>>(128);

        // Spawn the consumer loop in the background.
        // Pull consumer: worker explicitly fetches from NATS rather than NATS pushing.
        tokio::spawn(async move {
            let stream = match js.get_stream(STREAM_NAME).await {
                Ok(s) => s,
                Err(e) => {
                    let _ = tx.send(Err(DomainError::Internal(e.to_string()))).await;
                    return;
                }
            };

            // Delete any existing push consumer with this name — can't reuse as pull.
            // No-op if it doesn't exist or is already a pull consumer.
            if let Ok(info) = stream.consumer_info(CONSUMER_NAME).await {
                if info.config.deliver_subject.is_some() {
                    tracing::info!(
                        "deleting old push consumer '{CONSUMER_NAME}', replacing with pull"
                    );
                    let _ = stream.delete_consumer(CONSUMER_NAME).await;
                }
            }

            let consumer = match stream
                .get_or_create_consumer(
                    CONSUMER_NAME,
                    jetstream::consumer::pull::Config {
                        durable_name: Some(CONSUMER_NAME.to_string()),
                        deliver_policy: jetstream::consumer::DeliverPolicy::New,
                        ack_policy: jetstream::consumer::AckPolicy::Explicit,
                        ack_wait: std::time::Duration::from_secs(CONSUMER_ACK_WAIT_SECS),
                        max_deliver: CONSUMER_MAX_DELIVER,
                        // No filter_subject — consume everything from the stream.
                        // filter_subject matching the stream's own wildcard can be
                        // inconsistent across NATS server versions.
                        ..Default::default()
                    },
                )
                .await
            {
                Ok(c) => c,
                Err(e) => {
                    let _ = tx.send(Err(DomainError::Internal(e.to_string()))).await;
                    return;
                }
            };

            tracing::info!("NATS pull consumer ready");

            loop {
                // consumer.messages() uses long-poll (no no_wait flag) — NATS holds the
                // request open and delivers messages as they arrive.
                // fetch() in async-nats 0.48 defaults to no_wait:true which returns
                // immediately when the queue is empty, so we avoid it here.
                let mut messages = match consumer.messages().await {
                    Ok(m) => m,
                    Err(e) => {
                        tracing::error!("NATS messages() failed: {e}");
                        let _ = tx.send(Err(DomainError::Internal(e.to_string()))).await;
                        return;
                    }
                };

                use futures::StreamExt;
                while let Some(result) = messages.next().await {
                    let msg = match result {
                        Ok(m) => m,
                        Err(e) => {
                            tracing::warn!("NATS message error: {e}");
                            continue;
                        }
                    };

                    let subject = msg.subject.to_string();
                    let payload = msg.payload.to_vec();
                    let delivery_count = msg
                        .info()
                        .map(|info| info.delivered.max(0) as u64)
                        .unwrap_or(1);
                    let msg = Arc::new(msg);
                    let msg_nack = Arc::clone(&msg);

                    let raw = RawMessage {
                        subject,
                        payload,
                        delivery_count,
                        ack: Box::new(move || {
                            let m = Arc::clone(&msg);
                            tokio::spawn(async move {
                                let result = tokio::time::timeout(
                                    std::time::Duration::from_secs(ACK_TASK_TIMEOUT_SECS),
                                    m.ack(),
                                )
                                .await;
                                match result {
                                    Ok(Ok(())) => {}
                                    Ok(Err(e)) => tracing::warn!("NATS ack failed: {e}"),
                                    Err(_) => tracing::warn!(
                                        "NATS ack timed out after {ACK_TASK_TIMEOUT_SECS}s"
                                    ),
                                }
                            });
                        }),
                        nack: Box::new(move || {
                            let m = Arc::clone(&msg_nack);
                            tokio::spawn(async move {
                                let result = tokio::time::timeout(
                                    std::time::Duration::from_secs(ACK_TASK_TIMEOUT_SECS),
                                    m.ack_with(AckKind::Nak(None)),
                                )
                                .await;
                                match result {
                                    Ok(Ok(())) => {}
                                    Ok(Err(e)) => tracing::warn!("NATS nack failed: {e}"),
                                    Err(_) => tracing::warn!(
                                        "NATS nack timed out after {ACK_TASK_TIMEOUT_SECS}s"
                                    ),
                                }
                            });
                        }),
                    };

                    if tx.send(Ok(raw)).await.is_err() {
                        return; // receiver dropped — worker shutting down
                    }
                }
                // messages() stream ended (e.g. fetch timeout) — loop and restart
            }
        });

        // Bridge the channel receiver into a BoxStream.
        let rx = Arc::new(TokioMutex::new(rx));
        Box::pin(stream::unfold(rx, |rx| async move {
            let item = rx.lock().await.recv().await?;
            Some((item, rx))
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::{
        events::DomainEvent,
        value_objects::{LikeId, ThoughtId, UserId},
    };
    use event_payload::EventPayload;

    #[test]
    fn payload_from_domain_event_has_correct_subject() {
        let event = DomainEvent::ThoughtCreated {
            thought_id: ThoughtId::new(),
            user_id: UserId::new(),
            in_reply_to_id: None,
        };
        let payload = EventPayload::from(&event);
        assert_eq!(payload.subject(), "thoughts.created");
    }

    #[test]
    fn domain_event_roundtrip_via_payload() {
        let uid = UserId::new();
        let tid = ThoughtId::new();
        let event = DomainEvent::LikeAdded {
            like_id: LikeId::new(),
            user_id: uid.clone(),
            thought_id: tid.clone(),
        };
        let payload = EventPayload::from(&event);
        let back = DomainEvent::try_from(payload).unwrap();
        if let DomainEvent::LikeAdded {
            user_id,
            thought_id,
            ..
        } = back
        {
            assert_eq!(user_id, uid);
            assert_eq!(thought_id, tid);
        } else {
            panic!("wrong variant");
        }
    }
}
