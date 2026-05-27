use super::*;
use async_trait::async_trait;
use domain::{
    errors::DomainError,
    events::DomainEvent,
    models::{
        feed::{PageParams, Paginated, UserSummary},
        user::{UpdateProfileInput, User},
    },
    ports::{AuthService, GeneratedToken, PasswordHasher, UserReader, UserWriter},
    testing::{NoOpEventPublisher, TestStore},
    value_objects::{Email, PasswordHash, UserId, Username},
};

/// Simulates a concurrent registration that slips past the pre-checks and
/// hits the DB unique constraint — exactly what happens in the TOCTOU window.
struct ConflictOnSaveStore(TestStore);
struct EmailConflictOnSaveStore(TestStore);

#[async_trait]
impl UserReader for ConflictOnSaveStore {
    async fn find_by_id(&self, id: &UserId) -> Result<Option<User>, DomainError> {
        self.0.find_by_id(id).await
    }
    async fn find_by_username(&self, username: &Username) -> Result<Option<User>, DomainError> {
        self.0.find_by_username(username).await
    }
    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, DomainError> {
        self.0.find_by_email(email).await
    }
    async fn list_with_stats(&self) -> Result<Vec<UserSummary>, DomainError> {
        self.0.list_with_stats().await
    }
    async fn count(&self) -> Result<i64, DomainError> {
        self.0.count().await
    }
    async fn list_paginated(
        &self,
        page: PageParams,
    ) -> Result<Paginated<UserSummary>, DomainError> {
        self.0.list_paginated(page).await
    }
    async fn find_by_ids(
        &self,
        ids: &[UserId],
    ) -> Result<std::collections::HashMap<UserId, User>, DomainError> {
        self.0.find_by_ids(ids).await
    }
}

#[async_trait]
impl UserWriter for ConflictOnSaveStore {
    async fn save(&self, _user: &User) -> Result<(), DomainError> {
        Err(DomainError::UniqueViolation { field: "username" })
    }
    async fn update_profile(
        &self,
        user_id: &UserId,
        input: UpdateProfileInput,
    ) -> Result<(), DomainError> {
        self.0.update_profile(user_id, input).await
    }
    async fn set_also_known_as(
        &self,
        user_id: &UserId,
        value: Option<String>,
    ) -> Result<(), DomainError> {
        self.0.set_also_known_as(user_id, value).await
    }
}

#[async_trait]
impl UserReader for EmailConflictOnSaveStore {
    async fn find_by_id(&self, id: &UserId) -> Result<Option<User>, DomainError> {
        self.0.find_by_id(id).await
    }
    async fn find_by_username(&self, username: &Username) -> Result<Option<User>, DomainError> {
        self.0.find_by_username(username).await
    }
    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, DomainError> {
        self.0.find_by_email(email).await
    }
    async fn list_with_stats(&self) -> Result<Vec<UserSummary>, DomainError> {
        self.0.list_with_stats().await
    }
    async fn count(&self) -> Result<i64, DomainError> {
        self.0.count().await
    }
    async fn list_paginated(
        &self,
        page: PageParams,
    ) -> Result<Paginated<UserSummary>, DomainError> {
        self.0.list_paginated(page).await
    }
    async fn find_by_ids(
        &self,
        ids: &[UserId],
    ) -> Result<std::collections::HashMap<UserId, User>, DomainError> {
        self.0.find_by_ids(ids).await
    }
}

#[async_trait]
impl UserWriter for EmailConflictOnSaveStore {
    async fn save(&self, _user: &User) -> Result<(), DomainError> {
        Err(DomainError::UniqueViolation { field: "email" })
    }
    async fn update_profile(
        &self,
        user_id: &UserId,
        input: UpdateProfileInput,
    ) -> Result<(), DomainError> {
        self.0.update_profile(user_id, input).await
    }
    async fn set_also_known_as(
        &self,
        user_id: &UserId,
        value: Option<String>,
    ) -> Result<(), DomainError> {
        self.0.set_also_known_as(user_id, value).await
    }
}

struct FakeHasher;
#[async_trait]
impl PasswordHasher for FakeHasher {
    async fn hash(&self, plain: &str) -> Result<PasswordHash, DomainError> {
        Ok(PasswordHash(plain.to_string()))
    }
    async fn verify(&self, plain: &str, hash: &PasswordHash) -> Result<bool, DomainError> {
        Ok(plain == hash.0)
    }
}

struct FakeAuth;
impl AuthService for FakeAuth {
    fn generate_token(&self, uid: &UserId) -> Result<GeneratedToken, DomainError> {
        Ok(GeneratedToken {
            token: uid.to_string(),
            user_id: uid.clone(),
        })
    }
    fn validate_token(&self, token: &str) -> Result<UserId, DomainError> {
        Ok(UserId::from_uuid(
            uuid::Uuid::parse_str(token).map_err(|_| DomainError::Unauthorized)?,
        ))
    }
}

fn input() -> RegisterInput {
    RegisterInput {
        username: "alice".into(),
        email: "alice@ex.com".into(),
        password: "pw".into(),
    }
}

#[tokio::test]
async fn register_creates_user() {
    let store = TestStore::default();
    let out = register(&store, &FakeHasher, &FakeAuth, &NoOpEventPublisher, input())
        .await
        .unwrap();
    assert_eq!(out.user.username.as_str(), "alice");
    assert!(!out.token.is_empty());
}

#[tokio::test]
async fn register_rejects_duplicate_username() {
    let store = TestStore::default();
    register(&store, &FakeHasher, &FakeAuth, &NoOpEventPublisher, input())
        .await
        .unwrap();
    let err = register(&store, &FakeHasher, &FakeAuth, &NoOpEventPublisher, input())
        .await
        .unwrap_err();
    assert!(matches!(err, DomainError::Conflict(_)));
}

#[tokio::test]
async fn login_succeeds_with_correct_password() {
    let store = TestStore::default();
    register(&store, &FakeHasher, &FakeAuth, &NoOpEventPublisher, input())
        .await
        .unwrap();
    let out = login(
        &store,
        &FakeHasher,
        &FakeAuth,
        LoginInput {
            email: "alice@ex.com".into(),
            password: "pw".into(),
        },
    )
    .await
    .unwrap();
    assert!(!out.token.is_empty());
}

#[tokio::test]
async fn login_fails_wrong_password() {
    let store = TestStore::default();
    register(&store, &FakeHasher, &FakeAuth, &NoOpEventPublisher, input())
        .await
        .unwrap();
    let err = login(
        &store,
        &FakeHasher,
        &FakeAuth,
        LoginInput {
            email: "alice@ex.com".into(),
            password: "wrong".into(),
        },
    )
    .await
    .unwrap_err();
    assert!(matches!(err, DomainError::Unauthorized));
}

#[tokio::test]
async fn register_publishes_user_registered_event() {
    let store = TestStore::default();
    register(&store, &FakeHasher, &FakeAuth, &store, input())
        .await
        .unwrap();
    let events = store.events.lock().unwrap();
    assert_eq!(events.len(), 1);
    assert!(matches!(events[0], DomainEvent::UserRegistered { .. }));
}

#[tokio::test]
async fn login_fails_for_nonexistent_user() {
    let store = TestStore::default();
    let err = login(
        &store,
        &FakeHasher,
        &FakeAuth,
        LoginInput {
            email: "ghost@ex.com".into(),
            password: "pass".into(),
        },
    )
    .await
    .unwrap_err();
    assert!(matches!(err, DomainError::Unauthorized));
}

#[tokio::test]
async fn register_rejects_duplicate_email() {
    let store = TestStore::default();
    register(&store, &FakeHasher, &FakeAuth, &NoOpEventPublisher, input())
        .await
        .unwrap();
    let err = register(
        &store,
        &FakeHasher,
        &FakeAuth,
        &NoOpEventPublisher,
        RegisterInput {
            username: "alice2".into(),
            email: "alice@ex.com".into(),
            password: "pass2".into(),
        },
    )
    .await
    .unwrap_err();
    assert!(matches!(err, DomainError::Conflict(_)));
}

/// TOCTOU: a concurrent registration slips past the pre-checks and the DB
/// unique constraint fires on save. The map_err must convert it to a
/// human-readable Conflict, not bubble up a raw constraint name.
#[tokio::test]
async fn register_maps_db_conflict_on_username_to_conflict() {
    let store = ConflictOnSaveStore(TestStore::default());
    let err = register(&store, &FakeHasher, &FakeAuth, &NoOpEventPublisher, input())
        .await
        .unwrap_err();
    assert!(
        matches!(err, DomainError::Conflict(ref m) if m == "username taken"),
        "expected 'username taken', got: {:?}",
        err
    );
}

#[tokio::test]
async fn register_maps_db_conflict_on_email_to_conflict() {
    let store = EmailConflictOnSaveStore(TestStore::default());
    let err = register(&store, &FakeHasher, &FakeAuth, &NoOpEventPublisher, input())
        .await
        .unwrap_err();
    assert!(
        matches!(err, DomainError::Conflict(ref m) if m == "email taken"),
        "expected 'email taken', got: {:?}",
        err
    );
}
