use api::{
    models::{ApiErrorResponse, ParamsErrorResponse},
    routers::thought::*,
};
use models::{params::thought::CreateThoughtParams, schemas::thought::ThoughtSchema};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(thoughts_post, thoughts_delete, get_thought_by_id),
    components(schemas(
        CreateThoughtParams,
        ThoughtSchema,
        ApiErrorResponse,
        ParamsErrorResponse
    ))
)]
pub(super) struct ThoughtApi;
