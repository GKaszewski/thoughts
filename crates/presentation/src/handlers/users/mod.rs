use crate::{
    errors::ApiError,
    extractors::{AuthUser, Deps, FromAppState, OptionalAuthUser},
    handlers::auth::to_user_response,
    state::AppState,
};
use api_types::{
    requests::{PaginationQuery, UpdateProfileRequest},
    responses::{ErrorResponse, ProfileField, RemoteActorResponse, UserResponse},
};
use application::use_cases::profile::{
    get_user as fetch_user, get_user_by_id_or_username, update_profile,
};
use axum::{
    extract::{Path, Query},
    http::{header, HeaderMap},
    response::{IntoResponse, Response},
    Json,
};
use domain::{
    models::user::UpdateProfileInput,
    ports::{EventPublisher, FederationActionPort, FollowRepository, SearchPort, UserRepository},
};
use std::sync::Arc;

pub struct UsersDeps {
    pub users: Arc<dyn UserRepository>,
    pub events: Arc<dyn EventPublisher>,
    pub follows: Arc<dyn FollowRepository>,
    pub federation: Arc<dyn FederationActionPort>,
    pub search: Arc<dyn SearchPort>,
}

impl FromAppState for UsersDeps {
    fn from_state(s: &AppState) -> Self {
        Self {
            users: s.users.clone(),
            events: s.events.clone(),
            follows: s.follows.clone(),
            federation: s.federation.clone(),
            search: s.search.clone(),
        }
    }
}

#[utoipa::path(
    get, path = "/users/{username}",
    params(("username" = String, Path, description = "Username")),
    responses(
        (status = 200, body = UserResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
    )
)]
pub async fn get_user(
    Deps(d): Deps<UsersDeps>,
    Path(username): Path<String>,
    OptionalAuthUser(viewer): OptionalAuthUser,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let user = get_user_by_id_or_username(&*d.users, &username).await?;

    let accept = headers
        .get(header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if accept.contains("application/activity+json") {
        let json = d.federation.actor_json(&user.id).await?;
        Ok(([(header::CONTENT_TYPE, "application/activity+json")], json).into_response())
    } else {
        let is_followed = if let Some(viewer_id) = viewer {
            d.follows.find(&viewer_id, &user.id).await?.is_some()
        } else {
            false
        };
        let mut resp = to_user_response(&user);
        resp.is_followed_by_viewer = is_followed;
        Ok(Json(resp).into_response())
    }
}

#[utoipa::path(
    patch, path = "/users/me",
    request_body = UpdateProfileRequest,
    responses(
        (status = 200, body = UserResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn patch_profile(
    Deps(d): Deps<UsersDeps>,
    AuthUser(uid): AuthUser,
    Json(body): Json<UpdateProfileRequest>,
) -> Result<Json<UserResponse>, ApiError> {
    update_profile(
        &*d.users,
        &*d.events,
        &uid,
        UpdateProfileInput {
            display_name: body.display_name,
            bio: body.bio,
            avatar_url: body.avatar_url,
            header_url: body.header_url,
            custom_css: body.custom_css,
        },
    )
    .await?;
    let user = fetch_user(&*d.users, &uid).await?;
    Ok(Json(to_user_response(&user)))
}

#[utoipa::path(
    get, path = "/users/me",
    responses(
        (status = 200, body = UserResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_me(
    Deps(d): Deps<UsersDeps>,
    AuthUser(uid): AuthUser,
) -> Result<Json<UserResponse>, ApiError> {
    let user = fetch_user(&*d.users, &uid).await?;
    Ok(Json(to_user_response(&user)))
}

pub async fn get_me_following(
    Deps(d): Deps<UsersDeps>,
    AuthUser(uid): AuthUser,
    Query(q): Query<PaginationQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    use domain::models::feed::PageParams;
    let page = PageParams {
        page: q.page(),
        per_page: q.per_page(),
    };
    let result = d.follows.list_following(&uid, &page).await?;
    Ok(Json(serde_json::json!({
        "total": result.total,
        "items": result.items.iter().map(to_user_response).collect::<Vec<_>>(),
    })))
}

pub async fn get_users(
    Deps(d): Deps<UsersDeps>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    use domain::models::feed::PageParams;
    let page = params
        .get("page")
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(1);
    let per_page = params
        .get("per_page")
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(20);
    let page_params = PageParams { page, per_page };

    if let Some(q) = params.get("q").filter(|q| !q.trim().is_empty()) {
        let result = d.search.search_users(q, &page_params).await?;
        let users: Vec<_> = result
            .items
            .iter()
            .map(crate::handlers::auth::to_user_response)
            .collect();
        return Ok(Json(serde_json::json!({
            "items": users, "total": result.total, "page": result.page, "per_page": result.per_page
        })));
    }

    let result = d.users.list_paginated(page_params).await?;
    let items: Vec<_> = result
        .items
        .iter()
        .map(|u| {
            serde_json::json!({
                "id": u.id.as_uuid(),
                "username": u.username,
                "displayName": u.display_name,
                "avatarUrl": u.avatar_url,
                "bio": u.bio,
                "headerUrl": null,
                "customCss": null,
                "local": true,
                "isFollowedByViewer": false,
                "joinedAt": null,
            })
        })
        .collect();
    Ok(Json(serde_json::json!({
        "items": items, "total": result.total, "page": result.page, "per_page": result.per_page
    })))
}

pub async fn get_user_count(Deps(d): Deps<UsersDeps>) -> Result<Json<serde_json::Value>, ApiError> {
    let count = d.users.count().await?;
    Ok(Json(serde_json::json!({ "count": count })))
}

#[derive(serde::Deserialize)]
pub struct LookupQuery {
    pub handle: String,
}

pub async fn lookup_handler(
    Deps(d): Deps<UsersDeps>,
    Query(q): Query<LookupQuery>,
) -> Result<Json<RemoteActorResponse>, ApiError> {
    let actor = d.federation.lookup_actor(&q.handle).await?;
    Ok(Json(RemoteActorResponse {
        handle: actor.handle,
        display_name: actor.display_name,
        avatar_url: actor.avatar_url,
        url: actor.url,
        bio: actor.bio,
        banner_url: actor.banner_url,
        also_known_as: actor.also_known_as,
        outbox_url: actor.outbox_url,
        followers_url: actor.followers_url,
        following_url: actor.following_url,
        attachment: actor
            .attachment
            .into_iter()
            .map(|(name, value)| ProfileField { name, value })
            .collect(),
    }))
}

#[cfg(test)]
mod tests;
