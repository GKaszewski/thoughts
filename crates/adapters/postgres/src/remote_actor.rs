use crate::db_error::IntoDbResult;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::{
    errors::DomainError, models::remote_actor::RemoteActor, ports::RemoteActorRepository,
};
use sqlx::PgPool;

pub struct PgRemoteActorRepository {
    pool: PgPool,
}
impl PgRemoteActorRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RemoteActorRepository for PgRemoteActorRepository {
    async fn upsert(&self, a: &RemoteActor) -> Result<(), DomainError> {
        sqlx::query(
            "INSERT INTO remote_actors(url,handle,display_name,avatar_url,last_fetched_at)
             VALUES($1,$2,$3,$4,$5)
             ON CONFLICT(url) DO UPDATE SET handle=EXCLUDED.handle,display_name=EXCLUDED.display_name,
             avatar_url=EXCLUDED.avatar_url,last_fetched_at=EXCLUDED.last_fetched_at"
        )
        .bind(&a.url).bind(&a.handle).bind(&a.display_name).bind(&a.avatar_url).bind(a.last_fetched_at)
        .execute(&self.pool).await.into_domain().map(|_| ())
    }

    async fn find_by_url(&self, url: &str) -> Result<Option<RemoteActor>, DomainError> {
        #[derive(sqlx::FromRow)]
        struct Row {
            url: String,
            handle: String,
            display_name: Option<String>,
            avatar_url: Option<String>,
            last_fetched_at: DateTime<Utc>,
        }
        sqlx::query_as::<_, Row>(
            "SELECT url,handle,display_name,avatar_url,last_fetched_at FROM remote_actors WHERE url=$1"
        ).bind(url).fetch_optional(&self.pool).await
        .into_domain()
        .map(|o| o.map(|r| RemoteActor {
            url: r.url,
            handle: r.handle,
            display_name: r.display_name,
            avatar_url: r.avatar_url,
            last_fetched_at: r.last_fetched_at,
            bio: None,
            banner_url: None,
            also_known_as: None,
            outbox_url: None,
            followers_url: None,
            following_url: None,
            attachment: vec![],
        }))
    }
}
