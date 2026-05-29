use crate::{
    errors::ApiError,
    extractors::{Deps, FromAppState, OptionalAuthUser},
    handlers::feed::to_thought_response,
    state::AppState,
};
use activitypub::ActivityPubRepository;
use api_types::{
    requests::PaginationQuery,
    responses::{ActorConnectionPageResponse, ActorConnectionResponse},
};
use application::use_cases::federation_management::{
    get_actor_connections_page, get_remote_actor_posts,
};
use axum::{
    extract::{Path, Query},
    Json,
};
use domain::{
    models::feed::PageParams,
    ports::{
        FederationActionPort, FederationSchedulerPort, FeedRepository,
        RemoteActorConnectionRepository,
    },
};
use std::sync::Arc;

pub struct FederationActorsDeps {
    pub federation: Arc<dyn FederationActionPort>,
    pub ap_repo: Arc<dyn ActivityPubRepository>,
    pub feed: Arc<dyn FeedRepository>,
    pub federation_scheduler: Arc<dyn FederationSchedulerPort>,
    pub remote_actor_connections: Arc<dyn RemoteActorConnectionRepository>,
}

impl FromAppState for FederationActorsDeps {
    fn from_state(s: &AppState) -> Self {
        Self {
            federation: s.federation.clone(),
            ap_repo: s.ap_repo.clone(),
            feed: s.feed.clone(),
            federation_scheduler: s.federation_scheduler.clone(),
            remote_actor_connections: s.remote_actor_connections.clone(),
        }
    }
}

#[utoipa::path(
    get, path = "/federation/actors/{handle}/posts",
    params(
        ("handle" = String, Path, description = "Fediverse handle (@user@instance.tld)"),
        PaginationQuery,
    ),
    responses((status = 200, description = "Posts by this remote actor"))
)]
pub async fn remote_actor_posts_handler(
    Deps(d): Deps<FederationActorsDeps>,
    Path(handle): Path<String>,
    Query(q): Query<PaginationQuery>,
    OptionalAuthUser(viewer): OptionalAuthUser,
) -> Result<Json<serde_json::Value>, ApiError> {
    let page = PageParams {
        page: q.page(),
        per_page: q.per_page(),
    };
    let result = get_remote_actor_posts(
        &*d.federation,
        &*d.ap_repo,
        &*d.feed,
        &*d.federation_scheduler,
        &handle,
        page,
        viewer.as_ref(),
    )
    .await?;
    Ok(Json(serde_json::json!({
        "total": result.total,
        "page": result.page,
        "per_page": result.per_page,
        "items": result.items.iter().map(to_thought_response).collect::<Vec<_>>(),
    })))
}

#[utoipa::path(
    get, path = "/federation/actors/{handle}/followers-list",
    params(
        ("handle" = String, Path, description = "Fediverse handle (@user@instance.tld)"),
        PaginationQuery,
    ),
    responses((status = 200, description = "Followers of this remote actor", body = ActorConnectionPageResponse)),
)]
pub async fn actor_followers_handler(
    Deps(d): Deps<FederationActorsDeps>,
    Path(handle): Path<String>,
    Query(q): Query<PaginationQuery>,
) -> Result<Json<ActorConnectionPageResponse>, ApiError> {
    actor_connections_handler(d, handle, "followers", q.page() as u32).await
}

#[utoipa::path(
    get, path = "/federation/actors/{handle}/following-list",
    params(
        ("handle" = String, Path, description = "Fediverse handle (@user@instance.tld)"),
        PaginationQuery,
    ),
    responses((status = 200, description = "Accounts this remote actor follows", body = ActorConnectionPageResponse)),
)]
pub async fn actor_following_handler(
    Deps(d): Deps<FederationActorsDeps>,
    Path(handle): Path<String>,
    Query(q): Query<PaginationQuery>,
) -> Result<Json<ActorConnectionPageResponse>, ApiError> {
    actor_connections_handler(d, handle, "following", q.page() as u32).await
}

async fn actor_connections_handler(
    d: FederationActorsDeps,
    handle: String,
    connection_type: &str,
    page: u32,
) -> Result<Json<ActorConnectionPageResponse>, ApiError> {
    let (items, has_more) = get_actor_connections_page(
        &*d.federation,
        &*d.remote_actor_connections,
        &*d.federation_scheduler,
        &handle,
        connection_type,
        page,
    )
    .await?;
    Ok(Json(ActorConnectionPageResponse {
        items: items
            .into_iter()
            .map(|a| ActorConnectionResponse {
                handle: a.handle,
                display_name: a.display_name,
                avatar_url: a.avatar_url,
                url: a.url,
            })
            .collect(),
        page,
        has_more,
    }))
}

#[cfg(test)]
mod tests;
