use crate::domains::api_key;
use common::DateTimeWithTimeZoneWrapper;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, ToSchema)]
pub struct ApiKeySchema {
    pub id: Uuid,
    pub name: String,
    #[serde(rename = "keyPrefix")]
    pub key_prefix: String,
    #[serde(rename = "createdAt")]
    pub created_at: DateTimeWithTimeZoneWrapper,
}

#[derive(Serialize, ToSchema)]
pub struct ApiKeyResponse {
    #[serde(flatten)]
    pub key: ApiKeySchema,
    #[serde(skip_serializing_if = "Option::is_none", rename = "plaintextKey")]
    pub plaintext_key: Option<String>,
}

impl ApiKeyResponse {
    pub fn from_parts(model: api_key::Model, plaintext_key: Option<String>) -> Self {
        Self {
            key: ApiKeySchema {
                id: model.id,
                name: model.name,
                key_prefix: model.key_prefix,
                created_at: model.created_at.into(),
            },
            plaintext_key,
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct ApiKeyListSchema {
    #[serde(rename = "apiKeys")]
    pub api_keys: Vec<ApiKeySchema>,
}

impl From<Vec<api_key::Model>> for ApiKeyListSchema {
    fn from(keys: Vec<api_key::Model>) -> Self {
        Self {
            api_keys: keys
                .into_iter()
                .map(|k| ApiKeySchema {
                    id: k.id,
                    name: k.name,
                    key_prefix: k.key_prefix,
                    created_at: k.created_at.into(),
                })
                .collect(),
        }
    }
}

#[derive(Deserialize, ToSchema)]
pub struct ApiKeyRequest {
    pub name: String,
}
