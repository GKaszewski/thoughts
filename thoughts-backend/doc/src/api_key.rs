use api::{models::ApiErrorResponse, routers::api_key::*};
use models::schemas::api_key::{ApiKeyListSchema, ApiKeyRequest, ApiKeyResponse, ApiKeySchema};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(get_keys, create_key, delete_key),
    components(schemas(
        ApiKeySchema,
        ApiKeyListSchema,
        ApiKeyRequest,
        ApiKeyResponse,
        ApiErrorResponse,
    ))
)]
pub(super) struct ApiKeyApi;
