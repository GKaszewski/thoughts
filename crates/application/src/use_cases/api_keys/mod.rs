use chrono::Utc;
use domain::{
    errors::DomainError,
    models::api_key::ApiKey,
    ports::ApiKeyRepository,
    value_objects::{ApiKeyId, UserId},
};

pub async fn list_api_keys(
    keys: &dyn ApiKeyRepository,
    user_id: &UserId,
) -> Result<Vec<ApiKey>, DomainError> {
    keys.list_for_user(user_id).await
}

pub async fn create_api_key(
    keys: &dyn ApiKeyRepository,
    user_id: &UserId,
    name: String,
) -> Result<(ApiKey, String), DomainError> {
    let raw_key = uuid::Uuid::new_v4().to_string().replace('-', "");
    let key_hash = sha256_hex(&raw_key);
    let key = ApiKey {
        id: ApiKeyId::new(),
        user_id: user_id.clone(),
        key_hash,
        name,
        created_at: Utc::now(),
    };
    keys.save(&key).await?;
    Ok((key, raw_key))
}

pub async fn delete_api_key(
    keys: &dyn ApiKeyRepository,
    user_id: &UserId,
    key_id: &ApiKeyId,
) -> Result<(), DomainError> {
    keys.delete(key_id, user_id).await
}

fn sha256_hex(s: &str) -> String {
    use sha2::{Digest, Sha256};
    let hash = Sha256::digest(s.as_bytes());
    hex::encode(hash)
}

#[cfg(test)]
mod tests;
