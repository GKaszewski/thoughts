use crate::value_objects::{ApiKeyId, UserId};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct ApiKey {
    pub id: ApiKeyId,
    pub user_id: UserId,
    pub key_hash: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
}
