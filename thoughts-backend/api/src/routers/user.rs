use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use serde_json::{json, Value};

use app::persistence::{
    follow,
    thought::get_thoughts_by_user,
    user::{get_user, search_users},
};
use app::state::AppState;
use app::{error::UserError, persistence::user::get_user_by_username};
use models::schemas::thought::ThoughtListSchema;
use models::schemas::user::{UserListSchema, UserSchema};
use models::{queries::user::UserQuery, schemas::thought::ThoughtSchema};

use crate::extractor::Json;
use crate::models::ApiErrorResponse;
use crate::{error::ApiError, extractor::AuthUser};

#[utoipa::path(
    get,
    path = "",
    params(
        UserQuery
    ),
    responses(
        (status = 200, description = "List users", body = UserListSchema),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
async fn users_get(
    state: State<AppState>,
    query: Query<UserQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let Query(query) = query;

    let users = search_users(&state.conn, query)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(UserListSchema::from(users)))
}

#[utoipa::path(
    get,
    path = "/{username}/thoughts",
    params(
        ("username" = String, Path, description = "Username")
    ),
    responses(
        (status = 200, description = "List of user's thoughts", body = ThoughtListSchema),
        (status = 404, description = "User not found", body = ApiErrorResponse)
    )
)]
async fn user_thoughts_get(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let user = get_user_by_username(&state.conn, &username)
        .await?
        .ok_or(UserError::NotFound)?;

    let thoughts_with_authors = get_thoughts_by_user(&state.conn, user.id).await?;

    let thoughts_schema: Vec<ThoughtSchema> = thoughts_with_authors
        .into_iter()
        .map(ThoughtSchema::from)
        .collect();

    Ok(Json(ThoughtListSchema::from(thoughts_schema)))
}

#[utoipa::path(
    post,
    path = "/{username}/follow",
    params(
        ("username" = String, Path, description = "Username to follow")
    ),
    responses(
        (status = 204, description = "User followed successfully"),
        (status = 404, description = "User not found", body = ApiErrorResponse),
        (status = 409, description = "Already following", body = ApiErrorResponse)
    ),
    security(
        ("api_key" = []),
        ("bearer_auth" = [])
    )
)]
async fn user_follow_post(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(username): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let user_to_follow = get_user_by_username(&state.conn, &username)
        .await?
        .ok_or(UserError::NotFound)?;

    let result = follow::follow_user(&state.conn, auth_user.id, user_to_follow.id).await;

    match result {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e)
            if matches!(
                e.sql_err(),
                Some(sea_orm::SqlErr::UniqueConstraintViolation { .. })
            ) =>
        {
            Err(UserError::AlreadyFollowing.into())
        }
        Err(e) => Err(e.into()),
    }
}

#[utoipa::path(
    delete,
    path = "/{username}/follow",
    params(
        ("username" = String, Path, description = "Username to unfollow")
    ),
    responses(
        (status = 204, description = "User unfollowed successfully"),
        (status = 404, description = "User not found or not being followed", body = ApiErrorResponse)
    ),
    security(
        ("api_key" = []),
        ("bearer_auth" = [])
    )
)]
async fn user_follow_delete(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(username): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let user_to_unfollow = get_user_by_username(&state.conn, &username)
        .await?
        .ok_or(UserError::NotFound)?;

    follow::unfollow_user(&state.conn, auth_user.id, user_to_unfollow.id).await?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/{username}/inbox",
    request_body = Object,
    description = "The ActivityPub inbox for receiving activities.",
    responses(
        (status = 202, description = "Activity accepted"),
        (status = 400, description = "Bad Request"),
        (status = 404, description = "User not found")
    )
)]
async fn user_inbox_post(
    State(state): State<AppState>,
    Path(username): Path<String>,
    Json(activity): Json<Value>,
) -> Result<impl IntoResponse, ApiError> {
    let user = get_user_by_username(&state.conn, &username)
        .await?
        .ok_or(UserError::NotFound)?;

    let activity_type = activity["type"].as_str().unwrap_or_default();
    let actor_id = activity["actor"].as_str().unwrap_or_default();

    tracing::debug!(target: "activitypub", "Received activity '{}' from actor '{}' in {}'s inbox", activity_type, actor_id, username);

    // For now, we only handle the "Follow" activity
    if activity_type == "Follow" {
        follow::add_follower(&state.conn, user.id, actor_id).await?;
    }

    // Per the ActivityPub spec, we should return a 202 Accepted status
    Ok(StatusCode::ACCEPTED)
}

#[utoipa::path(
    get,
    path = "/{param}",
    params(
        ("param" = String, Path, description = "User ID or username")
    ),
    responses(
        (status = 200, description = "User profile or ActivityPub actor", body = UserSchema, content_type = "application/json"),
        (status = 200, description = "ActivityPub actor", body = Object, content_type = "application/activity+json"),
        (status = 404, description = "User not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
    security(
        ("api_key" = []),
        ("bearer_auth" = [])
    )
)]
async fn get_user_by_param(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Path(param): Path<String>,
) -> Response {
    // First, try to handle it as a numeric ID.
    if let Ok(id) = param.parse::<i32>() {
        return match get_user(&state.conn, id).await {
            Ok(Some(user)) => Json(UserSchema::from(user)).into_response(),
            Ok(None) => ApiError::from(UserError::NotFound).into_response(),
            Err(db_err) => ApiError::from(db_err).into_response(),
        };
    }

    // If it's not a number, treat it as a username and perform content negotiation.
    let username = param;
    let is_activitypub_request = headers
        .get(axum::http::header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .map_or(false, |s| s.contains("application/activity+json"));

    if is_activitypub_request {
        // This is the logic from `user_actor_get`.
        match get_user_by_username(&state.conn, &username).await {
            Ok(Some(user)) => {
                let user_url = format!("{}/users/{}", &state.base_url, user.username);
                let actor = json!({
                    "@context": [
                        "https://www.w3.org/ns/activitystreams",
                        "https://w3id.org/security/v1"
                    ],
                    "id": user_url,
                    "type": "Person",
                    "preferredUsername": user.username,
                    "inbox": format!("{}/inbox", user_url),
                    "outbox": format!("{}/outbox", user_url),
                });
                let mut headers = axum::http::HeaderMap::new();
                headers.insert(
                    axum::http::header::CONTENT_TYPE,
                    "application/activity+json".parse().unwrap(),
                );
                (headers, Json(actor)).into_response()
            }
            Ok(None) => ApiError::from(UserError::NotFound).into_response(),
            Err(e) => ApiError::from(e).into_response(),
        }
    } else {
        match get_user_by_username(&state.conn, &username).await {
            Ok(Some(user)) => Json(UserSchema::from(user)).into_response(),
            Ok(None) => ApiError::from(UserError::NotFound).into_response(),
            Err(e) => ApiError::from(e).into_response(),
        }
    }
}

#[utoipa::path(
    get,
    path = "/{username}/outbox",
    description = "The ActivityPub outbox for sending activities.",
    responses(
        (status = 200, description = "Activity collection", body = Object),
        (status = 404, description = "User not found")
    )
)]
async fn user_outbox_get(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let user = get_user_by_username(&state.conn, &username)
        .await?
        .ok_or(UserError::NotFound)?;

    let thoughts = get_thoughts_by_user(&state.conn, user.id).await?;

    // Format the outbox as an ActivityPub OrderedCollection
    let outbox_url = format!("{}/users/{}/outbox", &state.base_url, username);
    let items: Vec<Value> = thoughts
        .into_iter()
        .map(|thought| {
            let thought_url = format!("{}/thoughts/{}", &state.base_url, thought.id);
            let author_url = format!("{}/users/{}", &state.base_url, thought.author_username);
            json!({
                "id": format!("{}/activity", thought_url),
                "type": "Create",
                "actor": author_url,
                "published": thought.created_at,
                "to": ["https://www.w3.org/ns/activitystreams#Public"],
                "object": {
                    "id": thought_url,
                    "type": "Note",
                    "attributedTo": author_url,
                    "content": thought.content,
                    "published": thought.created_at,
                }
            })
        })
        .collect();

    let outbox = json!({
        "@context": "https://www.w3.org/ns/activitystreams",
        "id": outbox_url,
        "type": "OrderedCollection",
        "totalItems": items.len(),
        "orderedItems": items,
    });

    let mut headers = axum::http::HeaderMap::new();
    headers.insert(
        axum::http::header::CONTENT_TYPE,
        "application/activity+json".parse().unwrap(),
    );

    Ok((headers, Json(outbox)))
}

pub fn create_user_router() -> Router<AppState> {
    Router::new()
        .route("/", get(users_get))
        .route("/{param}", get(get_user_by_param))
        .route("/{username}/thoughts", get(user_thoughts_get))
        .route(
            "/{username}/follow",
            post(user_follow_post).delete(user_follow_delete),
        )
        .route("/{username}/inbox", post(user_inbox_post))
        .route("/{username}/outbox", get(user_outbox_get))
}
