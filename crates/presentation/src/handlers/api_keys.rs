use crate::{
    deps_struct,
    errors::ApiError,
    extractors::{AuthUser, Deps},
};
use api_types::{
    requests::CreateApiKeyRequest,
    responses::{ApiKeyResponse, CreatedApiKeyResponse},
};
use application::use_cases::api_keys::{create_api_key, delete_api_key, list_api_keys};
use axum::{extract::Path, http::StatusCode, Json};
use domain::{ports::ApiKeyRepository, value_objects::ApiKeyId};
use uuid::Uuid;

deps_struct!(ApiKeysDeps {
    api_keys: ApiKeyRepository,
});

#[utoipa::path(get, path = "/api-keys", responses((status = 200, description = "API keys", body = Vec<ApiKeyResponse>)), security(("bearer_auth" = [])))]
pub async fn get_api_keys(
    Deps(d): Deps<ApiKeysDeps>,
    AuthUser(uid): AuthUser,
) -> Result<Json<Vec<ApiKeyResponse>>, ApiError> {
    let keys = list_api_keys(&*d.api_keys, &uid).await?;
    Ok(Json(
        keys.into_iter()
            .map(|k| ApiKeyResponse {
                id: k.id.as_uuid(),
                name: k.name,
                created_at: k.created_at,
            })
            .collect(),
    ))
}
#[utoipa::path(post, path = "/api-keys", request_body = CreateApiKeyRequest, responses((status = 200, description = "Created — raw key shown once", body = CreatedApiKeyResponse)), security(("bearer_auth" = [])))]
pub async fn post_api_key(
    Deps(d): Deps<ApiKeysDeps>,
    AuthUser(uid): AuthUser,
    Json(body): Json<CreateApiKeyRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let (key, raw) = create_api_key(&*d.api_keys, &uid, body.name).await?;
    Ok(Json(
        serde_json::json!({ "id": key.id.as_uuid(), "name": key.name, "key": raw }),
    ))
}
#[utoipa::path(delete, path = "/api-keys/{id}", params(("id" = uuid::Uuid, Path, description = "Key ID")), responses((status = 204, description = "Deleted")), security(("bearer_auth" = [])))]
pub async fn delete_api_key_handler(
    Deps(d): Deps<ApiKeysDeps>,
    AuthUser(uid): AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    delete_api_key(&*d.api_keys, &uid, &ApiKeyId::from_uuid(id)).await?;
    Ok(StatusCode::NO_CONTENT)
}
