use super::*;

#[test]
fn note_serializes_with_public_audience() {
    let note = ThoughtNote::new_public(
        "https://example.com/thoughts/1".parse().unwrap(),
        "https://example.com/users/alice".parse().unwrap(),
        "Hello world".to_string(),
        chrono::Utc::now(),
        None,
        false,
        None,
        "https://example.com/users/alice/followers".parse().unwrap(),
    );
    let json = serde_json::to_string(&note).unwrap();
    assert!(json.contains(AS_PUBLIC));
    assert!(json.contains("Hello world"));
    assert!(json.contains("\"url\""));
}
