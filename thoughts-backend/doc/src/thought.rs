use api::{
    models::{ApiErrorResponse, ParamsErrorResponse},
    routers::thought::*,
};
use models::{
    params::thought::CreateThoughtParams,
    schemas::thought::{ThoughtSchema, ThoughtThreadSchema},
};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(thoughts_post, thoughts_delete, get_thought_by_id, get_thought_thread),
    components(schemas(
        CreateThoughtParams,
        ThoughtSchema,
        ThoughtThreadSchema,
        ApiErrorResponse,
        ParamsErrorResponse
    ))
)]
pub(super) struct ThoughtApi;
