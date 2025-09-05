use utoipa::OpenApi;

use api::models::{ApiErrorResponse, ParamsErrorResponse};
use api::routers::user::*;
use models::params::user::CreateUserParams;
use models::schemas::{
    thought::{ThoughtListSchema, ThoughtSchema},
    user::{UserListSchema, UserSchema},
};

#[derive(OpenApi)]
#[openapi(
    paths(
        users_get,
        users_id_get,
        user_thoughts_get,
        user_follow_post,
        user_follow_delete
    ),
    components(schemas(
        CreateUserParams,
        UserListSchema,
        UserSchema,
        ThoughtSchema,
        ThoughtListSchema,
        ApiErrorResponse,
        ParamsErrorResponse,
    ))
)]
pub(super) struct UserApi;
