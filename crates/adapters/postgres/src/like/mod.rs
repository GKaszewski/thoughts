use crate::db_error::IntoDbResult;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::{
    errors::DomainError,
    models::social::Like,
    ports::LikeRepository,
    value_objects::{LikeId, ThoughtId, UserId},
};
use sqlx::PgPool;

pub struct PgLikeRepository {
    pool: PgPool,
}
impl PgLikeRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl LikeRepository for PgLikeRepository {
    async fn save(&self, l: &Like) -> Result<(), DomainError> {
        sqlx::query(
            "INSERT INTO likes(id,user_id,thought_id,ap_id,created_at) VALUES($1,$2,$3,$4,$5) ON CONFLICT(user_id,thought_id) DO NOTHING"
        )
        .bind(l.id.as_uuid()).bind(l.user_id.as_uuid()).bind(l.thought_id.as_uuid()).bind(&l.ap_id).bind(l.created_at)
        .execute(&self.pool).await.into_domain().map(|_| ())
    }

    async fn delete(&self, user_id: &UserId, thought_id: &ThoughtId) -> Result<(), DomainError> {
        let r = sqlx::query("DELETE FROM likes WHERE user_id=$1 AND thought_id=$2")
            .bind(user_id.as_uuid())
            .bind(thought_id.as_uuid())
            .execute(&self.pool)
            .await
            .into_domain()?;
        if r.rows_affected() == 0 {
            return Err(DomainError::NotFound);
        }
        Ok(())
    }

    async fn find(
        &self,
        user_id: &UserId,
        thought_id: &ThoughtId,
    ) -> Result<Option<Like>, DomainError> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: uuid::Uuid,
            user_id: uuid::Uuid,
            thought_id: uuid::Uuid,
            ap_id: Option<String>,
            created_at: DateTime<Utc>,
        }
        sqlx::query_as::<_, Row>("SELECT id,user_id,thought_id,ap_id,created_at FROM likes WHERE user_id=$1 AND thought_id=$2")
            .bind(user_id.as_uuid()).bind(thought_id.as_uuid())
            .fetch_optional(&self.pool).await
            .into_domain()
            .map(|o| o.map(|r| Like { id: LikeId::from_uuid(r.id), user_id: UserId::from_uuid(r.user_id), thought_id: ThoughtId::from_uuid(r.thought_id), ap_id: r.ap_id, created_at: r.created_at }))
    }

    async fn count_for_thought(&self, thought_id: &ThoughtId) -> Result<i64, DomainError> {
        sqlx::query_scalar("SELECT COUNT(*) FROM likes WHERE thought_id=$1")
            .bind(thought_id.as_uuid())
            .fetch_one(&self.pool)
            .await
            .into_domain()
    }
}

#[cfg(test)]
mod tests;
