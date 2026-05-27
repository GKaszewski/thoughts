use super::*;

#[test]
fn extract_extensions_picks_up_non_standard_fields() {
    let obj = serde_json::json!({
        "type": "Note",
        "id": "https://example.com/notes/1",
        "content": "hello",
        "published": "2025-01-01T00:00:00Z",
        "movieTitle": "Dune",
        "rating": 5,
        "posterUrl": "https://example.com/poster.jpg"
    });
    let ext = extract_extensions(&obj).unwrap();
    assert_eq!(ext["movieTitle"], "Dune");
    assert_eq!(ext["rating"], 5);
    assert_eq!(ext["posterUrl"], "https://example.com/poster.jpg");
    assert!(ext.get("type").is_none());
    assert!(ext.get("content").is_none());
    assert!(ext.get("id").is_none());
}

#[test]
fn extract_extensions_returns_none_for_standard_only_note() {
    let obj = serde_json::json!({
        "type": "Note",
        "content": "hello",
        "published": "2025-01-01T00:00:00Z",
        "to": ["https://www.w3.org/ns/activitystreams#Public"],
        "tag": []
    });
    assert!(extract_extensions(&obj).is_none());
}

#[test]
fn extract_extensions_returns_none_for_non_object() {
    let obj = serde_json::json!("not an object");
    assert!(extract_extensions(&obj).is_none());
}

#[test]
fn try_from_ap_returns_none_for_person() {
    let person = serde_json::json!({ "type": "Person", "id": "https://example.com/users/1" });
    assert!(ThoughtNote::try_from_ap(person).is_none());
}

#[test]
fn try_from_ap_returns_none_for_missing_type() {
    let obj = serde_json::json!({ "content": "hello" });
    assert!(ThoughtNote::try_from_ap(obj).is_none());
}

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
