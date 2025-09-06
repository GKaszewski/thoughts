use utoipa::OpenApi;

use api::models::{ApiErrorResponse, ParamsErrorResponse};
use api::routers::friends::*;
use models::schemas::user::{UserListSchema, UserSchema};

#[derive(OpenApi)]
#[openapi(
    paths(get_friends_list,),
    components(schemas(UserListSchema, ApiErrorResponse, ParamsErrorResponse, UserSchema))
)]
pub(super) struct FriendsApi;
