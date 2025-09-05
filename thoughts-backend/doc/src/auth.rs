use api::{
    models::{ApiErrorResponse, ParamsErrorResponse},
    routers::auth::*,
};
use models::{
    params::auth::{LoginParams, RegisterParams},
    schemas::user::UserSchema,
};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(register, login),
    components(schemas(
        RegisterParams,
        LoginParams,
        UserSchema,
        TokenResponse,
        ApiErrorResponse,
        ParamsErrorResponse,
    ))
)]
pub(super) struct AuthApi;
