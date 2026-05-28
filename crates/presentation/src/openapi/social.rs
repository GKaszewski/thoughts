use api_types::requests::SetTopFriendsRequest;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::social::post_like,
        crate::handlers::social::delete_like,
        crate::handlers::social::post_boost,
        crate::handlers::social::delete_boost,
        crate::handlers::social::post_follow,
        crate::handlers::social::delete_follow,
        crate::handlers::social::post_block,
        crate::handlers::social::delete_block,
        crate::handlers::social::put_top_friends,
        crate::handlers::social::get_top_friends_handler,
        crate::handlers::social::get_friends_handler,
    ),
    components(schemas(SetTopFriendsRequest))
)]
pub struct SocialDoc;
