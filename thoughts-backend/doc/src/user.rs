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
        get_user_by_param,
        user_thoughts_get,
        user_follow_post,
        user_follow_delete,
        user_inbox_post,
        user_outbox_get,
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
