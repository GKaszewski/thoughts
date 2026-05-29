use crate::db_error::IntoDbResult;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

use domain::{
    errors::DomainError,
    models::{
        feed::{PageParams, Paginated},
        thought::{Thought, Visibility},
    },
    ports::ThoughtRepository,
    value_objects::{Content, ThoughtId, UserId},
};
use sqlx::PgPool;

pub struct PgThoughtRepository {
    pool: PgPool,
}
impl PgThoughtRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
pub(crate) struct ThoughtRow {
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub content: String,
    pub in_reply_to_id: Option<uuid::Uuid>,
    pub visibility: String,
    pub content_warning: Option<String>,
    pub sensitive: bool,
    pub local: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub note_extensions: Option<serde_json::Value>,
    pub mood: Option<String>,
}

impl TryFrom<ThoughtRow> for Thought {
    type Error = DomainError;
    fn try_from(r: ThoughtRow) -> Result<Self, DomainError> {
        Ok(Thought {
            id: ThoughtId::from_uuid(r.id),
            user_id: UserId::from_uuid(r.user_id),
            content: Content::new_remote(r.content),
            in_reply_to_id: r.in_reply_to_id.map(ThoughtId::from_uuid),
            visibility: Visibility::from_db_str(&r.visibility)?,
            content_warning: r.content_warning,
            sensitive: r.sensitive,
            local: r.local,
            created_at: r.created_at,
            updated_at: r.updated_at,
            note_extensions: r.note_extensions,
            mood: r.mood,
        })
    }
}

const THOUGHT_SELECT: &str =
    "SELECT id,user_id,content,in_reply_to_id,visibility,content_warning,sensitive,local,created_at,updated_at,note_extensions,mood FROM thoughts";

#[async_trait]
impl ThoughtRepository for PgThoughtRepository {
    async fn save(&self, t: &Thought) -> Result<(), DomainError> {
        sqlx::query(
            "INSERT INTO thoughts(id,user_id,content,in_reply_to_id,visibility,content_warning,sensitive,local,created_at,mood)
             VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
             ON CONFLICT(id) DO UPDATE SET content=EXCLUDED.content,updated_at=NOW()"
        )
        .bind(t.id.as_uuid())
        .bind(t.user_id.as_uuid())
        .bind(t.content.as_str())
        .bind(t.in_reply_to_id.as_ref().map(|x| x.as_uuid()))
        .bind(t.visibility.as_str())
        .bind(&t.content_warning)
        .bind(t.sensitive)
        .bind(t.local)
        .bind(t.created_at)
        .bind(&t.mood)
        .execute(&self.pool)
        .await
        .into_domain()
        .map(|_| ())
    }

    async fn find_by_id(&self, id: &ThoughtId) -> Result<Option<Thought>, DomainError> {
        sqlx::query_as::<_, ThoughtRow>(&format!("{THOUGHT_SELECT} WHERE id=$1"))
            .bind(id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .into_domain()
            .and_then(|o| o.map(Thought::try_from).transpose())
    }

    async fn delete(&self, id: &ThoughtId, user_id: &UserId) -> Result<(), DomainError> {
        let r = sqlx::query("DELETE FROM thoughts WHERE id=$1 AND user_id=$2")
            .bind(id.as_uuid())
            .bind(user_id.as_uuid())
            .execute(&self.pool)
            .await
            .into_domain()?;
        if r.rows_affected() == 0 {
            return Err(DomainError::NotFound);
        }
        Ok(())
    }

    async fn update_content(&self, id: &ThoughtId, content: &Content) -> Result<(), DomainError> {
        sqlx::query("UPDATE thoughts SET content=$2,updated_at=NOW() WHERE id=$1")
            .bind(id.as_uuid())
            .bind(content.as_str())
            .execute(&self.pool)
            .await
            .into_domain()
            .map(|_| ())
    }

    async fn get_thread(&self, id: &ThoughtId) -> Result<Vec<Thought>, DomainError> {
        // Recursive CTE: fetches the root thought and all nested replies at any depth.
        sqlx::query_as::<_, ThoughtRow>(
            "WITH RECURSIVE thread AS (
                SELECT id,user_id,content,in_reply_to_id,
                       visibility,content_warning,sensitive,local,created_at,updated_at,note_extensions,mood
                FROM thoughts WHERE id = $1
                UNION ALL
                SELECT t.id,t.user_id,t.content,t.in_reply_to_id,
                       t.visibility,t.content_warning,t.sensitive,t.local,t.created_at,t.updated_at,t.note_extensions,t.mood
                FROM thoughts t JOIN thread ON t.in_reply_to_id = thread.id
            )
            SELECT * FROM thread ORDER BY created_at ASC",
        )
        .bind(id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .into_domain()
        .and_then(|rows| rows.into_iter().map(Thought::try_from).collect())
    }

    async fn list_by_user(
        &self,
        user_id: &UserId,
        page: &PageParams,
    ) -> Result<Paginated<Thought>, DomainError> {
        let uid = user_id.as_uuid();
        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM thoughts WHERE user_id = $1")
            .bind(uid)
            .fetch_one(&self.pool)
            .await
            .into_domain()?;

        let rows = sqlx::query_as::<_, ThoughtRow>(&format!(
            "{THOUGHT_SELECT} WHERE user_id=$1 ORDER BY created_at DESC LIMIT $2 OFFSET $3"
        ))
        .bind(uid)
        .bind(page.limit())
        .bind(page.offset())
        .fetch_all(&self.pool)
        .await
        .into_domain()?;

        Ok(Paginated {
            items: rows
                .into_iter()
                .map(Thought::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            total,
            page: page.page,
            per_page: page.per_page,
        })
    }
}

#[cfg(test)]
mod tests;
