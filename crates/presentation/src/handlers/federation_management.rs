use crate::{
    deps_struct,
    errors::ApiError,
    extractors::{AuthUser, Deps},
};
use api_types::responses::{ProfileField, RemoteActorResponse};
use application::use_cases::federation_management::{
    accept_follow_request, list_pending_requests, list_remote_followers, list_remote_following,
    reject_follow_request, remove_remote_following,
};
use axum::{http::StatusCode, Json};
use domain::ports::{EventPublisher, FederationActionPort, FollowRepository, UserRepository};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ActorUrlBody {
    pub actor_url: String,
}

#[derive(Deserialize)]
pub struct HandleBody {
    pub handle: String,
}

#[derive(Deserialize)]
pub struct MoveBody {
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

pub async fn get_pending_requests(
    Deps(d): Deps<FederationManagementDeps>,
    AuthUser(uid): AuthUser,
) -> Result<Json<Vec<RemoteActorResponse>>, ApiError> {
    let actors = list_pending_requests(&*d.federation, &uid).await?;
    Ok(Json(actors.into_iter().map(to_response).collect()))
}

pub async fn post_accept_request(
    Deps(d): Deps<FederationManagementDeps>,
    AuthUser(uid): AuthUser,
    Json(body): Json<ActorUrlBody>,
) -> Result<StatusCode, ApiError> {
    accept_follow_request(&*d.federation, &uid, &body.actor_url).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_follower(
    Deps(d): Deps<FederationManagementDeps>,
    AuthUser(uid): AuthUser,
    Json(body): Json<ActorUrlBody>,
) -> Result<StatusCode, ApiError> {
    reject_follow_request(&*d.federation, &uid, &body.actor_url).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_remote_followers(
    Deps(d): Deps<FederationManagementDeps>,
    AuthUser(uid): AuthUser,
) -> Result<Json<Vec<RemoteActorResponse>>, ApiError> {
    let actors = list_remote_followers(&*d.federation, &uid).await?;
    Ok(Json(actors.into_iter().map(to_response).collect()))
}

pub async fn get_remote_following(
    Deps(d): Deps<FederationManagementDeps>,
    AuthUser(uid): AuthUser,
) -> Result<Json<Vec<RemoteActorResponse>>, ApiError> {
    let actors = list_remote_following(&*d.federation, &uid).await?;
    Ok(Json(actors.into_iter().map(to_response).collect()))
}

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

pub async fn post_move_account(
    Deps(d): Deps<FederationManagementDeps>,
    AuthUser(uid): AuthUser,
    Json(body): Json<MoveBody>,
) -> Result<StatusCode, ApiError> {
    let new_url = url::Url::parse(&body.new_actor_url)
        .map_err(|_| ApiError::BadRequest("invalid new_actor_url".into()))?;
    d.federation
        .broadcast_move(&uid, new_url)
        .await
        .map_err(ApiError::from)?;
    Ok(StatusCode::NO_CONTENT)
}
