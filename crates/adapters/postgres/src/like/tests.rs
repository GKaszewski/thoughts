    use super::*;
    use crate::test_helpers::seed_user_and_thought;
    use chrono::Utc;
    use domain::value_objects::*;

    #[sqlx::test(migrations = "./migrations")]
    async fn like_and_count(pool: sqlx::PgPool) {
        let (user, thought) = seed_user_and_thought(&pool).await;
        let repo = PgLikeRepository::new(pool);
        let like = Like {
            id: LikeId::new(),
            user_id: user.id.clone(),
            thought_id: thought.id.clone(),
            ap_id: None,
            created_at: Utc::now(),
        };
        repo.save(&like).await.unwrap();
        assert_eq!(repo.count_for_thought(&thought.id).await.unwrap(), 1);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn unlike(pool: sqlx::PgPool) {
        let (user, thought) = seed_user_and_thought(&pool).await;
        let repo = PgLikeRepository::new(pool);
        let like = Like {
            id: LikeId::new(),
            user_id: user.id.clone(),
            thought_id: thought.id.clone(),
            ap_id: None,
            created_at: Utc::now(),
        };
        repo.save(&like).await.unwrap();
        repo.delete(&user.id, &thought.id).await.unwrap();
        assert_eq!(repo.count_for_thought(&thought.id).await.unwrap(), 0);
    }
