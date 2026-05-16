use crate::{
    deps_struct,
    errors::ApiError,
    extractors::Deps,
};
use api_types::{
    requests::{LoginRequest, RegisterRequest},
    responses::{AuthResponse, ErrorResponse, UserResponse},
};
use application::use_cases::auth::{login, register, LoginInput, RegisterInput};
use axum::{http::StatusCode, response::IntoResponse, Json};
use domain::ports::{AuthService, EventPublisher, PasswordHasher, UserRepository};

deps_struct!(AuthDeps {
    users: UserRepository,
    hasher: PasswordHasher,
    auth: AuthService,
    events: EventPublisher,
});

pub fn to_user_response(u: &domain::models::user::User) -> UserResponse {
    UserResponse {
        id: u.id.as_uuid(),
        username: u.username.to_string(),
        display_name: u.display_name.clone(),
        bio: u.bio.clone(),
        avatar_url: u.avatar_url.clone(),
        header_url: u.header_url.clone(),
        custom_css: u.custom_css.clone(),
        local: u.local,
        is_followed_by_viewer: false,
        created_at: u.created_at,
    }
}

#[utoipa::path(
    post, path = "/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User registered", body = AuthResponse),
        (status = 409, description = "Username or email taken", body = ErrorResponse),
        (status = 422, description = "Invalid input", body = ErrorResponse),
    )
)]
pub async fn post_register(
    Deps(d): Deps<AuthDeps>,
    Json(body): Json<RegisterRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let out = register(
        &*d.users,
        &*d.hasher,
        &*d.auth,
        &*d.events,
        RegisterInput {
            username: body.username,
            email: body.email,
            password: body.password,
        },
    )
    .await?;
    let resp = AuthResponse {
        token: out.token,
        user: to_user_response(&out.user),
    };
    Ok((StatusCode::CREATED, Json(resp)))
}

#[utoipa::path(
    post, path = "/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 401, description = "Invalid credentials", body = ErrorResponse),
    )
)]
pub async fn post_login(
    Deps(d): Deps<AuthDeps>,
    Json(body): Json<LoginRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let out = login(
        &*d.users,
        &*d.hasher,
        &*d.auth,
        LoginInput {
            email: body.email,
            password: body.password,
        },
    )
    .await?;
    Ok(Json(AuthResponse {
        token: out.token,
        user: to_user_response(&out.user),
    }))
}
