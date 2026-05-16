    use super::*;
    use crate::test_helpers;
    use chrono::Utc;
    use domain::{
        models::{notification::NotificationKind, user::User},
        value_objects::*,
    };

    #[sqlx::test(migrations = "./migrations")]
    async fn save_and_list(pool: sqlx::PgPool) {
        let user = test_helpers::seed_user(&pool, "alice", "alice@ex.com").await;
        let from_user = test_helpers::seed_user(&pool, "bob", "bob@ex.com").await;
        let repo = PgNotificationRepository::new(pool);
        use domain::models::feed::PageParams;
        let n = Notification {
            id: NotificationId::new(),
            user_id: user.id.clone(),
            kind: NotificationKind::Follow {
                from_user_id: from_user.id.clone(),
            },
            read: false,
            created_at: Utc::now(),
        };
        repo.save(&n).await.unwrap();
        let page = repo
            .list_for_user(
                &user.id,
                &PageParams {
                    page: 1,
                    per_page: 20,
                },
            )
            .await
            .unwrap();
        assert_eq!(page.total, 1);
        assert!(!page.items[0].read);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn mark_all_read(pool: sqlx::PgPool) {
        let user = test_helpers::seed_user(&pool, "alice", "alice@ex.com").await;
        let from_user = test_helpers::seed_user(&pool, "bob", "bob@ex.com").await;
        let repo = PgNotificationRepository::new(pool);
        use domain::models::feed::PageParams;
        let n = Notification {
            id: NotificationId::new(),
            user_id: user.id.clone(),
            kind: NotificationKind::Follow {
                from_user_id: from_user.id.clone(),
            },
            read: false,
            created_at: Utc::now(),
        };
        repo.save(&n).await.unwrap();
        repo.mark_all_read(&user.id).await.unwrap();
        let page = repo
            .list_for_user(
                &user.id,
                &PageParams {
                    page: 1,
                    per_page: 20,
                },
            )
            .await
            .unwrap();
        assert!(page.items[0].read);
    }
