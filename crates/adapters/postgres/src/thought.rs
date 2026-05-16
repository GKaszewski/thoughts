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
        })
    }
}

const THOUGHT_SELECT: &str =
    "SELECT id,user_id,content,in_reply_to_id,visibility,content_warning,sensitive,local,created_at,updated_at FROM thoughts";

#[async_trait]
impl ThoughtRepository for PgThoughtRepository {
    async fn save(&self, t: &Thought) -> Result<(), DomainError> {
        sqlx::query(
            "INSERT INTO thoughts(id,user_id,content,in_reply_to_id,visibility,content_warning,sensitive,local,created_at)
             VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9)
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
                       visibility,content_warning,sensitive,local,created_at,updated_at
                FROM thoughts WHERE id = $1
                UNION ALL
                SELECT t.id,t.user_id,t.content,t.in_reply_to_id,
                       t.visibility,t.content_warning,t.sensitive,t.local,t.created_at,t.updated_at
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
mod tests {
    use super::*;
    use crate::test_helpers::seed_user;
    use domain::{
        models::thought::{Thought, Visibility},
        value_objects::*,
    };

    #[sqlx::test(migrations = "./migrations")]
    async fn save_and_find_thought(pool: sqlx::PgPool) {
        let user = seed_user(&pool, "alice", "alice@ex.com").await;
        let repo = PgThoughtRepository::new(pool);
        let t = Thought::new_local(
            ThoughtId::new(),
            user.id.clone(),
            Content::new_local("hello world").unwrap(),
            None,
            Visibility::Public,
            None,
            false,
        );
        repo.save(&t).await.unwrap();
        let found = repo.find_by_id(&t.id).await.unwrap().unwrap();
        assert_eq!(found.content.as_str(), "hello world");
        assert!(found.local);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn delete_thought(pool: sqlx::PgPool) {
        let user = seed_user(&pool, "bob", "bob@ex.com").await;
        let repo = PgThoughtRepository::new(pool);
        let t = Thought::new_local(
            ThoughtId::new(),
            user.id.clone(),
            Content::new_local("bye").unwrap(),
            None,
            Visibility::Public,
            None,
            false,
        );
        repo.save(&t).await.unwrap();
        repo.delete(&t.id, &user.id).await.unwrap();
        assert!(repo.find_by_id(&t.id).await.unwrap().is_none());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn delete_wrong_owner_returns_not_found(pool: sqlx::PgPool) {
        let alice = seed_user(&pool, "alice", "alice@ex.com").await;
        let bob = seed_user(&pool, "bob", "bob@ex.com").await;
        let repo = PgThoughtRepository::new(pool);
        let t = Thought::new_local(
            ThoughtId::new(),
            alice.id.clone(),
            Content::new_local("secret").unwrap(),
            None,
            Visibility::Public,
            None,
            false,
        );
        repo.save(&t).await.unwrap();
        let err = repo.delete(&t.id, &bob.id).await.unwrap_err();
        assert!(matches!(err, DomainError::NotFound));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn get_thread_returns_root_and_replies(pool: sqlx::PgPool) {
        let user = seed_user(&pool, "charlie", "charlie@ex.com").await;
        let repo = PgThoughtRepository::new(pool);
        let root = Thought::new_local(
            ThoughtId::new(),
            user.id.clone(),
            Content::new_local("root").unwrap(),
            None,
            Visibility::Public,
            None,
            false,
        );
        let reply = Thought::new_local(
            ThoughtId::new(),
            user.id.clone(),
            Content::new_local("reply").unwrap(),
            Some(root.id.clone()),
            Visibility::Public,
            None,
            false,
        );
        repo.save(&root).await.unwrap();
        repo.save(&reply).await.unwrap();
        let thread = repo.get_thread(&root.id).await.unwrap();
        assert_eq!(thread.len(), 2);
    }
}
