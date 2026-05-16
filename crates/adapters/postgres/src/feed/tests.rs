    use super::*;
    use crate::{thought::PgThoughtRepository, user::PgUserRepository};
    use domain::{
        models::{
            feed::PageParams,
            thought::{Thought, Visibility},
            user::User,
        },
        ports::{FeedQuery, ThoughtRepository, UserWriter},
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
        let t = Thought::new_local(
            ThoughtId::new(),
            u.id.clone(),
            Content::new_local(content).unwrap(),
            None,
            Visibility::Public,
            None,
            false,
        );
        trepo.save(&t).await.unwrap();
        (u, t)
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn public_feed_returns_local_thoughts(pool: sqlx::PgPool) {
        let (_, _) = seed(&pool, "alice", "hello").await;
        let repo = PgFeedRepository::new(pool);
        let result = repo
            .query(&FeedQuery::public(
                PageParams { page: 1, per_page: 20 },
                None,
            ))
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
            .query(&FeedQuery::search(
                "hello world",
                PageParams { page: 1, per_page: 20 },
                None,
            ))
            .await
            .unwrap();
        assert!(result.total >= 1);
        assert!(result
            .items
            .iter()
            .any(|e| e.thought.content.as_str() == "hello world"));
    }
