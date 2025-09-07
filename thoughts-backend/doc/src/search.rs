use api::{models::ApiErrorResponse, routers::search::*};
use models::schemas::{
    search::SearchResultsSchema,
    thought::{ThoughtListSchema, ThoughtSchema},
    user::{UserListSchema, UserSchema},
};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(search_all),
    components(schemas(
        SearchResultsSchema,
        ApiErrorResponse,
        ThoughtSchema,
        ThoughtListSchema,
        UserSchema,
        UserListSchema
    ))
)]
pub(super) struct SearchApi;
