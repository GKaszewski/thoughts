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
mod tests;
