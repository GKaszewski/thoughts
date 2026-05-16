mod federation_port_tests {
    use super::super::*;
    use crate::value_objects::UserId;

    fn uid() -> UserId {
        UserId::new()
    }

    #[tokio::test]
    async fn test_store_lookup_returns_not_found() {
        let store = TestStore::default();
        let err = store.lookup_actor("@alice@example.com").await.unwrap_err();
        assert!(matches!(err, DomainError::NotFound));
    }

    #[tokio::test]
    async fn test_store_follow_remote_is_noop_ok() {
        let store = TestStore::default();
        store
            .follow_remote(&uid(), "@alice@example.com")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_store_actor_json_returns_not_found() {
        let store = TestStore::default();
        let err = store.actor_json(&UserId::new()).await.unwrap_err();
        assert!(matches!(err, DomainError::NotFound));
    }

    #[tokio::test]
    async fn test_store_fetch_outbox_returns_empty() {
        let store = TestStore::default();
        let notes = store
            .fetch_outbox_page("https://example.com/outbox", 1)
            .await
            .unwrap();
        assert!(notes.is_empty());
    }

    #[tokio::test]
    async fn test_store_resolve_actor_profiles_returns_empty() {
        let store = TestStore::default();
        let result = store
            .resolve_actor_profiles(vec!["https://example.com/users/alice".into()])
            .await;
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_store_fetch_collection_urls_returns_empty() {
        let store = TestStore::default();
        let urls = store
            .fetch_actor_urls_from_collection("https://example.com/users/alice/followers")
            .await
            .unwrap();
        assert!(urls.is_empty());
    }
}

mod search_tests {
    use super::super::*;
    use crate::models::feed::PageParams;

    #[tokio::test]
    async fn test_store_search_thoughts_returns_empty() {
        let store = TestStore::default();
        let result = store
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
        assert_eq!(result.total, 0);
    }

    #[tokio::test]
    async fn test_store_search_users_returns_empty() {
        let store = TestStore::default();
        let result = store
            .search_users(
                "alice",
                &PageParams {
                    page: 1,
                    per_page: 20,
                },
            )
            .await
            .unwrap();
        assert_eq!(result.total, 0);
    }
}
