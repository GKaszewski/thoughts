use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(
    crate::handlers::feed::home_feed,
    crate::handlers::feed::public_feed,
    crate::handlers::feed::search_handler,
    crate::handlers::feed::user_thoughts_handler,
    crate::handlers::feed::get_followers_handler,
    crate::handlers::feed::get_following_handler,
    crate::handlers::feed::tag_thoughts_handler,
    crate::handlers::feed::get_popular_tags,
))]
pub struct FeedDoc;
