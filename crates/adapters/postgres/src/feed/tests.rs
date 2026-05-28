use super::*;
use crate::{thought::PgThoughtRepository, user::PgUserRepository};
use domain::{
    models::{
        feed::PageParams,
        thought::{NewThought, Thought, Visibility},
        user::User,
    },
    ports::{FeedOptions, FeedQuery, FeedRequest, ThoughtRepository, UserWriter},
    value_objects::*,
};

async fn seed(pool: &sqlx::PgPool, username: &str, content: &str) -> (User, Thought) {
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

#[sqlx::test(migrations = "./migrations")]
async fn public_feed_returns_local_thoughts(pool: sqlx::PgPool) {
    let (_, _) = seed(&pool, "alice", "hello").await;
    let repo = PgFeedRepository::new(pool);
    let result = repo
        .query(&FeedRequest {
            query: FeedQuery::public(
                PageParams {
                    page: 1,
                    per_page: 20,
                },
                None,
            ),
            options: FeedOptions::default(),
        })
        .await
        .unwrap();
    assert_eq!(result.total, 1);
    assert_eq!(result.items[0].thought.content.as_str(), "hello");
}

#[sqlx::test(migrations = "./migrations")]
async fn search_returns_matching_thoughts(pool: sqlx::PgPool) {
    let (_, _) = seed(&pool, "alice", "hello world").await;
    let (_, _) = seed(&pool, "bob", "goodbye world").await;
    let repo = PgFeedRepository::new(pool);
    let result = repo
        .query(&FeedRequest {
            query: FeedQuery::search(
                "hello world",
                PageParams {
                    page: 1,
                    per_page: 20,
                },
                None,
            ),
            options: FeedOptions::default(),
        })
        .await
        .unwrap();
    assert!(result.total >= 1);
    assert!(result
        .items
        .iter()
        .any(|e| e.thought.content.as_str() == "hello world"));
}
