use crate::db_error::IntoDbResult;
use async_trait::async_trait;
use domain::{
    errors::DomainError, models::actor_connection_summary::ActorConnectionSummary,
    ports::RemoteActorConnectionRepository,
};
use sqlx::PgPool;

pub struct PgRemoteActorConnectionRepository {
    pool: PgPool,
}

impl PgRemoteActorConnectionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RemoteActorConnectionRepository for PgRemoteActorConnectionRepository {
    async fn upsert_connections(
        &self,
        actor_url: &str,
        connection_type: &str,
        page: u32,
        actors: &[ActorConnectionSummary],
    ) -> Result<(), DomainError> {
        for actor in actors {
            sqlx::query(
                "INSERT INTO remote_actor_connections
                    (actor_url, connection_type, page, connected_actor_url,
                     connected_handle, connected_display_name, connected_avatar_url, fetched_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())
                 ON CONFLICT(actor_url, connection_type, page, connected_actor_url)
                 DO UPDATE SET
                     connected_handle       = EXCLUDED.connected_handle,
                     connected_display_name = EXCLUDED.connected_display_name,
                     connected_avatar_url   = EXCLUDED.connected_avatar_url,
                     fetched_at             = NOW()",
            )
            .bind(actor_url)
            .bind(connection_type)
            .bind(page as i32)
            .bind(&actor.url)
            .bind(&actor.handle)
            .bind(&actor.display_name)
            .bind(&actor.avatar_url)
            .execute(&self.pool)
            .await
            .into_domain()?;
        }
        Ok(())
    }

    async fn list_connections(
        &self,
        actor_url: &str,
        connection_type: &str,
        page: u32,
    ) -> Result<Vec<ActorConnectionSummary>, DomainError> {
        #[derive(sqlx::FromRow)]
        struct Row {
            connected_actor_url: String,
            connected_handle: String,
            connected_display_name: Option<String>,
            connected_avatar_url: Option<String>,
        }
        let rows = sqlx::query_as::<_, Row>(
            "SELECT connected_actor_url, connected_handle, connected_display_name, connected_avatar_url
             FROM remote_actor_connections
             WHERE actor_url = $1 AND connection_type = $2 AND page = $3
             ORDER BY connected_handle",
        )
        .bind(actor_url)
        .bind(connection_type)
        .bind(page as i32)
        .fetch_all(&self.pool)
        .await
        .into_domain()?;

        Ok(rows
            .into_iter()
            .map(|r| ActorConnectionSummary {
                url: r.connected_actor_url,
                handle: r.connected_handle,
                display_name: r.connected_display_name,
                avatar_url: r.connected_avatar_url,
            })
            .collect())
    }

    async fn connection_page_age(
        &self,
        actor_url: &str,
        connection_type: &str,
        page: u32,
    ) -> Result<Option<chrono::DateTime<chrono::Utc>>, DomainError> {
        let row: Option<(Option<chrono::DateTime<chrono::Utc>>,)> = sqlx::query_as(
            "SELECT MAX(fetched_at) FROM remote_actor_connections
             WHERE actor_url = $1 AND connection_type = $2 AND page = $3",
        )
        .bind(actor_url)
        .bind(connection_type)
        .bind(page as i32)
        .fetch_optional(&self.pool)
        .await
        .into_domain()?;

        Ok(row.and_then(|(ts,)| ts))
    }
}
