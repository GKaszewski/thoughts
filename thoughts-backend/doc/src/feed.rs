use api::{models::ApiErrorResponse, routers::feed::*};
use models::schemas::thought::{ThoughtListSchema, ThoughtSchema};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(feed_get),
    components(schemas(ThoughtSchema, ThoughtListSchema, ApiErrorResponse))
)]
pub(super) struct FeedApi;
