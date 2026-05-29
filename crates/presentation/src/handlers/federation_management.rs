use crate::{
    deps_struct,
    errors::ApiError,
    extractors::{AuthUser, Deps},
};
use api_types::responses::{ErrorResponse, ProfileField, RemoteActorResponse};
use application::use_cases::federation_management::{
    accept_follow_request, get_remote_friends, initiate_actor_move, list_pending_requests,
    list_remote_followers, list_remote_following, reject_follow_request, remove_remote_following,
    set_also_known_as,
};
use axum::{http::StatusCode, Json};
use domain::ports::{EventPublisher, FederationActionPort, FollowRepository, UserRepository};
use serde::Deserialize;

#[derive(Deserialize, utoipa::ToSchema)]
pub struct ActorUrlBody {
    /// Full ActivityPub actor URL
    pub actor_url: String,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct HandleBody {
    /// Fediverse handle (`@user@instance.tld`)
    pub handle: String,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct MoveBody {
    /// New actor URL to migrate to
    pub new_actor_url: String,
}

deps_struct!(FederationManagementDeps {
    federation: FederationActionPort,
    follows: FollowRepository,
    users: UserRepository,
    events: EventPublisher,
});

fn to_response(a: domain::models::remote_actor::RemoteActor) -> RemoteActorResponse {
    RemoteActorResponse {
        handle: a.handle,
        display_name: a.display_name,
        avatar_url: a.avatar_url,
        url: a.url,
        bio: a.bio,
        banner_url: a.banner_url,
        also_known_as: a.also_known_as,
        outbox_url: a.outbox_url,
        followers_url: a.followers_url,
        following_url: a.following_url,
        attachment: a
            .attachment
            .into_iter()
            .map(|(name, value)| ProfileField { name, value })
            .collect(),
    }
}

#[utoipa::path(
    get, path = "/federation/me/followers/pending",
    responses((status = 200, description = "Pending inbound follow requests", body = Vec<RemoteActorResponse>)),
    security(("bearer_auth" = []))
)]
pub async fn get_pending_requests(
    Deps(d): Deps<FederationManagementDeps>,
    AuthUser(uid): AuthUser,
) -> Result<Json<Vec<RemoteActorResponse>>, ApiError> {
    let actors = list_pending_requests(&*d.federation, &uid).await?;
    Ok(Json(actors.into_iter().map(to_response).collect()))
}

#[utoipa::path(
    post, path = "/federation/me/followers/accept",
    request_body = ActorUrlBody,
    responses(
        (status = 204, description = "Follow request accepted"),
        (status = 400, description = "Invalid request", body = ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn post_accept_request(
    Deps(d): Deps<FederationManagementDeps>,
    AuthUser(uid): AuthUser,
    Json(body): Json<ActorUrlBody>,
) -> Result<StatusCode, ApiError> {
    accept_follow_request(&*d.federation, &*d.events, &uid, &body.actor_url).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete, path = "/federation/me/followers",
    request_body = ActorUrlBody,
    responses(
        (status = 204, description = "Follower removed / request rejected"),
        (status = 400, description = "Invalid request", body = ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_follower(
    Deps(d): Deps<FederationManagementDeps>,
    AuthUser(uid): AuthUser,
    Json(body): Json<ActorUrlBody>,
) -> Result<StatusCode, ApiError> {
    reject_follow_request(&*d.federation, &*d.events, &uid, &body.actor_url).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get, path = "/federation/me/followers",
    responses((status = 200, description = "Accepted remote followers", body = Vec<RemoteActorResponse>)),
    security(("bearer_auth" = []))
)]
pub async fn get_remote_followers(
    Deps(d): Deps<FederationManagementDeps>,
    AuthUser(uid): AuthUser,
) -> Result<Json<Vec<RemoteActorResponse>>, ApiError> {
    let actors = list_remote_followers(&*d.federation, &uid).await?;
    Ok(Json(actors.into_iter().map(to_response).collect()))
}

#[utoipa::path(
    get, path = "/federation/me/following",
    responses((status = 200, description = "Remote accounts I follow", body = Vec<RemoteActorResponse>)),
    security(("bearer_auth" = []))
)]
pub async fn get_remote_following(
    Deps(d): Deps<FederationManagementDeps>,
    AuthUser(uid): AuthUser,
) -> Result<Json<Vec<RemoteActorResponse>>, ApiError> {
    let actors = list_remote_following(&*d.federation, &uid).await?;
    Ok(Json(actors.into_iter().map(to_response).collect()))
}

#[utoipa::path(
    get, path = "/federation/me/friends",
    responses((status = 200, description = "Remote mutual follows (I follow them and they follow me)", body = Vec<RemoteActorResponse>)),
    security(("bearer_auth" = []))
)]
pub async fn get_remote_friends_handler(
    Deps(d): Deps<FederationManagementDeps>,
    AuthUser(uid): AuthUser,
) -> Result<Json<Vec<RemoteActorResponse>>, ApiError> {
    let actors = get_remote_friends(&*d.federation, &uid).await?;
    Ok(Json(actors.into_iter().map(to_response).collect()))
}

#[utoipa::path(
    delete, path = "/federation/me/following",
    request_body = HandleBody,
    responses(
        (status = 204, description = "Unfollowed remote account"),
        (status = 400, description = "Invalid handle", body = ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_following(
    Deps(d): Deps<FederationManagementDeps>,
    AuthUser(uid): AuthUser,
    Json(body): Json<HandleBody>,
) -> Result<StatusCode, ApiError> {
    remove_remote_following(
        &*d.follows,
        &*d.users,
        &*d.federation,
        &*d.events,
        &uid,
        &body.handle,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post, path = "/federation/me/move",
    request_body = MoveBody,
    responses(
        (status = 204, description = "Account move initiated"),
        (status = 400, description = "Invalid URL", body = ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn post_move_account(
    Deps(d): Deps<FederationManagementDeps>,
    AuthUser(uid): AuthUser,
    Json(body): Json<MoveBody>,
) -> Result<StatusCode, ApiError> {
    let new_url = url::Url::parse(&body.new_actor_url)
        .map_err(|_| ApiError::BadRequest("invalid new_actor_url".into()))?;
    initiate_actor_move(&*d.events, &uid, new_url).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct AlsoKnownAsBody {
    /// Actor URL of the account this identity is also known as (for migration verification)
    pub also_known_as: Option<String>,
}

#[utoipa::path(
    patch, path = "/federation/me/also-known-as",
    request_body = AlsoKnownAsBody,
    responses((status = 204, description = "Also-known-as updated")),
    security(("bearer_auth" = []))
)]
pub async fn patch_also_known_as(
    Deps(d): Deps<FederationManagementDeps>,
    AuthUser(uid): AuthUser,
    Json(body): Json<AlsoKnownAsBody>,
) -> Result<StatusCode, ApiError> {
    set_also_known_as(&*d.users, &uid, body.also_known_as).await?;
    Ok(StatusCode::NO_CONTENT)
}
