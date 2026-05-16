use async_trait::async_trait;
use domain::{
    errors::DomainError,
    events::{DomainEvent, EventEnvelope},
    ports::{EventConsumer, EventPublisher},
};
use event_payload::EventPayload;
use futures::stream::BoxStream;

/// Abstraction over any pub/sub transport backend.
/// Implement this for NATS, Kafka, Redis Streams, etc.
/// The adapter calls `publish_bytes(subject, bytes)` — subjects come from `EventPayload::subject()`.
#[async_trait]
pub trait Transport: Send + Sync {
    async fn publish_bytes(&self, subject: &str, bytes: &[u8]) -> Result<(), DomainError>;
}

/// Routes domain events to a transport backend.
///
/// Converts: `DomainEvent` → `EventPayload` → JSON bytes → `transport.publish_bytes(subject, bytes)`
///
/// To swap transports (e.g. NATS → Kafka), replace the `T` at the composition root.
pub struct EventPublisherAdapter<T: Transport> {
    transport: T,
}

impl<T: Transport> EventPublisherAdapter<T> {
    pub fn new(transport: T) -> Self {
        Self { transport }
    }
}

#[async_trait]
impl<T: Transport> EventPublisher for EventPublisherAdapter<T> {
    async fn publish(&self, event: &DomainEvent) -> Result<(), DomainError> {
        let payload = EventPayload::from(event);
        let subject = payload.subject();
        let bytes =
            serde_json::to_vec(&payload).map_err(|e| DomainError::Internal(e.to_string()))?;
        tracing::debug!(subject, "publishing event");
        self.transport.publish_bytes(subject, &bytes).await
    }
}

/// A raw inbound message from a transport backend.
/// `ack` and `nack` are transport-level acknowledgements (e.g. Kafka offset commit).
/// For at-most-once transports (basic NATS), both are no-ops.
pub struct RawMessage {
    pub subject: String,
    pub payload: Vec<u8>,
    pub delivery_count: u64,
    pub ack: Box<dyn Fn() + Send + Sync>,
    pub nack: Box<dyn Fn() + Send + Sync>,
}

/// Abstraction over any subscribe/consume backend.
pub trait MessageSource: Send + Sync {
    fn messages(&self) -> BoxStream<'_, Result<RawMessage, DomainError>>;
}

/// Deserializes raw transport messages into domain `EventEnvelope`s.
/// Invalid or unknown messages are skipped with a warning — stream continues.
pub struct EventConsumerAdapter<S: MessageSource> {
    source: S,
}

impl<S: MessageSource> EventConsumerAdapter<S> {
    pub fn new(source: S) -> Self {
        Self { source }
    }
}

impl<S: MessageSource> EventConsumer for EventConsumerAdapter<S> {
    fn consume(&self) -> BoxStream<'_, Result<EventEnvelope, DomainError>> {
        use futures::StreamExt;
        let stream = self.source.messages();
        Box::pin(stream.filter_map(|result| async move {
            match result {
                Err(e) => {
                    tracing::warn!("transport error: {e}");
                    None
                }
                Ok(msg) => {
                    let payload = match serde_json::from_slice::<EventPayload>(&msg.payload) {
                        Ok(p) => p,
                        Err(e) => {
                            tracing::warn!("failed to deserialize event payload — acking to prevent orphan: {e}");
                            (msg.ack)();
                            return None;
                        }
                    };
                    let event = match DomainEvent::try_from(payload) {
                        Ok(e) => e,
                        Err(e) => {
                            tracing::warn!("unknown or malformed event type — acking to prevent orphan: {e}");
                            (msg.ack)();
                            return None;
                        }
                    };
                    Some(Ok(EventEnvelope {
                        event,
                        delivery_count: msg.delivery_count,
                        ack: msg.ack,
                        nack: msg.nack,
                    }))
                }
            }
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use domain::value_objects::{ThoughtId, UserId};
    use std::sync::{Arc, Mutex};

    struct SpyTransport {
        calls: Arc<Mutex<Vec<(String, Vec<u8>)>>>,
    }
    impl SpyTransport {
        fn new() -> (Self, Arc<Mutex<Vec<(String, Vec<u8>)>>>) {
            let calls = Arc::new(Mutex::new(vec![]));
            (
                Self {
                    calls: calls.clone(),
                },
                calls,
            )
        }
    }
    #[async_trait]
    impl Transport for SpyTransport {
        async fn publish_bytes(&self, subject: &str, bytes: &[u8]) -> Result<(), DomainError> {
            self.calls
                .lock()
                .unwrap()
                .push((subject.to_string(), bytes.to_vec()));
            Ok(())
        }
    }

    #[tokio::test]
    async fn thought_created_routes_to_correct_subject() {
        let (spy, calls) = SpyTransport::new();
        let publisher = EventPublisherAdapter::new(spy);
        publisher
            .publish(&DomainEvent::ThoughtCreated {
                thought_id: ThoughtId::new(),
                user_id: UserId::new(),
                in_reply_to_id: None,
            })
            .await
            .unwrap();
        let calls = calls.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, "thoughts.created");
    }

    #[tokio::test]
    async fn serialized_payload_is_valid_json() {
        let (spy, calls) = SpyTransport::new();
        let publisher = EventPublisherAdapter::new(spy);
        publisher
            .publish(&DomainEvent::UserBlocked {
                blocker_id: UserId::new(),
                blocked_id: UserId::new(),
            })
            .await
            .unwrap();
        let bytes = calls.lock().unwrap()[0].1.clone();
        let json: serde_json::Value = serde_json::from_slice(&bytes).expect("valid JSON");
        assert_eq!(json["type"], "UserBlocked");
    }

    #[tokio::test]
    async fn consumer_adapter_deserializes_and_yields_event() {
        use domain::value_objects::ThoughtId;
        use futures::StreamExt;

        let event = DomainEvent::ThoughtCreated {
            thought_id: ThoughtId::new(),
            user_id: UserId::new(),
            in_reply_to_id: None,
        };
        let payload = EventPayload::from(&event);
        let bytes = serde_json::to_vec(&payload).unwrap();

        struct OneMessageSource {
            bytes: Vec<u8>,
        }
        #[async_trait::async_trait]
        impl MessageSource for OneMessageSource {
            fn messages(&self) -> futures::stream::BoxStream<'_, Result<RawMessage, DomainError>> {
                let msg = RawMessage {
                    subject: "thoughts.created".to_string(),
                    payload: self.bytes.clone(),
                    delivery_count: 1,
                    ack: Box::new(|| {}),
                    nack: Box::new(|| {}),
                };
                Box::pin(futures::stream::once(async { Ok(msg) }))
            }
        }

        let adapter = EventConsumerAdapter::new(OneMessageSource { bytes });
        let mut stream = adapter.consume();
        let envelope = stream.next().await.unwrap().unwrap();
        assert!(matches!(envelope.event, DomainEvent::ThoughtCreated { .. }));
    }

    #[tokio::test]
    async fn consumer_adapter_skips_invalid_payloads() {
        use futures::StreamExt;

        struct BadMessageSource;
        #[async_trait::async_trait]
        impl MessageSource for BadMessageSource {
            fn messages(&self) -> futures::stream::BoxStream<'_, Result<RawMessage, DomainError>> {
                let msg = RawMessage {
                    subject: "bad".to_string(),
                    payload: b"not valid json".to_vec(),
                    delivery_count: 1,
                    ack: Box::new(|| {}),
                    nack: Box::new(|| {}),
                };
                Box::pin(futures::stream::once(async { Ok(msg) }))
            }
        }

        let adapter = EventConsumerAdapter::new(BadMessageSource);
        let mut stream = adapter.consume();
        assert!(stream.next().await.is_none());
    }
}
