use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(
    crate::handlers::feed::home_feed,
    crate::handlers::feed::public_feed,
    crate::handlers::feed::search_handler,
    crate::handlers::feed::user_thoughts_handler,
    crate::handlers::feed::tag_thoughts_handler,
))]
pub struct FeedDoc;
