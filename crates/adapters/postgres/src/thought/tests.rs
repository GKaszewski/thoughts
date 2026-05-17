
use super::*;
use crate::test_helpers::seed_user;
use domain::{
    models::thought::{Thought, Visibility},
    value_objects::*,
};

#[sqlx::test(migrations = "./migrations")]
async fn save_and_find_thought(pool: sqlx::PgPool) {
    let user = seed_user(&pool, "alice", "alice@ex.com").await;
    let repo = PgThoughtRepository::new(pool);
    let t = Thought::new_local(
        ThoughtId::new(),
        user.id.clone(),
        Content::new_local("hello world").unwrap(),
        None,
        Visibility::Public,
        None,
        false,
    );
    repo.save(&t).await.unwrap();
    let found = repo.find_by_id(&t.id).await.unwrap().unwrap();
    assert_eq!(found.content.as_str(), "hello world");
    assert!(found.local);
}

#[sqlx::test(migrations = "./migrations")]
async fn delete_thought(pool: sqlx::PgPool) {
    let user = seed_user(&pool, "bob", "bob@ex.com").await;
    let repo = PgThoughtRepository::new(pool);
    let t = Thought::new_local(
        ThoughtId::new(),
        user.id.clone(),
        Content::new_local("bye").unwrap(),
        None,
        Visibility::Public,
        None,
        false,
    );
    repo.save(&t).await.unwrap();
    repo.delete(&t.id, &user.id).await.unwrap();
    assert!(repo.find_by_id(&t.id).await.unwrap().is_none());
}

#[sqlx::test(migrations = "./migrations")]
async fn delete_wrong_owner_returns_not_found(pool: sqlx::PgPool) {
    let alice = seed_user(&pool, "alice", "alice@ex.com").await;
    let bob = seed_user(&pool, "bob", "bob@ex.com").await;
    let repo = PgThoughtRepository::new(pool);
    let t = Thought::new_local(
        ThoughtId::new(),
        alice.id.clone(),
        Content::new_local("secret").unwrap(),
        None,
        Visibility::Public,
        None,
        false,
    );
    repo.save(&t).await.unwrap();
    let err = repo.delete(&t.id, &bob.id).await.unwrap_err();
    assert!(matches!(err, DomainError::NotFound));
}

#[sqlx::test(migrations = "./migrations")]
async fn get_thread_returns_root_and_replies(pool: sqlx::PgPool) {
    let user = seed_user(&pool, "charlie", "charlie@ex.com").await;
    let repo = PgThoughtRepository::new(pool);
    let root = Thought::new_local(
        ThoughtId::new(),
        user.id.clone(),
        Content::new_local("root").unwrap(),
        None,
        Visibility::Public,
        None,
        false,
    );
    let reply = Thought::new_local(
        ThoughtId::new(),
        user.id.clone(),
        Content::new_local("reply").unwrap(),
        Some(root.id.clone()),
        Visibility::Public,
        None,
        false,
    );
    repo.save(&root).await.unwrap();
    repo.save(&reply).await.unwrap();
    let thread = repo.get_thread(&root.id).await.unwrap();
    assert_eq!(thread.len(), 2);
}
