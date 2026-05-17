use super::*;
use crate::test_helpers::seed_user;
use chrono::Utc;
use domain::value_objects::*;

#[sqlx::test(migrations = "./migrations")]
async fn block_exists(pool: sqlx::PgPool) {
    let alice = seed_user(&pool, "alice", "alice@ex.com").await;
    let bob = seed_user(&pool, "bob", "bob@ex.com").await;
    let repo = PgBlockRepository::new(pool);
    let block = Block {
        blocker_id: alice.id.clone(),
        blocked_id: bob.id.clone(),
        created_at: Utc::now(),
    };
    repo.save(&block).await.unwrap();
    assert!(repo.exists(&alice.id, &bob.id).await.unwrap());
    assert!(!repo.exists(&bob.id, &alice.id).await.unwrap());
}

#[sqlx::test(migrations = "./migrations")]
async fn unblock(pool: sqlx::PgPool) {
    let alice = seed_user(&pool, "alice", "alice@ex.com").await;
    let bob = seed_user(&pool, "bob", "bob@ex.com").await;
    let repo = PgBlockRepository::new(pool);
    let block = Block {
        blocker_id: alice.id.clone(),
        blocked_id: bob.id.clone(),
        created_at: Utc::now(),
    };
    repo.save(&block).await.unwrap();
    repo.delete(&alice.id, &bob.id).await.unwrap();
    assert!(!repo.exists(&alice.id, &bob.id).await.unwrap());
}
