use super::*;
use domain::{
    models::{
        thought::{NewThought, Thought, Visibility},
        user::User,
    },
    ports::{SearchPort, ThoughtRepository, UserWriter},
    value_objects::{Content, Email, PasswordHash, ThoughtId, UserId, Username},
};

async fn seed_thought(pool: &sqlx::PgPool, username: &str, content: &str) -> (User, Thought) {
    use postgres::{thought::PgThoughtRepository, user::PgUserRepository};
    let urepo = PgUserRepository::new(pool.clone());
    let trepo = PgThoughtRepository::new(pool.clone());
    let u = User::new_local(
        UserId::new(),
        Username::new(username).unwrap(),
        Email::new(format!("{username}@ex.com")).unwrap(),
        PasswordHash("h".into()),
    );
    urepo.save(&u).await.unwrap();
    let t = Thought::new_local(NewThought {
        id: ThoughtId::new(),
        user_id: u.id.clone(),
        content: Content::new_local(content).unwrap(),
        in_reply_to_id: None,
        visibility: Visibility::Public,
        content_warning: None,
        sensitive: false,
    });
    trepo.save(&t).await.unwrap();
    (u, t)
}

#[sqlx::test(migrations = "../postgres/migrations")]
async fn search_thoughts_finds_by_keyword(pool: sqlx::PgPool) {
    seed_thought(&pool, "alice", "hello world").await;
    seed_thought(&pool, "bob", "goodbye universe").await;
    let repo = PgSearchRepository::new(pool);
    let result = repo
        .search_thoughts(
            "hello world",
            &PageParams {
                page: 1,
                per_page: 20,
            },
            None,
        )
        .await
        .unwrap();
    assert_eq!(result.total, 1);
    assert_eq!(result.items[0].thought.content.as_str(), "hello world");
}

#[sqlx::test(migrations = "../postgres/migrations")]
async fn search_users_finds_by_username(pool: sqlx::PgPool) {
    use postgres::user::PgUserRepository;
    let urepo = PgUserRepository::new(pool.clone());
    let alice = User::new_local(
        UserId::new(),
        Username::new("alice_search").unwrap(),
        Email::new("alice@ex.com").unwrap(),
        PasswordHash("h".into()),
    );
    urepo.save(&alice).await.unwrap();
    let repo = PgSearchRepository::new(pool);
    let result = repo
        .search_users(
            "alice",
            &PageParams {
                page: 1,
                per_page: 20,
            },
        )
        .await
        .unwrap();
    assert!(!result.items.is_empty());
    assert!(result
        .items
        .iter()
        .any(|u| u.username.as_str() == "alice_search"));
}

#[sqlx::test(migrations = "../postgres/migrations")]
async fn search_thoughts_returns_empty_for_no_match(pool: sqlx::PgPool) {
    seed_thought(&pool, "alice", "hello world").await;
    let repo = PgSearchRepository::new(pool);
    let result = repo
        .search_thoughts(
            "zzzzzzzzz",
            &PageParams {
                page: 1,
                per_page: 20,
            },
            None,
        )
        .await
        .unwrap();
    assert_eq!(result.total, 0);
}

#[sqlx::test(migrations = "../postgres/migrations")]
async fn search_thoughts_viewer_context(pool: sqlx::PgPool) {
    use domain::models::social::Like;
    use domain::ports::LikeRepository;
    use domain::value_objects::LikeId;
    use postgres::like::PgLikeRepository;

    let (alice, thought) = seed_thought(&pool, "alice", "hello world").await;

    // alice likes her own thought
    let like_repo = PgLikeRepository::new(pool.clone());
    like_repo
        .save(&Like {
            id: LikeId::new(),
            user_id: alice.id.clone(),
            thought_id: thought.id.clone(),
            ap_id: None,
            created_at: chrono::Utc::now(),
        })
        .await
        .unwrap();

    let repo = PgSearchRepository::new(pool);

    // with viewer — should see liked = true
    let authed = repo
        .search_thoughts(
            "hello",
            &PageParams {
                page: 1,
                per_page: 20,
            },
            Some(&alice.id),
        )
        .await
        .unwrap();
    assert_eq!(authed.items.len(), 1);
    let ctx = authed.items[0]
        .viewer
        .as_ref()
        .expect("viewer context present");
    assert!(ctx.liked, "alice should see the thought as liked");
    assert!(!ctx.boosted);

    // without viewer — viewer should be None
    let anon = repo
        .search_thoughts(
            "hello",
            &PageParams {
                page: 1,
                per_page: 20,
            },
            None,
        )
        .await
        .unwrap();
    assert_eq!(anon.items.len(), 1);
    assert!(
        anon.items[0].viewer.is_none(),
        "anonymous request has no viewer context"
    );
}
