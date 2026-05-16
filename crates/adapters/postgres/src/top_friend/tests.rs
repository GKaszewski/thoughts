    use super::*;
    use crate::user::PgUserRepository;
    use domain::ports::UserWriter;
    use domain::{models::user::User, value_objects::*};

    async fn seed_user(pool: &sqlx::PgPool, username: &str, email: &str) -> User {
        let repo = PgUserRepository::new(pool.clone());
        let u = User::new_local(
            UserId::new(),
            Username::new(username).unwrap(),
            Email::new(email).unwrap(),
            PasswordHash("h".into()),
        );
        repo.save(&u).await.unwrap();
        u
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn set_and_list_top_friends(pool: sqlx::PgPool) {
        let alice = seed_user(&pool, "alice", "alice@ex.com").await;
        let bob = seed_user(&pool, "bob", "bob@ex.com").await;
        let repo = PgTopFriendRepository::new(pool);
        repo.set_top_friends(&alice.id, vec![(bob.id.clone(), 1)])
            .await
            .unwrap();
        let friends = repo.list_for_user(&alice.id).await.unwrap();
        assert_eq!(friends.len(), 1);
        assert_eq!(friends[0].0.position, 1);
        assert_eq!(friends[0].1.username.as_str(), "bob");
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn replace_top_friends(pool: sqlx::PgPool) {
        let alice = seed_user(&pool, "alice", "alice@ex.com").await;
        let bob = seed_user(&pool, "bob", "bob@ex.com").await;
        let carol = seed_user(&pool, "carol", "carol@ex.com").await;
        let repo = PgTopFriendRepository::new(pool);
        repo.set_top_friends(&alice.id, vec![(bob.id.clone(), 1)])
            .await
            .unwrap();
        repo.set_top_friends(&alice.id, vec![(carol.id.clone(), 1)])
            .await
            .unwrap();
        let friends = repo.list_for_user(&alice.id).await.unwrap();
        assert_eq!(friends.len(), 1);
        assert_eq!(friends[0].1.username.as_str(), "carol");
    }
