use crate::db_error::IntoDbResult;
use async_trait::async_trait;

const MAX_REMOTE_CONTENT_CHARS: usize = 5000;
const THOUGHTS_PATH_PREFIX: &str = "/thoughts/";
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use activitypub::{AcceptNoteInput, ActivityPubRepository, ActorApUrls, OutboxEntry};
use domain::{
    errors::DomainError,
    models::thought::{Thought, Visibility},
    value_objects::{Content, ThoughtId, UserId, Username},
};

#[derive(sqlx::FromRow)]
struct OutboxRow {
    id: uuid::Uuid,
    user_id: uuid::Uuid,
    content: String,
    created_at: DateTime<Utc>,
    in_reply_to_id: Option<uuid::Uuid>,
    content_warning: Option<String>,
    sensitive: bool,
    username: String,
    updated_at: Option<DateTime<Utc>>,
}

impl OutboxRow {
    fn into_entry(self) -> OutboxEntry {
        OutboxEntry {
            thought: Thought {
                id: ThoughtId::from_uuid(self.id),
                user_id: UserId::from_uuid(self.user_id),
                content: Content::new_remote(self.content),
                in_reply_to_id: self.in_reply_to_id.map(ThoughtId::from_uuid),
                visibility: Visibility::Public,
                content_warning: self.content_warning,
                sensitive: self.sensitive,
                local: true,
                created_at: self.created_at,
                updated_at: self.updated_at,
                note_extensions: None,
                mood: None,
            },
            author_username: Username::from_trusted(self.username),
        }
    }
}

pub struct PgActivityPubRepository {
    pool: PgPool,
}

impl PgActivityPubRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ActivityPubRepository for PgActivityPubRepository {
    async fn outbox_entries_for_actor(
        &self,
        user_id: &UserId,
    ) -> Result<Vec<OutboxEntry>, DomainError> {
        sqlx::query_as::<_, OutboxRow>(
            "SELECT t.id, t.user_id, t.content, t.created_at, t.in_reply_to_id, t.content_warning, t.sensitive, u.username, t.updated_at
             FROM thoughts t JOIN users u ON u.id=t.user_id
             WHERE t.user_id=$1 AND t.local=true AND t.visibility='public'
             ORDER BY t.created_at DESC",
        )
        .bind(user_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .into_domain()
        .map(|rows| rows.into_iter().map(OutboxRow::into_entry).collect())
    }

    async fn outbox_page_for_actor(
        &self,
        user_id: &UserId,
        before: Option<DateTime<Utc>>,
        limit: usize,
    ) -> Result<Vec<OutboxEntry>, DomainError> {
        let rows = if let Some(before) = before {
            sqlx::query_as::<_, OutboxRow>(
                "SELECT t.id, t.user_id, t.content, t.created_at, t.in_reply_to_id, t.content_warning, t.sensitive, u.username, t.updated_at
                 FROM thoughts t JOIN users u ON u.id=t.user_id
                 WHERE t.user_id=$1 AND t.local=true AND t.visibility='public' AND t.created_at < $2
                 ORDER BY t.created_at DESC LIMIT $3",
            )
            .bind(user_id.as_uuid())
            .bind(before)
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, OutboxRow>(
                "SELECT t.id, t.user_id, t.content, t.created_at, t.in_reply_to_id, t.content_warning, t.sensitive, u.username, t.updated_at
                 FROM thoughts t JOIN users u ON u.id=t.user_id
                 WHERE t.user_id=$1 AND t.local=true AND t.visibility='public'
                 ORDER BY t.created_at DESC LIMIT $2",
            )
            .bind(user_id.as_uuid())
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await
        }
        .into_domain()?;

        Ok(rows.into_iter().map(OutboxRow::into_entry).collect())
    }

    async fn find_remote_actor_id(
        &self,
        actor_ap_url: &str,
    ) -> Result<Option<UserId>, DomainError> {
        sqlx::query_scalar::<_, uuid::Uuid>("SELECT id FROM users WHERE ap_id=$1")
            .bind(actor_ap_url)
            .fetch_optional(&self.pool)
            .await
            .into_domain()
            .map(|o| o.map(UserId::from_uuid))
    }

    async fn intern_remote_actor(&self, actor_ap_url: &str) -> Result<UserId, DomainError> {
        if let Some(id) = self.find_remote_actor_id(actor_ap_url).await? {
            return Ok(id);
        }
        let new_id = uuid::Uuid::new_v4();
        let parsed = url::Url::parse(actor_ap_url).ok();
        let domain_str = parsed
            .as_ref()
            .and_then(|u| u.host_str().map(|s| s.to_string()))
            .unwrap_or_default();
        let last_seg = parsed
            .and_then(|u| {
                u.path_segments()
                    .and_then(|mut s| s.next_back().map(|s| s.to_string()))
            })
            .unwrap_or_default();
        let handle = if last_seg.is_empty() || domain_str.is_empty() {
            format!("r_{}", &new_id.to_string()[..13])
        } else {
            let candidate = format!("{}@{}", last_seg, domain_str);
            if candidate.len() <= 255 {
                candidate
            } else {
                format!("r_{}", &new_id.to_string()[..13])
            }
        };
        let result = sqlx::query(
            "INSERT INTO users(id,username,email,password_hash,local,ap_id,created_at,updated_at)
             VALUES($1,$2,$3,'',false,$4,NOW(),NOW()) ON CONFLICT(ap_id) DO NOTHING",
        )
        .bind(new_id)
        .bind(&handle)
        .bind(format!("{}@remote", new_id))
        .bind(actor_ap_url)
        .execute(&self.pool)
        .await;

        if result.is_err() {
            let fallback = format!("r_{}", &new_id.to_string()[..13]);
            let new_id2 = uuid::Uuid::new_v4();
            sqlx::query(
                "INSERT INTO users(id,username,email,password_hash,local,ap_id,created_at,updated_at)
                 VALUES($1,$2,$3,'',false,$4,NOW(),NOW()) ON CONFLICT(ap_id) DO NOTHING",
            )
            .bind(new_id2)
            .bind(&fallback)
            .bind(format!("{}@remote", new_id2))
            .bind(actor_ap_url)
            .execute(&self.pool)
            .await
            .into_domain()?;
        }

        self.find_remote_actor_id(actor_ap_url)
            .await?
            .ok_or_else(|| {
                DomainError::Internal(
                    "intern_remote_actor: insert succeeded but row not found".into(),
                )
            })
    }

    async fn update_remote_actor_display(
        &self,
        user_id: &UserId,
        display_name: Option<&str>,
        avatar_url: Option<&str>,
    ) -> Result<(), DomainError> {
        sqlx::query(
            "UPDATE users SET display_name=$1, avatar_url=$2, updated_at=NOW()
             WHERE id=$3 AND local=false",
        )
        .bind(display_name)
        .bind(avatar_url)
        .bind(user_id.as_uuid())
        .execute(&self.pool)
        .await
        .into_domain()
        .map(|_| ())
    }

    async fn accept_note(&self, input: AcceptNoteInput<'_>) -> Result<ThoughtId, DomainError> {
        let AcceptNoteInput {
            ap_id,
            author_id,
            content,
            published,
            sensitive,
            content_warning,
            visibility,
            in_reply_to,
            note_extensions,
        } = input;
        let capped: String = content.chars().take(MAX_REMOTE_CONTENT_CHARS).collect();
        let (in_reply_to_id, in_reply_to_url) = match in_reply_to {
            Some(url) => {
                // Fast path: local thought URL contains the UUID directly.
                let local_uuid = url::Url::parse(url).ok().and_then(|u| {
                    u.path()
                        .strip_prefix(THOUGHTS_PATH_PREFIX)
                        .and_then(|s| s.split('/').next())
                        .and_then(|s| uuid::Uuid::parse_str(s).ok())
                });
                // Slow path: remote parent — look up by ap_id so remote-to-remote
                // replies are threaded correctly in the feed.
                let resolved = if local_uuid.is_some() {
                    local_uuid
                } else {
                    sqlx::query_scalar::<_, uuid::Uuid>("SELECT id FROM thoughts WHERE ap_id=$1")
                        .bind(url)
                        .fetch_optional(&self.pool)
                        .await
                        .into_domain()?
                };
                (resolved, Some(url.to_string()))
            }
            None => (None, None),
        };
        sqlx::query(
            "INSERT INTO thoughts(id,user_id,content,ap_id,visibility,sensitive,local,content_warning,created_at,in_reply_to_id,in_reply_to_url,note_extensions)
             VALUES($1,$2,$3,$4,$8,$5,false,$6,$7,$9,$10,$11) ON CONFLICT(ap_id) DO NOTHING",
        )
        .bind(uuid::Uuid::new_v4())
        .bind(author_id.as_uuid())
        .bind(&capped)
        .bind(ap_id)
        .bind(sensitive)
        .bind(content_warning)
        .bind(published)
        .bind(visibility)
        .bind(in_reply_to_id)
        .bind(&in_reply_to_url)
        .bind(note_extensions)
        .execute(&self.pool)
        .await
        .into_domain()?;

        // SELECT the id — works whether the INSERT was a no-op or not (idempotent).
        let row: (uuid::Uuid,) = sqlx::query_as("SELECT id FROM thoughts WHERE ap_id=$1")
            .bind(ap_id)
            .fetch_one(&self.pool)
            .await
            .into_domain()?;
        Ok(ThoughtId::from_uuid(row.0))
    }

    async fn apply_note_update(
        &self,
        ap_id: &str,
        new_content: &str,
        note_extensions: Option<serde_json::Value>,
    ) -> Result<(), DomainError> {
        let capped: String = new_content.chars().take(MAX_REMOTE_CONTENT_CHARS).collect();
        sqlx::query(
            "UPDATE thoughts SET content=$2,note_extensions=$3,updated_at=NOW() WHERE ap_id=$1 AND local=false",
        )
        .bind(ap_id)
        .bind(&capped)
        .bind(&note_extensions)
        .execute(&self.pool)
        .await
        .into_domain()
        .map(|_| ())
    }

    async fn retract_note(&self, ap_id: &str) -> Result<(), DomainError> {
        sqlx::query("DELETE FROM thoughts WHERE ap_id=$1 AND local=false")
            .bind(ap_id)
            .execute(&self.pool)
            .await
            .into_domain()
            .map(|_| ())
    }

    async fn retract_actor_notes(&self, actor_ap_url: &str) -> Result<(), DomainError> {
        sqlx::query(
            "DELETE FROM thoughts WHERE local=false AND user_id=(SELECT id FROM users WHERE ap_id=$1)",
        )
        .bind(actor_ap_url)
        .execute(&self.pool)
        .await
        .into_domain()
        .map(|_| ())
    }

    async fn count_local_notes(&self) -> Result<u64, DomainError> {
        let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM thoughts WHERE local=true")
            .fetch_one(&self.pool)
            .await
            .into_domain()?;
        Ok(n as u64)
    }

    async fn get_thought_ap_id(
        &self,
        thought_id: &ThoughtId,
    ) -> Result<Option<String>, DomainError> {
        sqlx::query_scalar::<_, String>(
            "SELECT ap_id FROM thoughts WHERE id = $1 AND ap_id IS NOT NULL",
        )
        .bind(thought_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .into_domain()
    }

    async fn get_actor_ap_urls(
        &self,
        user_id: &UserId,
    ) -> Result<Option<ActorApUrls>, DomainError> {
        sqlx::query_as::<_, (String, String)>(
            "SELECT ap_id, inbox_url FROM users \
             WHERE id = $1 AND ap_id IS NOT NULL AND inbox_url IS NOT NULL",
        )
        .bind(user_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .into_domain()
        .map(|opt| opt.map(|(ap_id, inbox_url)| ActorApUrls { ap_id, inbox_url }))
    }

    async fn sync_remote_actor_to_user(&self, actor_ap_url: &str) -> Result<(), DomainError> {
        sqlx::query(
            "UPDATE users SET display_name = ra.display_name, avatar_url = ra.avatar_url, updated_at = NOW()
             FROM remote_actors ra
             WHERE users.ap_id = ra.url AND users.ap_id = $1 AND users.local = false",
        )
        .bind(actor_ap_url)
        .execute(&self.pool)
        .await
        .into_domain()
        .map(|_| ())
    }
}

#[cfg(test)]
mod tests;
