use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};

use app::{
    error::UserError,
    persistence::thought::{create_thought, delete_thought, get_thought},
    state::AppState,
};
use models::{
    params::thought::CreateThoughtParams,
    schemas::thought::{ThoughtSchema, ThoughtThreadSchema},
};
use sea_orm::prelude::Uuid;

use crate::{
    error::ApiError,
    extractor::{AuthUser, Json, OptionalAuthUser, Valid},
    models::{ApiErrorResponse, ParamsErrorResponse},
};

#[utoipa::path(
    get,
    path = "/{id}",
    params(
        ("id" = Uuid, Path, description = "Thought ID")
    ),
    responses(
        (status = 200, description = "Thought found", body = ThoughtSchema),
        (status = 404, description = "Not Found", body = ApiErrorResponse)
    )
)]
async fn get_thought_by_id(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    viewer: OptionalAuthUser,
) -> Result<impl IntoResponse, ApiError> {
    let viewer_id = viewer.0.map(|u| u.id);
    let thought = get_thought(&state.conn, id, viewer_id)
        .await?
        .ok_or(UserError::NotFound)?;

    let author = app::persistence::user::get_user(&state.conn, thought.author_id)
        .await?
        .ok_or(UserError::NotFound)?;

    let schema = ThoughtSchema::from_models(&thought, &author);
    Ok(Json(schema))
}

#[utoipa::path(
    post,
    path = "",
    request_body = CreateThoughtParams,
    responses(
        (status = 201, description = "Thought created", body = ThoughtSchema),
        (status = 400, description = "Bad request", body = ApiErrorResponse),
        (status = 422, description = "Validation error", body = ParamsErrorResponse)
    ),
    security(
        ("api_key" = []),
        ("bearer_auth" = [])
    )
)]
async fn thoughts_post(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Valid(Json(params)): Valid<Json<CreateThoughtParams>>,
) -> Result<impl IntoResponse, ApiError> {
    let thought = create_thought(&state.conn, auth_user.id, params).await?;
    let author = app::persistence::user::get_user(&state.conn, auth_user.id)
        .await?
        .ok_or(UserError::NotFound)?; // Should not happen if auth is valid

    let schema = ThoughtSchema::from_models(&thought, &author);
    Ok((StatusCode::CREATED, Json(schema)))
}

#[utoipa::path(
    delete,
    path = "/{id}",
    params(
        ("id" = i32, Path, description = "Thought ID")
    ),
    responses(
        (status = 204, description = "Thought deleted"),
        (status = 403, description = "Forbidden", body = ApiErrorResponse),
        (status = 404, description = "Not Found", body = ApiErrorResponse)
    ),
    security(
        ("api_key" = []),
        ("bearer_auth" = [])
    )
)]
async fn thoughts_delete(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let thought = get_thought(&state.conn, id, Some(auth_user.id))
        .await?
        .ok_or(UserError::NotFound)?;

    if thought.author_id != auth_user.id {
        return Err(UserError::Forbidden.into());
    }

    delete_thought(&state.conn, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/{id}/thread",
    params(
        ("id" = Uuid, Path, description = "Thought ID")
    ),
    responses(
        (status = 200, description = "Thought thread found", body = ThoughtThreadSchema),
        (status = 404, description = "Not Found", body = ApiErrorResponse)
    )
)]
async fn get_thought_thread(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    viewer: OptionalAuthUser,
) -> Result<impl IntoResponse, ApiError> {
    let viewer_id = viewer.0.map(|u| u.id);
    let thread = app::persistence::thought::get_thought_with_replies(&state.conn, id, viewer_id)
        .await?
        .ok_or(UserError::NotFound)?;

    Ok(Json(thread))
}

pub fn create_thought_router() -> Router<AppState> {
    Router::new()
        .route("/", post(thoughts_post))
        .route("/{id}/thread", get(get_thought_thread))
        .route("/{id}", get(get_thought_by_id).delete(thoughts_delete))
}
