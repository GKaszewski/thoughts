use crate::db_error::IntoDbResult;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::{
    errors::DomainError,
    models::social::Boost,
    ports::BoostRepository,
    value_objects::{BoostId, ThoughtId, UserId},
};
use sqlx::PgPool;

pub struct PgBoostRepository {
    pool: PgPool,
}
impl PgBoostRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl BoostRepository for PgBoostRepository {
    async fn save(&self, b: &Boost) -> Result<(), DomainError> {
        sqlx::query(
            "INSERT INTO boosts(id,user_id,thought_id,ap_id,created_at) VALUES($1,$2,$3,$4,$5) ON CONFLICT(user_id,thought_id) DO NOTHING"
        )
        .bind(b.id.as_uuid()).bind(b.user_id.as_uuid()).bind(b.thought_id.as_uuid()).bind(&b.ap_id).bind(b.created_at)
        .execute(&self.pool).await.into_domain().map(|_| ())
    }

    async fn delete(&self, user_id: &UserId, thought_id: &ThoughtId) -> Result<(), DomainError> {
        let r = sqlx::query("DELETE FROM boosts WHERE user_id=$1 AND thought_id=$2")
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
    ) -> Result<Option<Boost>, DomainError> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: uuid::Uuid,
            user_id: uuid::Uuid,
            thought_id: uuid::Uuid,
            ap_id: Option<String>,
            created_at: DateTime<Utc>,
        }
        sqlx::query_as::<_, Row>("SELECT id,user_id,thought_id,ap_id,created_at FROM boosts WHERE user_id=$1 AND thought_id=$2")
            .bind(user_id.as_uuid()).bind(thought_id.as_uuid())
            .fetch_optional(&self.pool).await
            .into_domain()
            .map(|o| o.map(|r| Boost { id: BoostId::from_uuid(r.id), user_id: UserId::from_uuid(r.user_id), thought_id: ThoughtId::from_uuid(r.thought_id), ap_id: r.ap_id, created_at: r.created_at }))
    }

    async fn count_for_thought(&self, thought_id: &ThoughtId) -> Result<i64, DomainError> {
        sqlx::query_scalar("SELECT COUNT(*) FROM boosts WHERE thought_id=$1")
            .bind(thought_id.as_uuid())
            .fetch_one(&self.pool)
            .await
            .into_domain()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::seed_user_and_thought;
    use chrono::Utc;
    use domain::value_objects::*;

    #[sqlx::test(migrations = "./migrations")]
    async fn boost_and_count(pool: sqlx::PgPool) {
        let (user, thought) = seed_user_and_thought(&pool).await;
        let repo = PgBoostRepository::new(pool);
        let boost = Boost {
            id: BoostId::new(),
            user_id: user.id.clone(),
            thought_id: thought.id.clone(),
            ap_id: None,
            created_at: Utc::now(),
        };
        repo.save(&boost).await.unwrap();
        assert_eq!(repo.count_for_thought(&thought.id).await.unwrap(), 1);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn unboost(pool: sqlx::PgPool) {
        let (user, thought) = seed_user_and_thought(&pool).await;
        let repo = PgBoostRepository::new(pool);
        let boost = Boost {
            id: BoostId::new(),
            user_id: user.id.clone(),
            thought_id: thought.id.clone(),
            ap_id: None,
            created_at: Utc::now(),
        };
        repo.save(&boost).await.unwrap();
        repo.delete(&user.id, &thought.id).await.unwrap();
        assert_eq!(repo.count_for_thought(&thought.id).await.unwrap(), 0);
    }
}
