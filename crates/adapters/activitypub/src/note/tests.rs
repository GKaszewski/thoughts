use super::*;

#[test]
fn note_serializes_with_public_audience() {
    let note = ThoughtNote::new_public(super::ThoughtNoteInput {
        id: "https://example.com/thoughts/1".parse().unwrap(),
        actor_url: "https://example.com/users/alice".parse().unwrap(),
        content: "Hello world".to_string(),
        published: chrono::Utc::now(),
        in_reply_to: None,
        sensitive: false,
        summary: None,
        followers_url: "https://example.com/users/alice/followers".parse().unwrap(),
    });
    let json = serde_json::to_string(&note).unwrap();
    assert!(json.contains(AS_PUBLIC));
    assert!(json.contains("Hello world"));
    assert!(json.contains("\"url\""));
}
