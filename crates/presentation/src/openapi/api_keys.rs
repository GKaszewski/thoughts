use api_types::{
    requests::CreateApiKeyRequest,
    responses::{ApiKeyResponse, CreatedApiKeyResponse},
};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::api_keys::get_api_keys,
        crate::handlers::api_keys::post_api_key,
        crate::handlers::api_keys::delete_api_key_handler,
    ),
    components(schemas(CreateApiKeyRequest, ApiKeyResponse, CreatedApiKeyResponse))
)]
pub struct ApiKeysDoc;
