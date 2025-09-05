use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, post},
    Router,
};

use app::{
    error::UserError,
    persistence::thought::{create_thought, delete_thought, get_thought},
    state::AppState,
};
use models::{params::thought::CreateThoughtParams, schemas::thought::ThoughtSchema};

use crate::{
    error::ApiError,
    extractor::{AuthUser, Json, Valid},
    federation,
    models::{ApiErrorResponse, ParamsErrorResponse},
};

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

    // Spawn a background task to handle federation without blocking the response
    tokio::spawn(federation::federate_thought(
        state.clone(),
        thought.clone(),
        author.clone(),
    ));

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
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApiError> {
    let thought = get_thought(&state.conn, id)
        .await?
        .ok_or(UserError::NotFound)?;

    if thought.author_id != auth_user.id {
        return Err(UserError::Forbidden.into());
    }

    delete_thought(&state.conn, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub fn create_thought_router() -> Router<AppState> {
    Router::new()
        .route("/", post(thoughts_post))
        .route("/{id}", delete(thoughts_delete))
}
