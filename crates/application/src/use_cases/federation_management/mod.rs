use domain::{
    errors::DomainError,
    events::DomainEvent,
    models::{
        actor_connection_summary::ActorConnectionSummary,
        feed::{FeedEntry, PageParams, Paginated},
        remote_actor::RemoteActor,
    },
    ports::{
        EventPublisher, FederationActionPort, FederationContentRepository, FederationFollowPort,
        FederationFollowRequestPort, FederationSchedulerPort, FeedOptions, FeedQuery,
        FeedRepository, FeedRequest, FollowRepository, RemoteActorConnectionRepository, UserReader,
        UserWriter,
    },
    value_objects::UserId,
};

use super::social;

pub async fn initiate_actor_move(
    events: &dyn EventPublisher,
    user_id: &UserId,
    new_actor_url: url::Url,
) -> Result<(), DomainError> {
    events
        .publish(&DomainEvent::ActorMoved {
            user_id: user_id.clone(),
            new_actor_url: new_actor_url.to_string(),
        })
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))
}

pub async fn list_pending_requests(
    federation: &dyn FederationFollowRequestPort,
    user_id: &UserId,
) -> Result<Vec<RemoteActor>, DomainError> {
    federation.get_pending_followers(user_id).await
}

pub async fn accept_follow_request(
    federation: &dyn FederationFollowRequestPort,
    events: &dyn EventPublisher,
    user_id: &UserId,
    actor_url: &str,
) -> Result<(), DomainError> {
    events
        .publish(&DomainEvent::RemoteFollowAccepted {
            local_user_id: user_id.clone(),
            remote_actor_url: actor_url.to_string(),
        })
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;
    federation.mark_follower_accepted(user_id, actor_url).await
}

pub async fn reject_follow_request(
    federation: &dyn FederationFollowRequestPort,
    events: &dyn EventPublisher,
    user_id: &UserId,
    actor_url: &str,
) -> Result<(), DomainError> {
    events
        .publish(&DomainEvent::RemoteFollowRejected {
            local_user_id: user_id.clone(),
            remote_actor_url: actor_url.to_string(),
        })
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;
    federation.mark_follower_rejected(user_id, actor_url).await
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

pub async fn get_remote_friends(
    federation: &dyn FederationActionPort,
    user_id: &UserId,
) -> Result<Vec<RemoteActor>, DomainError> {
    use std::collections::HashSet;
    let following = federation.get_remote_following(user_id).await?;
    let followers = federation.get_remote_followers(user_id).await?;
    let follower_urls: HashSet<&str> = followers.iter().map(|a| a.url.as_str()).collect();
    Ok(following
        .into_iter()
        .filter(|a| follower_urls.contains(a.url.as_str()))
        .collect())
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
    ap_repo: &dyn FederationContentRepository,
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
    let result = feed
        .query(&FeedRequest {
            query: FeedQuery::user(author_id, page.clone(), viewer_id.cloned()),
            options: FeedOptions::default(),
        })
        .await?;
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
        // Always fetch from page 1 — the full collection is fetched and chunked.
        let _ = scheduler
            .schedule_connections_fetch(&actor.url, &collection_url, connection_type, 1)
            .await;
    }
    let has_more = items.len() >= PAGE_SIZE;
    Ok((items, has_more))
}

pub async fn set_also_known_as(
    users: &dyn UserWriter,
    user_id: &UserId,
    value: Option<String>,
) -> Result<(), DomainError> {
    users.set_also_known_as(user_id, value).await
}

#[cfg(test)]
mod tests;
