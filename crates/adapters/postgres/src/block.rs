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
mod tests {
    use super::*;
    use crate::test_helpers::seed_user;
    use chrono::Utc;
    use domain::value_objects::*;

    #[sqlx::test(migrations = "./migrations")]
    async fn block_exists(pool: sqlx::PgPool) {
        let alice = seed_user(&pool, "alice", "alice@ex.com").await;
        let bob = seed_user(&pool, "bob", "bob@ex.com").await;
        let repo = PgBlockRepository::new(pool);
        let block = Block {
            blocker_id: alice.id.clone(),
            blocked_id: bob.id.clone(),
            created_at: Utc::now(),
        };
        repo.save(&block).await.unwrap();
        assert!(repo.exists(&alice.id, &bob.id).await.unwrap());
        assert!(!repo.exists(&bob.id, &alice.id).await.unwrap());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn unblock(pool: sqlx::PgPool) {
        let alice = seed_user(&pool, "alice", "alice@ex.com").await;
        let bob = seed_user(&pool, "bob", "bob@ex.com").await;
        let repo = PgBlockRepository::new(pool);
        let block = Block {
            blocker_id: alice.id.clone(),
            blocked_id: bob.id.clone(),
            created_at: Utc::now(),
        };
        repo.save(&block).await.unwrap();
        repo.delete(&alice.id, &bob.id).await.unwrap();
        assert!(!repo.exists(&alice.id, &bob.id).await.unwrap());
    }
}
