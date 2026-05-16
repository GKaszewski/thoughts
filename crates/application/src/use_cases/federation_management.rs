use activitypub_base::ActivityPubRepository;
use domain::{
    errors::DomainError,
    models::{
        actor_connection_summary::ActorConnectionSummary,
        feed::{FeedEntry, PageParams, Paginated},
        remote_actor::RemoteActor,
    },
    ports::{
        EventPublisher, FederationActionPort, FederationFollowPort,
        FederationFollowRequestPort, FederationSchedulerPort, FeedQuery, FeedRepository,
        FollowRepository, RemoteActorConnectionRepository, UserReader,
    },
    value_objects::UserId,
};

use super::social;

pub async fn list_pending_requests(
    federation: &dyn FederationFollowRequestPort,
    user_id: &UserId,
) -> Result<Vec<RemoteActor>, DomainError> {
    federation.get_pending_followers(user_id).await
}

pub async fn accept_follow_request(
    federation: &dyn FederationFollowRequestPort,
    user_id: &UserId,
    actor_url: &str,
) -> Result<(), DomainError> {
    federation.accept_follow_request(user_id, actor_url).await
}

pub async fn reject_follow_request(
    federation: &dyn FederationFollowRequestPort,
    user_id: &UserId,
    actor_url: &str,
) -> Result<(), DomainError> {
    federation.reject_follow_request(user_id, actor_url).await
}

pub async fn list_remote_followers(
    federation: &dyn FederationFollowRequestPort,
    user_id: &UserId,
) -> Result<Vec<RemoteActor>, DomainError> {
    federation.get_remote_followers(user_id).await
}

pub async fn remove_remote_follower(
    federation: &dyn FederationFollowRequestPort,
    user_id: &UserId,
    actor_url: &str,
) -> Result<(), DomainError> {
    federation.remove_remote_follower(user_id, actor_url).await
}

pub async fn list_remote_following(
    federation: &dyn FederationFollowPort,
    user_id: &UserId,
) -> Result<Vec<RemoteActor>, DomainError> {
    federation.get_remote_following(user_id).await
}

pub async fn remove_remote_following(
    follows: &dyn FollowRepository,
    users: &dyn UserReader,
    federation: &dyn FederationFollowPort,
    events: &dyn EventPublisher,
    user_id: &UserId,
    handle: &str,
) -> Result<(), DomainError> {
    social::unfollow_actor(follows, users, federation, events, user_id, handle).await
}

pub async fn get_remote_actor_posts(
    federation: &dyn FederationActionPort,
    ap_repo: &dyn ActivityPubRepository,
    feed: &dyn FeedRepository,
    scheduler: &dyn FederationSchedulerPort,
    handle: &str,
    page: PageParams,
    viewer_id: Option<&UserId>,
) -> Result<Paginated<FeedEntry>, DomainError> {
    let actor = federation.lookup_actor(handle).await?;
    let author_id = match ap_repo.find_remote_actor_id(&actor.url).await? {
        Some(id) => id,
        None => ap_repo.intern_remote_actor(&actor.url).await?,
    };
    let result = feed.query(&FeedQuery::user(author_id, page.clone(), viewer_id.cloned())).await?;
    if let Some(outbox_url) = actor.outbox_url {
        let _ = scheduler
            .schedule_actor_posts_fetch(&actor.url, &outbox_url)
            .await;
    }
    Ok(result)
}

const ACTOR_CONNECTIONS_CACHE_TTL_SECS: i64 = 3600;

pub async fn get_actor_connections_page(
    federation: &dyn FederationActionPort,
    connections: &dyn RemoteActorConnectionRepository,
    scheduler: &dyn FederationSchedulerPort,
    handle: &str,
    connection_type: &str,
    page: u32,
) -> Result<(Vec<ActorConnectionSummary>, bool), DomainError> {
    const PAGE_SIZE: usize = 20;
    let actor = federation.lookup_actor(handle).await?;
    let collection_url = match connection_type {
        "followers" => actor.followers_url.ok_or(DomainError::NotFound)?,
        _ => actor.following_url.ok_or(DomainError::NotFound)?,
    };
    let items = connections
        .list_connections(&actor.url, connection_type, page)
        .await?;
    let stale = match connections
        .connection_page_age(&actor.url, connection_type, page)
        .await?
    {
        None => true,
        Some(age) => {
            chrono::Utc::now().signed_duration_since(age).num_seconds()
                > ACTOR_CONNECTIONS_CACHE_TTL_SECS
        }
    };
    if stale {
        let _ = scheduler
            .schedule_connections_fetch(&actor.url, &collection_url, connection_type, page)
            .await;
    }
    let has_more = items.len() >= PAGE_SIZE;
    Ok((items, has_more))
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::testing::TestStore;

    #[tokio::test]
    async fn list_pending_returns_empty_by_default() {
        let store = TestStore::default();
        let uid = UserId::new();
        let result = list_pending_requests(&store, &uid).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn accept_follow_request_returns_ok() {
        let store = TestStore::default();
        let uid = UserId::new();
        accept_follow_request(&store, &uid, "https://mastodon.social/users/alice")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn reject_follow_request_returns_ok() {
        let store = TestStore::default();
        let uid = UserId::new();
        reject_follow_request(&store, &uid, "https://mastodon.social/users/alice")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn list_remote_followers_returns_empty_by_default() {
        let store = TestStore::default();
        let uid = UserId::new();
        let result = list_remote_followers(&store, &uid).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn remove_remote_follower_returns_ok() {
        let store = TestStore::default();
        let uid = UserId::new();
        remove_remote_follower(&store, &uid, "https://mastodon.social/users/alice")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn list_remote_following_returns_empty_by_default() {
        let store = TestStore::default();
        let uid = UserId::new();
        let result = list_remote_following(&store, &uid).await.unwrap();
        assert!(result.is_empty());
    }
}
