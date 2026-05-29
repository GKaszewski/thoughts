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
        let also_known_as: Option<Vec<&str>> = if a.also_known_as.is_empty() {
            None
        } else {
            Some(a.also_known_as.iter().map(|s| s.as_str()).collect())
        };
        let attachment_json: serde_json::Value = a
            .attachment
            .iter()
            .map(|(n, v)| serde_json::json!({"name": n, "value": v}))
            .collect();
        sqlx::query(
            "INSERT INTO remote_actors(url,handle,display_name,avatar_url,last_fetched_at,
             bio,banner_url,outbox_url,followers_url,following_url,also_known_as,attachment)
             VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)
             ON CONFLICT(url) DO UPDATE SET
             handle=EXCLUDED.handle,display_name=EXCLUDED.display_name,
             avatar_url=EXCLUDED.avatar_url,last_fetched_at=EXCLUDED.last_fetched_at,
             bio=EXCLUDED.bio,banner_url=EXCLUDED.banner_url,
             outbox_url=EXCLUDED.outbox_url,followers_url=EXCLUDED.followers_url,
             following_url=EXCLUDED.following_url,also_known_as=EXCLUDED.also_known_as,
             attachment=EXCLUDED.attachment",
        )
        .bind(&a.url)
        .bind(&a.handle)
        .bind(&a.display_name)
        .bind(&a.avatar_url)
        .bind(a.last_fetched_at)
        .bind(&a.bio)
        .bind(&a.banner_url)
        .bind(&a.outbox_url)
        .bind(&a.followers_url)
        .bind(&a.following_url)
        .bind(also_known_as.as_deref())
        .bind(&attachment_json)
        .execute(&self.pool)
        .await
        .into_domain()
        .map(|_| ())
    }

    async fn find_by_url(&self, url: &str) -> Result<Option<RemoteActor>, DomainError> {
        #[derive(sqlx::FromRow)]
        struct Row {
            url: String,
            handle: String,
            display_name: Option<String>,
            avatar_url: Option<String>,
            last_fetched_at: DateTime<Utc>,
            bio: Option<String>,
            banner_url: Option<String>,
            outbox_url: Option<String>,
            followers_url: Option<String>,
            following_url: Option<String>,
            also_known_as: Option<Vec<String>>,
            inbox_url: Option<String>,
            shared_inbox_url: Option<String>,
            attachment: Option<serde_json::Value>,
        }
        sqlx::query_as::<_, Row>(
            "SELECT url,handle,display_name,avatar_url,last_fetched_at,
             bio,banner_url,outbox_url,followers_url,following_url,also_known_as,
             inbox_url,shared_inbox_url,attachment
             FROM remote_actors WHERE url=$1",
        )
        .bind(url)
        .fetch_optional(&self.pool)
        .await
        .into_domain()
        .map(|o| {
            o.map(|r| RemoteActor {
                url: r.url,
                handle: r.handle,
                display_name: r.display_name,
                avatar_url: r.avatar_url,
                last_fetched_at: r.last_fetched_at,
                bio: r.bio,
                banner_url: r.banner_url,
                also_known_as: r.also_known_as.unwrap_or_default(),
                outbox_url: r.outbox_url,
                followers_url: r.followers_url,
                following_url: r.following_url,
                inbox_url: r.inbox_url,
                shared_inbox_url: r.shared_inbox_url,
                attachment: r
                    .attachment
                    .and_then(|v| v.as_array().cloned())
                    .map(|arr| {
                        arr.into_iter()
                            .filter_map(|item| {
                                let name = item.get("name")?.as_str()?.to_string();
                                let value = item.get("value")?.as_str()?.to_string();
                                Some((name, value))
                            })
                            .collect()
                    })
                    .unwrap_or_default(),
            })
        })
    }
}
