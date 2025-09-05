use axum::{
    debug_handler, extract::State, http::StatusCode, response::IntoResponse, routing::post, Router,
};
use jsonwebtoken::{encode, EncodingKey, Header};
use once_cell::sync::Lazy;
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};
use utoipa::ToSchema;

use crate::{
    error::ApiError,
    extractor::{Claims, Json, Valid},
    models::{ApiErrorResponse, ParamsErrorResponse},
};
use app::{persistence::auth, state::AppState};
use models::{
    params::auth::{LoginParams, RegisterParams},
    schemas::user::UserSchema,
};

static JWT_SECRET: Lazy<String> =
    Lazy::new(|| std::env::var("AUTH_SECRET").expect("AUTH_SECRET must be set"));

#[derive(Serialize, ToSchema)]
pub struct TokenResponse {
    token: String,
}

#[utoipa::path(
    post,
    path = "/register",
    request_body = RegisterParams,
    responses(
        (status = 201, description = "User registered", body = UserSchema),
        (status = 400, description = "Bad request", body = ApiErrorResponse),
        (status = 409, description = "Username already exists", body = ApiErrorResponse),
        (status = 422, description = "Validation error", body = ParamsErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
#[axum::debug_handler]
async fn register(
    State(state): State<AppState>,
    Valid(Json(params)): Valid<Json<RegisterParams>>,
) -> Result<impl IntoResponse, ApiError> {
    let user = auth::register_user(&state.conn, params).await?;
    Ok((StatusCode::CREATED, Json(UserSchema::from(user))))
}

#[utoipa::path(
    post,
    path = "/login",
    request_body = LoginParams,
    responses(
        (status = 200, description = "User logged in", body = TokenResponse),
        (status = 400, description = "Bad request", body = ApiErrorResponse),
        (status = 401, description = "Invalid credentials", body = ApiErrorResponse),
        (status = 422, description = "Validation error", body = ParamsErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
#[debug_handler]
async fn login(
    state: State<AppState>,
    Valid(Json(params)): Valid<Json<LoginParams>>,
) -> Result<impl IntoResponse, ApiError> {
    let user = auth::authenticate_user(&state.conn, params).await?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let claims = Claims {
        sub: user.id,
        exp: (now + 3600 * 24) as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET.as_ref()),
    )
    .map_err(|e| ApiError::from(app::error::UserError::Internal(e.to_string())))?;

    Ok((StatusCode::OK, Json(TokenResponse { token })))
}

pub fn create_auth_router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
}
