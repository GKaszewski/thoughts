use crate::db_error::IntoDbResult;
use async_trait::async_trait;
use domain::{
    errors::DomainError, models::social::Block, ports::BlockRepository, value_objects::UserId,
};
use sqlx::PgPool;

pub struct PgBlockRepository {
    pool: PgPool,
}
impl PgBlockRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl BlockRepository for PgBlockRepository {
    async fn save(&self, b: &Block) -> Result<(), DomainError> {
        sqlx::query(
            "INSERT INTO blocks(blocker_id,blocked_id,created_at) VALUES($1,$2,$3) ON CONFLICT DO NOTHING"
        )
        .bind(b.blocker_id.as_uuid())
        .bind(b.blocked_id.as_uuid())
        .bind(b.created_at)
        .execute(&self.pool)
        .await
        .into_domain()
        .map(|_| ())
    }

    async fn delete(&self, blocker_id: &UserId, blocked_id: &UserId) -> Result<(), DomainError> {
        sqlx::query("DELETE FROM blocks WHERE blocker_id=$1 AND blocked_id=$2")
            .bind(blocker_id.as_uuid())
            .bind(blocked_id.as_uuid())
            .execute(&self.pool)
            .await
            .into_domain()
            .map(|_| ())
    }

    async fn exists(&self, blocker_id: &UserId, blocked_id: &UserId) -> Result<bool, DomainError> {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM blocks WHERE blocker_id=$1 AND blocked_id=$2")
                .bind(blocker_id.as_uuid())
                .bind(blocked_id.as_uuid())
                .fetch_one(&self.pool)
                .await
                .into_domain()?;
        Ok(count > 0)
    }
}

#[cfg(test)]
mod tests;
