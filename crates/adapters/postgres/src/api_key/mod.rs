use crate::db_error::IntoDbResult;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::{
    errors::DomainError,
    models::api_key::ApiKey,
    ports::ApiKeyRepository,
    value_objects::{ApiKeyId, UserId},
};
use sqlx::PgPool;

pub struct PgApiKeyRepository {
    pool: PgPool,
}
impl PgApiKeyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ApiKeyRepository for PgApiKeyRepository {
    async fn save(&self, k: &ApiKey) -> Result<(), DomainError> {
        sqlx::query(
            "INSERT INTO api_keys(id,user_id,key_hash,name,created_at) VALUES($1,$2,$3,$4,$5)",
        )
        .bind(k.id.as_uuid())
        .bind(k.user_id.as_uuid())
        .bind(&k.key_hash)
        .bind(&k.name)
        .bind(k.created_at)
        .execute(&self.pool)
        .await
        .into_domain()
        .map(|_| ())
    }

    async fn find_by_hash(&self, hash: &str) -> Result<Option<ApiKey>, DomainError> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: uuid::Uuid,
            user_id: uuid::Uuid,
            key_hash: String,
            name: String,
            created_at: DateTime<Utc>,
        }
        sqlx::query_as::<_, Row>(
            "SELECT id,user_id,key_hash,name,created_at FROM api_keys WHERE key_hash=$1",
        )
        .bind(hash)
        .fetch_optional(&self.pool)
        .await
        .into_domain()
        .map(|o| {
            o.map(|r| ApiKey {
                id: ApiKeyId::from_uuid(r.id),
                user_id: UserId::from_uuid(r.user_id),
                key_hash: r.key_hash,
                name: r.name,
                created_at: r.created_at,
            })
        })
    }

    async fn list_for_user(&self, user_id: &UserId) -> Result<Vec<ApiKey>, DomainError> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: uuid::Uuid,
            user_id: uuid::Uuid,
            key_hash: String,
            name: String,
            created_at: DateTime<Utc>,
        }
        sqlx::query_as::<_, Row>("SELECT id,user_id,key_hash,name,created_at FROM api_keys WHERE user_id=$1 ORDER BY created_at DESC")
            .bind(user_id.as_uuid()).fetch_all(&self.pool).await
            .into_domain()
            .map(|rows| rows.into_iter().map(|r| ApiKey { id: ApiKeyId::from_uuid(r.id), user_id: UserId::from_uuid(r.user_id), key_hash: r.key_hash, name: r.name, created_at: r.created_at }).collect())
    }

    async fn delete(&self, id: &ApiKeyId, user_id: &UserId) -> Result<(), DomainError> {
        sqlx::query("DELETE FROM api_keys WHERE id=$1 AND user_id=$2")
            .bind(id.as_uuid())
            .bind(user_id.as_uuid())
            .execute(&self.pool)
            .await
            .into_domain()
            .map(|_| ())
    }
}

#[cfg(test)]
mod tests;
