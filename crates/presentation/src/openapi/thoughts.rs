use api_types::{
    requests::{CreateThoughtRequest, EditThoughtRequest},
    responses::ErrorResponse,
};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::thoughts::post_thought,
        crate::handlers::thoughts::get_thought_handler,
        crate::handlers::thoughts::patch_thought,
        crate::handlers::thoughts::delete_thought_handler,
        crate::handlers::thoughts::get_thread_handler,
    ),
    components(schemas(CreateThoughtRequest, EditThoughtRequest, ErrorResponse))
)]
pub struct ThoughtsDoc;
