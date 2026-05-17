use super::*;
use domain::{models::user::User, value_objects::*};

#[sqlx::test(migrations = "./migrations")]
async fn save_and_find_by_id(pool: sqlx::PgPool) {
    let repo = PgUserRepository::new(pool);
    let user = User::new_local(
        UserId::new(),
        Username::new("alice").unwrap(),
        Email::new("alice@ex.com").unwrap(),
        PasswordHash("hash".into()),
    );
    repo.save(&user).await.unwrap();
    let found = repo.find_by_id(&user.id).await.unwrap().unwrap();
    assert_eq!(found.username.as_str(), "alice");
    assert_eq!(found.email.as_str(), "alice@ex.com");
}

#[sqlx::test(migrations = "./migrations")]
async fn find_by_username_returns_none_when_missing(pool: sqlx::PgPool) {
    let repo = PgUserRepository::new(pool);
    let result = repo
        .find_by_username(&Username::new("ghost").unwrap())
        .await
        .unwrap();
    assert!(result.is_none());
}

#[sqlx::test(migrations = "./migrations")]
async fn find_by_email(pool: sqlx::PgPool) {
    let repo = PgUserRepository::new(pool);
    let user = User::new_local(
        UserId::new(),
        Username::new("bob").unwrap(),
        Email::new("bob@ex.com").unwrap(),
        PasswordHash("hash".into()),
    );
    repo.save(&user).await.unwrap();
    let found = repo
        .find_by_email(&Email::new("bob@ex.com").unwrap())
        .await
        .unwrap();
    assert!(found.is_some());
}

#[sqlx::test(migrations = "./migrations")]
async fn update_profile_changes_fields(pool: sqlx::PgPool) {
    let repo = PgUserRepository::new(pool);
    let user = User::new_local(
        UserId::new(),
        Username::new("charlie").unwrap(),
        Email::new("charlie@ex.com").unwrap(),
        PasswordHash("hash".into()),
    );
    repo.save(&user).await.unwrap();
    repo.update_profile(
        &user.id,
        Some("Charlie".into()),
        Some("bio".into()),
        None,
        None,
        None,
    )
    .await
    .unwrap();
    let found = repo.find_by_id(&user.id).await.unwrap().unwrap();
    assert_eq!(found.display_name.as_deref(), Some("Charlie"));
    assert_eq!(found.bio.as_deref(), Some("bio"));
}
