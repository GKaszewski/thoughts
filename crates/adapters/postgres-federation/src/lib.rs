use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use activitypub_base::{
    ApUser, ApUserRepository, BlockedDomain, FederationRepository, Follower, FollowerStatus,
    FollowingStatus, RemoteActor,
};

// ── PostgresFederationRepository ─────────────────────────────────────────────

pub struct PostgresFederationRepository {
    pool: PgPool,
}

impl PostgresFederationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn status_str(s: &FollowerStatus) -> &'static str {
    match s {
        FollowerStatus::Pending => "pending",
        FollowerStatus::Accepted => "accepted",
        FollowerStatus::Rejected => "rejected",
    }
}
fn str_status(s: &str) -> FollowerStatus {
    match s {
        "accepted" => FollowerStatus::Accepted,
        "rejected" => FollowerStatus::Rejected,
        _ => FollowerStatus::Pending,
    }
}

fn map_remote_actor(
    url: String,
    handle: String,
    inbox_url: String,
    shared_inbox_url: Option<String>,
    display_name: Option<String>,
    avatar_url: Option<String>,
    outbox_url: Option<String>,
) -> RemoteActor {
    RemoteActor {
        url,
        handle,
        inbox_url,
        shared_inbox_url,
        display_name,
        avatar_url,
        outbox_url,
    }
}

#[async_trait]
impl FederationRepository for PostgresFederationRepository {
    async fn add_follower(
        &self,
        local_user_id: uuid::Uuid,
        remote_actor_url: &str,
        status: FollowerStatus,
        follow_activity_id: &str,
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO federation_followers(local_user_id,remote_actor_url,status,follow_activity_id)
             VALUES($1,$2,$3,$4)
             ON CONFLICT(local_user_id,remote_actor_url) DO UPDATE
             SET status=EXCLUDED.status, follow_activity_id=EXCLUDED.follow_activity_id"
        )
        .bind(local_user_id).bind(remote_actor_url).bind(status_str(&status)).bind(follow_activity_id)
        .execute(&self.pool).await.map_err(|e| anyhow!(e)).map(|_| ())
    }

    async fn get_follower_follow_activity_id(
        &self,
        local_user_id: uuid::Uuid,
        remote_actor_url: &str,
    ) -> Result<Option<String>> {
        sqlx::query_scalar::<_, String>(
            "SELECT follow_activity_id FROM federation_followers WHERE local_user_id=$1 AND remote_actor_url=$2"
        ).bind(local_user_id).bind(remote_actor_url).fetch_optional(&self.pool).await.map_err(|e| anyhow!(e))
    }

    async fn remove_follower(
        &self,
        local_user_id: uuid::Uuid,
        remote_actor_url: &str,
    ) -> Result<()> {
        sqlx::query(
            "DELETE FROM federation_followers WHERE local_user_id=$1 AND remote_actor_url=$2",
        )
        .bind(local_user_id)
        .bind(remote_actor_url)
        .execute(&self.pool)
        .await
        .map_err(|e| anyhow!(e))
        .map(|_| ())
    }

    async fn get_followers(&self, local_user_id: uuid::Uuid) -> Result<Vec<Follower>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            remote_actor_url: String,
            status: String,
            handle: String,
            inbox_url: String,
            shared_inbox_url: Option<String>,
            display_name: Option<String>,
            avatar_url: Option<String>,
            outbox_url: Option<String>,
        }
        sqlx::query_as::<_, Row>(
            "SELECT f.remote_actor_url, f.status, COALESCE(r.handle,'') AS handle,
             COALESCE(r.inbox_url,'') AS inbox_url, r.shared_inbox_url, r.display_name, r.avatar_url, r.outbox_url
             FROM federation_followers f
             LEFT JOIN remote_actors r ON r.url=f.remote_actor_url
             WHERE f.local_user_id=$1 AND f.status='accepted'"
        ).bind(local_user_id).fetch_all(&self.pool).await.map_err(|e| anyhow!(e)).map(|rows| rows.into_iter().map(|r| Follower {
            actor: map_remote_actor(r.remote_actor_url, r.handle, r.inbox_url, r.shared_inbox_url, r.display_name, r.avatar_url, r.outbox_url),
            status: str_status(&r.status),
        }).collect())
    }

    async fn get_followers_page(
        &self,
        local_user_id: uuid::Uuid,
        offset: u32,
        limit: usize,
    ) -> Result<Vec<Follower>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            remote_actor_url: String,
            status: String,
            handle: String,
            inbox_url: String,
            shared_inbox_url: Option<String>,
            display_name: Option<String>,
            avatar_url: Option<String>,
            outbox_url: Option<String>,
        }
        sqlx::query_as::<_, Row>(
            "SELECT f.remote_actor_url, f.status, COALESCE(r.handle,'') AS handle,
             COALESCE(r.inbox_url,'') AS inbox_url, r.shared_inbox_url, r.display_name, r.avatar_url, r.outbox_url
             FROM federation_followers f
             LEFT JOIN remote_actors r ON r.url=f.remote_actor_url
             WHERE f.local_user_id=$1 AND f.status='accepted'
             ORDER BY f.created_at DESC LIMIT $2 OFFSET $3"
        ).bind(local_user_id).bind(limit as i64).bind(offset as i64).fetch_all(&self.pool).await.map_err(|e| anyhow!(e)).map(|rows| rows.into_iter().map(|r| Follower {
            actor: map_remote_actor(r.remote_actor_url, r.handle, r.inbox_url, r.shared_inbox_url, r.display_name, r.avatar_url, r.outbox_url),
            status: str_status(&r.status),
        }).collect())
    }

    async fn count_followers(&self, local_user_id: uuid::Uuid) -> Result<usize> {
        let n: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM federation_followers WHERE local_user_id=$1 AND status='accepted'"
        ).bind(local_user_id).fetch_one(&self.pool).await.map_err(|e| anyhow!(e))?;
        Ok(n as usize)
    }

    async fn get_pending_followers(&self, local_user_id: uuid::Uuid) -> Result<Vec<RemoteActor>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            remote_actor_url: String,
            handle: String,
            inbox_url: String,
            shared_inbox_url: Option<String>,
            display_name: Option<String>,
            avatar_url: Option<String>,
            outbox_url: Option<String>,
        }
        sqlx::query_as::<_, Row>(
            "SELECT f.remote_actor_url, COALESCE(r.handle,'') AS handle,
             COALESCE(r.inbox_url,'') AS inbox_url, r.shared_inbox_url, r.display_name, r.avatar_url, r.outbox_url
             FROM federation_followers f
             LEFT JOIN remote_actors r ON r.url=f.remote_actor_url
             WHERE f.local_user_id=$1 AND f.status='pending'"
        ).bind(local_user_id).fetch_all(&self.pool).await.map_err(|e| anyhow!(e)).map(|rows| rows.into_iter().map(|r|
            map_remote_actor(r.remote_actor_url, r.handle, r.inbox_url, r.shared_inbox_url, r.display_name, r.avatar_url, r.outbox_url)
        ).collect())
    }

    async fn update_follower_status(
        &self,
        local_user_id: uuid::Uuid,
        remote_actor_url: &str,
        status: FollowerStatus,
    ) -> Result<()> {
        sqlx::query("UPDATE federation_followers SET status=$3 WHERE local_user_id=$1 AND remote_actor_url=$2")
            .bind(local_user_id).bind(remote_actor_url).bind(status_str(&status))
            .execute(&self.pool).await.map_err(|e| anyhow!(e)).map(|_| ())
    }

    async fn add_following(
        &self,
        local_user_id: uuid::Uuid,
        actor: RemoteActor,
        follow_activity_id: &str,
    ) -> Result<()> {
        self.upsert_remote_actor(actor.clone()).await?;
        sqlx::query(
            "INSERT INTO federation_following(local_user_id,remote_actor_url,follow_activity_id,outbox_url)
             VALUES($1,$2,$3,$4)
             ON CONFLICT(local_user_id,remote_actor_url) DO UPDATE
             SET follow_activity_id=EXCLUDED.follow_activity_id"
        )
        .bind(local_user_id).bind(&actor.url).bind(follow_activity_id).bind(&actor.outbox_url)
        .execute(&self.pool).await.map_err(|e| anyhow!(e)).map(|_| ())
    }

    async fn get_follow_activity_id(
        &self,
        local_user_id: uuid::Uuid,
        remote_actor_url: &str,
    ) -> Result<Option<String>> {
        sqlx::query_scalar::<_, String>(
            "SELECT follow_activity_id FROM federation_following WHERE local_user_id=$1 AND remote_actor_url=$2"
        ).bind(local_user_id).bind(remote_actor_url).fetch_optional(&self.pool).await.map_err(|e| anyhow!(e))
    }

    async fn remove_following(&self, local_user_id: uuid::Uuid, actor_url: &str) -> Result<()> {
        sqlx::query(
            "DELETE FROM federation_following WHERE local_user_id=$1 AND remote_actor_url=$2",
        )
        .bind(local_user_id)
        .bind(actor_url)
        .execute(&self.pool)
        .await
        .map_err(|e| anyhow!(e))
        .map(|_| ())
    }

    async fn get_following(&self, local_user_id: uuid::Uuid) -> Result<Vec<RemoteActor>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            remote_actor_url: String,
            handle: String,
            inbox_url: String,
            shared_inbox_url: Option<String>,
            display_name: Option<String>,
            avatar_url: Option<String>,
            outbox_url: Option<String>,
        }
        sqlx::query_as::<_, Row>(
            "SELECT f.remote_actor_url, COALESCE(r.handle,'') AS handle,
             COALESCE(r.inbox_url,'') AS inbox_url, r.shared_inbox_url, r.display_name, r.avatar_url, r.outbox_url
             FROM federation_following f
             LEFT JOIN remote_actors r ON r.url=f.remote_actor_url
             WHERE f.local_user_id=$1"
        ).bind(local_user_id).fetch_all(&self.pool).await.map_err(|e| anyhow!(e)).map(|rows| rows.into_iter().map(|r|
            map_remote_actor(r.remote_actor_url, r.handle, r.inbox_url, r.shared_inbox_url, r.display_name, r.avatar_url, r.outbox_url)
        ).collect())
    }

    async fn get_following_page(
        &self,
        local_user_id: uuid::Uuid,
        offset: u32,
        limit: usize,
    ) -> Result<Vec<RemoteActor>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            remote_actor_url: String,
            handle: String,
            inbox_url: String,
            shared_inbox_url: Option<String>,
            display_name: Option<String>,
            avatar_url: Option<String>,
            outbox_url: Option<String>,
        }
        sqlx::query_as::<_, Row>(
            "SELECT f.remote_actor_url, COALESCE(r.handle,'') AS handle,
             COALESCE(r.inbox_url,'') AS inbox_url, r.shared_inbox_url, r.display_name, r.avatar_url, r.outbox_url
             FROM federation_following f
             LEFT JOIN remote_actors r ON r.url=f.remote_actor_url
             WHERE f.local_user_id=$1 AND f.status='accepted'
             ORDER BY f.created_at DESC LIMIT $2 OFFSET $3"
        ).bind(local_user_id).bind(limit as i64).bind(offset as i64).fetch_all(&self.pool).await.map_err(|e| anyhow!(e)).map(|rows| rows.into_iter().map(|r|
            map_remote_actor(r.remote_actor_url, r.handle, r.inbox_url, r.shared_inbox_url, r.display_name, r.avatar_url, r.outbox_url)
        ).collect())
    }

    async fn count_following(&self, local_user_id: uuid::Uuid) -> Result<usize> {
        let n: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM federation_following WHERE local_user_id=$1 AND status='accepted'")
                .bind(local_user_id)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| anyhow!(e))?;
        Ok(n as usize)
    }

    async fn update_following_status(
        &self,
        _local_user_id: uuid::Uuid,
        _remote_actor_url: &str,
        _status: FollowingStatus,
    ) -> Result<()> {
        Ok(())
    }

    async fn get_following_outbox_url(
        &self,
        local_user_id: uuid::Uuid,
        remote_actor_url: &str,
    ) -> Result<Option<String>> {
        sqlx::query_scalar::<_, String>(
            "SELECT outbox_url FROM federation_following WHERE local_user_id=$1 AND remote_actor_url=$2"
        ).bind(local_user_id).bind(remote_actor_url).fetch_optional(&self.pool).await.map_err(|e| anyhow!(e))
    }

    async fn upsert_remote_actor(&self, actor: RemoteActor) -> Result<()> {
        sqlx::query(
            "INSERT INTO remote_actors(url,handle,display_name,inbox_url,shared_inbox_url,public_key,avatar_url,outbox_url,last_fetched_at)
             VALUES($1,$2,$3,$4,$5,'',$6,$7,NOW())
             ON CONFLICT(url) DO UPDATE SET handle=EXCLUDED.handle,display_name=EXCLUDED.display_name,
             inbox_url=EXCLUDED.inbox_url,shared_inbox_url=EXCLUDED.shared_inbox_url,
             avatar_url=EXCLUDED.avatar_url,outbox_url=EXCLUDED.outbox_url,last_fetched_at=NOW()"
        )
        .bind(&actor.url).bind(&actor.handle).bind(&actor.display_name)
        .bind(&actor.inbox_url).bind(&actor.shared_inbox_url)
        .bind(&actor.avatar_url).bind(&actor.outbox_url)
        .execute(&self.pool).await.map_err(|e| anyhow!(e)).map(|_| ())
    }

    async fn get_remote_actor(&self, actor_url: &str) -> Result<Option<RemoteActor>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            url: String,
            handle: String,
            inbox_url: String,
            shared_inbox_url: Option<String>,
            display_name: Option<String>,
            avatar_url: Option<String>,
            outbox_url: Option<String>,
        }
        sqlx::query_as::<_, Row>(
            "SELECT url,handle,inbox_url,shared_inbox_url,display_name,avatar_url,outbox_url FROM remote_actors WHERE url=$1"
        ).bind(actor_url).fetch_optional(&self.pool).await.map_err(|e| anyhow!(e)).map(|o| o.map(|r|
            map_remote_actor(r.url, r.handle, r.inbox_url, r.shared_inbox_url, r.display_name, r.avatar_url, r.outbox_url)
        ))
    }

    async fn get_local_actor_keypair(
        &self,
        user_id: uuid::Uuid,
    ) -> Result<Option<(String, String)>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            public_key: Option<String>,
            private_key: Option<String>,
        }
        let row = sqlx::query_as::<_, Row>(
            "SELECT public_key, private_key FROM users WHERE id=$1 AND local=true",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| anyhow!(e))?;
        Ok(row.and_then(|r| match (r.public_key, r.private_key) {
            (Some(pub_k), Some(priv_k)) => Some((pub_k, priv_k)),
            _ => None,
        }))
    }

    async fn save_local_actor_keypair(
        &self,
        user_id: uuid::Uuid,
        public_key: String,
        private_key: String,
    ) -> Result<()> {
        sqlx::query("UPDATE users SET public_key=$2, private_key=$3, updated_at=NOW() WHERE id=$1")
            .bind(user_id)
            .bind(&public_key)
            .bind(&private_key)
            .execute(&self.pool)
            .await
            .map_err(|e| anyhow!(e))
            .map(|_| ())
    }

    async fn add_announce(
        &self,
        activity_id: &str,
        object_url: &str,
        actor_url: &str,
        announced_at: DateTime<Utc>,
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO federation_announces(activity_id,object_url,actor_url,announced_at)
             VALUES($1,$2,$3,$4) ON CONFLICT(activity_id) DO NOTHING",
        )
        .bind(activity_id)
        .bind(object_url)
        .bind(actor_url)
        .bind(announced_at)
        .execute(&self.pool)
        .await
        .map_err(|e| anyhow!(e))
        .map(|_| ())
    }

    async fn count_announces(&self, object_url: &str) -> Result<usize> {
        let n: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM federation_announces WHERE object_url=$1")
                .bind(object_url)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| anyhow!(e))?;
        Ok(n as usize)
    }

    async fn add_blocked_domain(&self, domain: &str, reason: Option<&str>) -> Result<()> {
        sqlx::query(
            "INSERT INTO federation_blocked_domains(domain,reason) VALUES($1,$2) ON CONFLICT(domain) DO NOTHING"
        ).bind(domain).bind(reason).execute(&self.pool).await.map_err(|e| anyhow!(e)).map(|_| ())
    }

    async fn remove_blocked_domain(&self, domain: &str) -> Result<()> {
        sqlx::query("DELETE FROM federation_blocked_domains WHERE domain=$1")
            .bind(domain)
            .execute(&self.pool)
            .await
            .map_err(|e| anyhow!(e))
            .map(|_| ())
    }

    async fn get_blocked_domains(&self) -> Result<Vec<BlockedDomain>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            domain: String,
            reason: Option<String>,
            blocked_at: DateTime<Utc>,
        }
        sqlx::query_as::<_, Row>(
            "SELECT domain,reason,blocked_at FROM federation_blocked_domains ORDER BY domain",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| anyhow!(e))
        .map(|rows| {
            rows.into_iter()
                .map(|r| BlockedDomain {
                    domain: r.domain,
                    reason: r.reason,
                    blocked_at: r.blocked_at.to_rfc3339(),
                })
                .collect()
        })
    }

    async fn is_domain_blocked(&self, domain: &str) -> Result<bool> {
        let n: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM federation_blocked_domains WHERE domain=$1")
                .bind(domain)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| anyhow!(e))?;
        Ok(n > 0)
    }

    async fn add_blocked_actor(&self, local_user_id: uuid::Uuid, actor_url: &str) -> Result<()> {
        sqlx::query(
            "INSERT INTO federation_blocked_actors(local_user_id,actor_url) VALUES($1,$2) ON CONFLICT DO NOTHING"
        ).bind(local_user_id).bind(actor_url).execute(&self.pool).await.map_err(|e| anyhow!(e)).map(|_| ())
    }

    async fn remove_blocked_actor(&self, local_user_id: uuid::Uuid, actor_url: &str) -> Result<()> {
        sqlx::query("DELETE FROM federation_blocked_actors WHERE local_user_id=$1 AND actor_url=$2")
            .bind(local_user_id)
            .bind(actor_url)
            .execute(&self.pool)
            .await
            .map_err(|e| anyhow!(e))
            .map(|_| ())
    }

    async fn get_blocked_actors(&self, local_user_id: uuid::Uuid) -> Result<Vec<String>> {
        sqlx::query_scalar::<_, String>(
            "SELECT actor_url FROM federation_blocked_actors WHERE local_user_id=$1 ORDER BY created_at DESC"
        ).bind(local_user_id).fetch_all(&self.pool).await.map_err(|e| anyhow!(e))
    }

    async fn is_actor_blocked(&self, local_user_id: uuid::Uuid, actor_url: &str) -> Result<bool> {
        let n: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM federation_blocked_actors WHERE local_user_id=$1 AND actor_url=$2"
        ).bind(local_user_id).bind(actor_url).fetch_one(&self.pool).await.map_err(|e| anyhow!(e))?;
        Ok(n > 0)
    }
}

// ── PostgresApUserRepository ──────────────────────────────────────────────────

pub struct PostgresApUserRepository {
    pool: PgPool,
    base_url: String,
}

impl PostgresApUserRepository {
    pub fn new(pool: PgPool, base_url: String) -> Self {
        Self { pool, base_url }
    }

    fn row_to_ap_user(
        &self,
        id: uuid::Uuid,
        username: String,
        bio: Option<String>,
        avatar_url: Option<String>,
    ) -> ApUser {
        let profile_url = url::Url::parse(&format!("{}/users/{}", self.base_url, username)).ok();
        let avatar_url = avatar_url.and_then(|u| url::Url::parse(&u).ok());
        ApUser {
            id,
            username,
            bio,
            avatar_url,
            banner_url: None,
            also_known_as: None,
            profile_url,
            attachment: vec![],
        }
    }
}

#[async_trait]
impl ApUserRepository for PostgresApUserRepository {
    async fn find_by_id(&self, id: uuid::Uuid) -> Result<Option<ApUser>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: uuid::Uuid,
            username: String,
            bio: Option<String>,
            avatar_url: Option<String>,
        }
        let row = sqlx::query_as::<_, Row>(
            "SELECT id,username,bio,avatar_url FROM users WHERE id=$1 AND local=true",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| anyhow!(e))?;
        Ok(row.map(|r| self.row_to_ap_user(r.id, r.username, r.bio, r.avatar_url)))
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<ApUser>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: uuid::Uuid,
            username: String,
            bio: Option<String>,
            avatar_url: Option<String>,
        }
        let row = sqlx::query_as::<_, Row>(
            "SELECT id,username,bio,avatar_url FROM users WHERE username=$1 AND local=true",
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| anyhow!(e))?;
        Ok(row.map(|r| self.row_to_ap_user(r.id, r.username, r.bio, r.avatar_url)))
    }

    async fn count_users(&self) -> Result<usize> {
        let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE local=true")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!(e))?;
        Ok(n as usize)
    }
}
