use crate::db_error::IntoDbResult;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

use domain::{
    errors::DomainError,
    models::{
        feed::{PageParams, Paginated},
        social::{Follow, FollowState},
        user::User,
    },
    ports::FollowRepository,
    value_objects::UserId,
};
use sqlx::PgPool;

pub struct PgFollowRepository {
    pool: PgPool,
}
impl PgFollowRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl FollowRepository for PgFollowRepository {
    async fn save(&self, f: &Follow) -> Result<(), DomainError> {
        sqlx::query(
            "INSERT INTO follows(follower_id,following_id,state,ap_id,created_at)
             VALUES($1,$2,$3,$4,$5)
             ON CONFLICT(follower_id,following_id) DO UPDATE SET state=EXCLUDED.state,ap_id=EXCLUDED.ap_id"
        )
        .bind(f.follower_id.as_uuid())
        .bind(f.following_id.as_uuid())
        .bind(f.state.as_str())
        .bind(&f.ap_id)
        .bind(f.created_at)
        .execute(&self.pool)
        .await
        .into_domain()
        .map(|_| ())
    }

    async fn delete(&self, follower_id: &UserId, following_id: &UserId) -> Result<(), DomainError> {
        let r = sqlx::query("DELETE FROM follows WHERE follower_id=$1 AND following_id=$2")
            .bind(follower_id.as_uuid())
            .bind(following_id.as_uuid())
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
        follower_id: &UserId,
        following_id: &UserId,
    ) -> Result<Option<Follow>, DomainError> {
        #[derive(sqlx::FromRow)]
        struct Row {
            follower_id: uuid::Uuid,
            following_id: uuid::Uuid,
            state: String,
            ap_id: Option<String>,
            created_at: DateTime<Utc>,
        }
        sqlx::query_as::<_, Row>(
            "SELECT follower_id,following_id,state,ap_id,created_at FROM follows WHERE follower_id=$1 AND following_id=$2"
        )
        .bind(follower_id.as_uuid())
        .bind(following_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .into_domain()
        .and_then(|o| {
            o.map(|r| {
                Ok(Follow {
                    follower_id: UserId::from_uuid(r.follower_id),
                    following_id: UserId::from_uuid(r.following_id),
                    state: FollowState::from_db_str(&r.state)?,
                    ap_id: r.ap_id,
                    created_at: r.created_at,
                })
            })
            .transpose()
        })
    }

    async fn update_state(
        &self,
        follower_id: &UserId,
        following_id: &UserId,
        state: &FollowState,
    ) -> Result<(), DomainError> {
        sqlx::query("UPDATE follows SET state=$3 WHERE follower_id=$1 AND following_id=$2")
            .bind(follower_id.as_uuid())
            .bind(following_id.as_uuid())
            .bind(state.as_str())
            .execute(&self.pool)
            .await
            .into_domain()
            .map(|_| ())
    }

    async fn list_followers(
        &self,
        user_id: &UserId,
        page: &PageParams,
    ) -> Result<Paginated<User>, DomainError> {
        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM follows WHERE following_id=$1 AND state='accepted'",
        )
        .bind(user_id.as_uuid())
        .fetch_one(&self.pool)
        .await
        .into_domain()?;

        let rows = sqlx::query_as::<_, crate::user::UserRow>(
            "SELECT u.id,u.username,u.email,u.password_hash,u.display_name,u.bio,u.avatar_url,u.header_url,u.custom_css,u.local,u.ap_id,u.inbox_url,u.created_at,u.updated_at
             FROM users u JOIN follows f ON f.follower_id=u.id
             WHERE f.following_id=$1 AND f.state='accepted'
             ORDER BY f.created_at DESC LIMIT $2 OFFSET $3"
        )
        .bind(user_id.as_uuid())
        .bind(page.limit())
        .bind(page.offset())
        .fetch_all(&self.pool)
        .await
        .into_domain()?;

        Ok(Paginated {
            items: rows.into_iter().map(User::from).collect(),
            total,
            page: page.page,
            per_page: page.per_page,
        })
    }

    async fn list_following(
        &self,
        user_id: &UserId,
        page: &PageParams,
    ) -> Result<Paginated<User>, DomainError> {
        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM follows WHERE follower_id=$1 AND state='accepted'",
        )
        .bind(user_id.as_uuid())
        .fetch_one(&self.pool)
        .await
        .into_domain()?;

        let rows = sqlx::query_as::<_, crate::user::UserRow>(
            "SELECT u.id,u.username,u.email,u.password_hash,u.display_name,u.bio,u.avatar_url,u.header_url,u.custom_css,u.local,u.ap_id,u.inbox_url,u.created_at,u.updated_at
             FROM users u JOIN follows f ON f.following_id=u.id
             WHERE f.follower_id=$1 AND f.state='accepted'
             ORDER BY f.created_at DESC LIMIT $2 OFFSET $3"
        )
        .bind(user_id.as_uuid())
        .bind(page.limit())
        .bind(page.offset())
        .fetch_all(&self.pool)
        .await
        .into_domain()?;

        Ok(Paginated {
            items: rows.into_iter().map(User::from).collect(),
            total,
            page: page.page,
            per_page: page.per_page,
        })
    }

    async fn get_accepted_following_ids(
        &self,
        user_id: &UserId,
    ) -> Result<Vec<UserId>, DomainError> {
        let ids: Vec<uuid::Uuid> = sqlx::query_scalar(
            "SELECT following_id FROM follows WHERE follower_id=$1 AND state='accepted'",
        )
        .bind(user_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .into_domain()?;
        Ok(ids.into_iter().map(UserId::from_uuid).collect())
    }

    async fn list_mutual(
        &self,
        user_id: &UserId,
        page: &PageParams,
    ) -> Result<Paginated<User>, DomainError> {
        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(DISTINCT u.id)
             FROM users u
             WHERE EXISTS (
               SELECT 1 FROM follows f1 WHERE f1.follower_id=$1 AND f1.following_id=u.id AND f1.state='accepted'
             ) AND EXISTS (
               SELECT 1 FROM follows f2 WHERE f2.following_id=$1 AND f2.follower_id=u.id AND f2.state='accepted'
             )",
        )
        .bind(user_id.as_uuid())
        .fetch_one(&self.pool)
        .await
        .into_domain()?;

        let rows = sqlx::query_as::<_, crate::user::UserRow>(
            "SELECT u.id,u.username,u.email,u.password_hash,u.display_name,u.bio,u.avatar_url,u.header_url,u.custom_css,u.local,u.ap_id,u.inbox_url,u.created_at,u.updated_at
             FROM users u
             WHERE EXISTS (
               SELECT 1 FROM follows f1 WHERE f1.follower_id=$1 AND f1.following_id=u.id AND f1.state='accepted'
             ) AND EXISTS (
               SELECT 1 FROM follows f2 WHERE f2.following_id=$1 AND f2.follower_id=u.id AND f2.state='accepted'
             )
             ORDER BY u.created_at DESC LIMIT $2 OFFSET $3"
        )
        .bind(user_id.as_uuid())
        .bind(page.limit())
        .bind(page.offset())
        .fetch_all(&self.pool)
        .await
        .into_domain()?;

        Ok(Paginated {
            items: rows.into_iter().map(User::from).collect(),
            total,
            page: page.page,
            per_page: page.per_page,
        })
    }
}

#[cfg(test)]
mod tests;
