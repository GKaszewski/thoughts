use super::*;
use domain::ports::AuthService;

#[test]
fn generate_and_validate_token() {
    let svc = JwtAuthService::new("a-secret-that-is-at-least-32-bytes!!".into(), 3600);
    let id = UserId::new();
    let tok = svc.generate_token(&id).unwrap();
    let parsed = svc.validate_token(&tok.token).unwrap();
    assert_eq!(parsed.as_uuid(), id.as_uuid());
}

#[test]
fn invalid_token_returns_unauthorized() {
    let svc = JwtAuthService::new("a-secret-that-is-at-least-32-bytes!!".into(), 3600);
    let err = svc.validate_token("not.a.token").unwrap_err();
    assert!(matches!(err, DomainError::Unauthorized));
}

#[tokio::test]
async fn hash_and_verify() {
    let hasher = Argon2PasswordHasher;
    let hash = hasher.hash("mypassword").await.unwrap();
    assert!(hasher.verify("mypassword", &hash).await.unwrap());
    assert!(!hasher.verify("wrongpassword", &hash).await.unwrap());
}
