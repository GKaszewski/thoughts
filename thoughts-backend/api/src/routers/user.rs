use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use sea_orm::{DbErr, TryIntoModel};

use app::persistence::{
    follow,
    thought::get_thoughts_by_user,
    user::{create_user, get_user, search_users},
};
use app::state::AppState;
use app::{error::UserError, persistence::user::get_user_by_username};
use models::schemas::user::{UserListSchema, UserSchema};
use models::{params::user::CreateUserParams, schemas::thought::ThoughtListSchema};
use models::{queries::user::UserQuery, schemas::thought::ThoughtSchema};

use crate::extractor::{Json, Valid};
use crate::models::{ApiErrorResponse, ParamsErrorResponse};
use crate::{error::ApiError, extractor::AuthUser};

#[utoipa::path(
    post,
    path = "",
    request_body = CreateUserParams,
    responses(
        (status = 201, description = "User created", body = UserSchema),
        (status = 400, description = "Bad request", body = ApiErrorResponse),
        (status = 409, description = "Username already exists", body = ApiErrorResponse),
        (status = 422, description = "Validation error", body = ParamsErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
async fn users_post(
    state: State<AppState>,
    Valid(Json(params)): Valid<Json<CreateUserParams>>,
) -> Result<impl IntoResponse, ApiError> {
    let user = create_user(&state.conn, params)
        .await
        .map_err(ApiError::from)?;

    let user = user.try_into_model().unwrap();
    Ok((StatusCode::CREATED, Json(UserSchema::from(user))))
}

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
    path = "/{id}",
    params(
        ("id" = i32, Path, description = "User id")
    ),
    responses(
        (status = 200, description = "Get user", body = UserSchema),
        (status = 404, description = "Not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
async fn users_id_get(
    state: State<AppState>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApiError> {
    let user = get_user(&state.conn, id).await.map_err(ApiError::from)?;

    user.map(|user| Json(UserSchema::from(user)))
        .ok_or_else(|| UserError::NotFound.into())
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
        Err(DbErr::UnpackInsertId) => Err(UserError::AlreadyFollowing.into()),
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

pub fn create_user_router() -> Router<AppState> {
    Router::new()
        .route("/", post(users_post).get(users_get))
        .route("/{id}", get(users_id_get))
        .route("/{username}/thoughts", get(user_thoughts_get))
        .route(
            "/{username}/follow",
            post(user_follow_post).delete(user_follow_delete),
        )
}
