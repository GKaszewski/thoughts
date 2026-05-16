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
