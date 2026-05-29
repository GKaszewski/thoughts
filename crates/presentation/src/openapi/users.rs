use api_types::{
    requests::UpdateProfileRequest,
    responses::{ErrorResponse, UserResponse},
};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::users::get_users,
        crate::handlers::users::get_user_count,
        crate::handlers::users::lookup_handler,
        crate::handlers::users::get_me,
        crate::handlers::users::get_me_following,
        crate::handlers::users::get_user,
        crate::handlers::users::patch_profile,
        crate::handlers::users::upload_avatar,
        crate::handlers::users::upload_banner,
    ),
    components(schemas(UserResponse, UpdateProfileRequest, ErrorResponse))
)]
pub struct UsersDoc;
