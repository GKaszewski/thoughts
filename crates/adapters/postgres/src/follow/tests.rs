use super::*;
use crate::test_helpers::seed_user;
use chrono::Utc;

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

#[sqlx::test(migrations = "./migrations")]
async fn list_mutual_returns_only_mutual_accepted_follows(pool: sqlx::PgPool) {
    let alice = seed_user(&pool, "alice", "alice@ex.com").await;
    let bob = seed_user(&pool, "bob", "bob@ex.com").await;
    let carol = seed_user(&pool, "carol", "carol@ex.com").await;
    let repo = PgFollowRepository::new(pool);
    let page = domain::models::feed::PageParams {
        page: 1,
        per_page: 20,
    };

    // alice → bob (accepted), bob → alice (accepted) = friends
    repo.save(&Follow {
        follower_id: alice.id.clone(),
        following_id: bob.id.clone(),
        state: FollowState::Accepted,
        ap_id: None,
        created_at: Utc::now(),
    })
    .await
    .unwrap();
    repo.save(&Follow {
        follower_id: bob.id.clone(),
        following_id: alice.id.clone(),
        state: FollowState::Accepted,
        ap_id: None,
        created_at: Utc::now(),
    })
    .await
    .unwrap();

    // alice → carol (accepted), carol does NOT follow back = not a friend
    repo.save(&Follow {
        follower_id: alice.id.clone(),
        following_id: carol.id.clone(),
        state: FollowState::Accepted,
        ap_id: None,
        created_at: Utc::now(),
    })
    .await
    .unwrap();

    let result = repo.list_mutual(&alice.id, &page).await.unwrap();
    assert_eq!(result.total, 1);
    assert_eq!(result.items.len(), 1);
    assert_eq!(result.items[0].id, bob.id);
}

#[sqlx::test(migrations = "./migrations")]
async fn list_mutual_excludes_pending_follows(pool: sqlx::PgPool) {
    let alice = seed_user(&pool, "alice", "alice@ex.com").await;
    let bob = seed_user(&pool, "bob", "bob@ex.com").await;
    let repo = PgFollowRepository::new(pool);
    let page = domain::models::feed::PageParams {
        page: 1,
        per_page: 20,
    };

    // alice → bob (accepted), bob → alice (PENDING) = NOT a friend
    repo.save(&Follow {
        follower_id: alice.id.clone(),
        following_id: bob.id.clone(),
        state: FollowState::Accepted,
        ap_id: None,
        created_at: Utc::now(),
    })
    .await
    .unwrap();
    repo.save(&Follow {
        follower_id: bob.id.clone(),
        following_id: alice.id.clone(),
        state: FollowState::Pending,
        ap_id: None,
        created_at: Utc::now(),
    })
    .await
    .unwrap();

    let result = repo.list_mutual(&alice.id, &page).await.unwrap();
    assert_eq!(result.total, 0);
    assert!(result.items.is_empty());
}
