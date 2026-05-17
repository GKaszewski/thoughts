
use super::*;
use crate::user::PgUserRepository;
use chrono::Utc;
use domain::ports::UserWriter;
use domain::{models::user::User, value_objects::*};

async fn seed_user(pool: &sqlx::PgPool) -> User {
    let repo = PgUserRepository::new(pool.clone());
    let u = User::new_local(
        UserId::new(),
        Username::new("alice").unwrap(),
        Email::new("alice@ex.com").unwrap(),
        PasswordHash("h".into()),
    );
    repo.save(&u).await.unwrap();
    u
}

#[sqlx::test(migrations = "./migrations")]
async fn save_and_find_by_hash(pool: sqlx::PgPool) {
    let user = seed_user(&pool).await;
    let repo = PgApiKeyRepository::new(pool);
    let key = ApiKey {
        id: ApiKeyId::new(),
        user_id: user.id.clone(),
        key_hash: "abc123".into(),
        name: "test".into(),
        created_at: Utc::now(),
    };
    repo.save(&key).await.unwrap();
    let found = repo.find_by_hash("abc123").await.unwrap().unwrap();
    assert_eq!(found.name, "test");
}

#[sqlx::test(migrations = "./migrations")]
async fn delete_key(pool: sqlx::PgPool) {
    let user = seed_user(&pool).await;
    let repo = PgApiKeyRepository::new(pool);
    let key = ApiKey {
        id: ApiKeyId::new(),
        user_id: user.id.clone(),
        key_hash: "def456".into(),
        name: "key2".into(),
        created_at: Utc::now(),
    };
    repo.save(&key).await.unwrap();
    repo.delete(&key.id, &user.id).await.unwrap();
    assert!(repo.find_by_hash("def456").await.unwrap().is_none());
}
