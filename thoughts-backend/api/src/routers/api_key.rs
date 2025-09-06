use crate::{
    error::ApiError,
    extractor::{AuthUser, Json},
    models::ApiErrorResponse,
};
use app::{persistence::api_key, state::AppState};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get},
    Router,
};
use models::schemas::api_key::{ApiKeyListSchema, ApiKeyRequest, ApiKeyResponse};
use sea_orm::prelude::Uuid;

#[utoipa::path(
    get,
    path = "/me/api-keys",
    responses(
        (status = 200, description = "List of API keys", body = ApiKeyListSchema),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
    security(
        ("bearerAuth" = [])
    )
)]
async fn get_keys(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<impl IntoResponse, ApiError> {
    let keys = api_key::get_api_keys_for_user(&state.conn, auth_user.id).await?;
    Ok(Json(ApiKeyListSchema::from(keys)))
}

#[utoipa::path(
    post,
    path = "/me/api-keys",
    request_body = ApiKeyRequest,
    responses(
        (status = 201, description = "API key created", body = ApiKeyResponse),
        (status = 400, description = "Bad request", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 422, description = "Validation error", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
    security(
        ("bearerAuth" = [])
    )
)]
async fn create_key(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(params): Json<ApiKeyRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let (key_model, plaintext_key) =
        api_key::create_api_key(&state.conn, auth_user.id, params.name).await?;

    let response = ApiKeyResponse::from_parts(key_model, Some(plaintext_key));
    Ok((StatusCode::CREATED, Json(response)))
}

#[utoipa::path(
    delete,
    path = "/me/api-keys/{key_id}",
    responses(
        (status = 204, description = "API key deleted"),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 404, description = "API key not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
    params(
        ("key_id" = Uuid, Path, description = "The ID of the API key to delete")
    ),
    security(
        ("bearerAuth" = [])
    )
)]
async fn delete_key(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(key_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    api_key::delete_api_key(&state.conn, key_id, auth_user.id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub fn create_api_key_router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_keys).post(create_key))
        .route("/{key_id}", delete(delete_key))
}
