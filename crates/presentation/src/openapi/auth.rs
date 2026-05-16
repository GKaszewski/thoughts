use api_types::{
    requests::{LoginRequest, RegisterRequest},
    responses::{AuthResponse, ErrorResponse},
};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::auth::post_register,
        crate::handlers::auth::post_login
    ),
    components(schemas(RegisterRequest, LoginRequest, AuthResponse, ErrorResponse))
)]
pub struct AuthDoc;
