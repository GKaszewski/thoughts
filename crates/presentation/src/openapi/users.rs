use api_types::{
    requests::UpdateProfileRequest,
    responses::{ErrorResponse, UserResponse},
};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::users::get_me,
        crate::handlers::users::get_user,
        crate::handlers::users::patch_profile,
    ),
    components(schemas(UserResponse, UpdateProfileRequest, ErrorResponse))
)]
pub struct UsersDoc;
