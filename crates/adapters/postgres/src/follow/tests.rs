
use super::*;
use crate::test_helpers::seed_user;
use chrono::Utc;
use domain::value_objects::*;

#[sqlx::test(migrations = "./migrations")]
async fn save_and_find_follow(pool: sqlx::PgPool) {
    let alice = seed_user(&pool, "alice", "alice@ex.com").await;
    let bob = seed_user(&pool, "bob", "bob@ex.com").await;
    let repo = PgFollowRepository::new(pool);
    let follow = Follow {
        follower_id: alice.id.clone(),
        following_id: bob.id.clone(),
        state: FollowState::Accepted,
        ap_id: None,
        created_at: Utc::now(),
    };
    repo.save(&follow).await.unwrap();
    let found = repo.find(&alice.id, &bob.id).await.unwrap().unwrap();
    assert_eq!(found.state, FollowState::Accepted);
}

#[sqlx::test(migrations = "./migrations")]
async fn update_state(pool: sqlx::PgPool) {
    let alice = seed_user(&pool, "alice", "alice@ex.com").await;
    let bob = seed_user(&pool, "bob", "bob@ex.com").await;
    let repo = PgFollowRepository::new(pool);
    let follow = Follow {
        follower_id: alice.id.clone(),
        following_id: bob.id.clone(),
        state: FollowState::Pending,
        ap_id: None,
        created_at: Utc::now(),
    };
    repo.save(&follow).await.unwrap();
    repo.update_state(&alice.id, &bob.id, &FollowState::Accepted)
        .await
        .unwrap();
    let found = repo.find(&alice.id, &bob.id).await.unwrap().unwrap();
    assert_eq!(found.state, FollowState::Accepted);
}

#[sqlx::test(migrations = "./migrations")]
async fn get_accepted_following_ids(pool: sqlx::PgPool) {
    let alice = seed_user(&pool, "alice", "alice@ex.com").await;
    let bob = seed_user(&pool, "bob", "bob@ex.com").await;
    let repo = PgFollowRepository::new(pool);
    let follow = Follow {
        follower_id: alice.id.clone(),
        following_id: bob.id.clone(),
        state: FollowState::Accepted,
        ap_id: None,
        created_at: Utc::now(),
    };
    repo.save(&follow).await.unwrap();
    let ids = repo.get_accepted_following_ids(&alice.id).await.unwrap();
    assert_eq!(ids, vec![bob.id]);
}
