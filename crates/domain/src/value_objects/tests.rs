use super::*;

#[test]
fn username_rejects_empty() {
    assert!(Username::new("").is_err());
}
#[test]
fn username_rejects_too_long() {
    assert!(Username::new("a".repeat(33)).is_err());
}
#[test]
fn username_rejects_invalid_chars() {
    assert!(Username::new("hello world").is_err());
}
#[test]
fn username_accepts_valid() {
    assert!(Username::new("hello_123").is_ok());
}
#[test]
fn content_local_rejects_over_128() {
    assert!(Content::new_local("a".repeat(129)).is_err());
}
#[test]
fn content_local_accepts_128() {
    assert!(Content::new_local("a".repeat(128)).is_ok());
}
#[test]
fn email_rejects_no_at() {
    assert!(Email::new("notanemail").is_err());
}
