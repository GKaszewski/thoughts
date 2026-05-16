use super::*;

#[test]
fn user_url_format() {
    let urls = ThoughtsUrls::new("https://example.com");
    assert_eq!(
        urls.user_url("alice").as_str(),
        "https://example.com/users/alice"
    );
}

#[test]
fn thought_url_format() {
    let urls = ThoughtsUrls::new("https://example.com");
    let id = uuid::Uuid::nil();
    assert!(urls
        .thought_url(id)
        .as_str()
        .starts_with("https://example.com/thoughts/"));
}
