// in thoughts-backend/doc/src/tag.rs

use api::{models::ApiErrorResponse, routers::tag::*};
use models::schemas::thought::{ThoughtListSchema, ThoughtSchema};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(get_thoughts_by_tag, get_popular_tags),
    components(schemas(ThoughtSchema, ThoughtListSchema, ApiErrorResponse))
)]
pub(super) struct TagApi;
