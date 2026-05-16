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
