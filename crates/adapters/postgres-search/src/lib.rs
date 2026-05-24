use async_trait::async_trait;
use chrono::{DateTime, Utc};

use domain::{
    errors::DomainError,
    models::{
        feed::{FeedEntry, PageParams, Paginated},
        thought::{Thought, Visibility},
        user::User,
    },
    ports::SearchPort,
    value_objects::{Content, Email, PasswordHash, ThoughtId, UserId, Username},
};
use postgres::user::{UserRow, USER_SELECT};
use sqlx::PgPool;

pub struct PgSearchRepository {
    pool: PgPool,
}
impl PgSearchRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct FeedRow {
    thought_id: uuid::Uuid,
    t_user_id: uuid::Uuid,
    content: String,
    in_reply_to_id: Option<uuid::Uuid>,
    visibility: String,
    content_warning: Option<String>,
    sensitive: bool,
    t_local: bool,
    thought_created_at: DateTime<Utc>,
    updated_at: Option<DateTime<Utc>>,
    author_id: uuid::Uuid,
    username: String,
    email: String,
    password_hash: String,
    display_name: Option<String>,
    bio: Option<String>,
    avatar_url: Option<String>,
    header_url: Option<String>,
    custom_css: Option<String>,
    author_local: bool,
    author_created_at: DateTime<Utc>,
    author_updated_at: DateTime<Utc>,
    like_count: i64,
    boost_count: i64,
    reply_count: i64,
    liked_by_viewer: bool,
    boosted_by_viewer: bool,
    note_extensions: Option<serde_json::Value>,
}

fn feed_select(viewer: Option<uuid::Uuid>) -> String {
    let viewer_checks = match viewer {
        Some(uid) => format!(
            "EXISTS(SELECT 1 FROM likes  WHERE user_id='{uid}' AND thought_id=t.id) AS liked_by_viewer,\n\
             EXISTS(SELECT 1 FROM boosts WHERE user_id='{uid}' AND thought_id=t.id) AS boosted_by_viewer"
        ),
        None => "false AS liked_by_viewer, false AS boosted_by_viewer".to_string(),
    };
    format!(
        "\n    SELECT\n\
         t.id AS thought_id, t.user_id AS t_user_id, t.content,\n\
         t.in_reply_to_id,\n\
         t.visibility, t.content_warning, t.sensitive, t.local AS t_local,\n\
         t.created_at AS thought_created_at, t.updated_at, t.note_extensions,\n\
         u.id AS author_id, u.username, u.email, u.password_hash,\n\
         u.display_name, u.bio, u.avatar_url, u.header_url, u.custom_css,\n\
         u.local AS author_local,\n\
         u.created_at AS author_created_at, u.updated_at AS author_updated_at,\n\
         (SELECT COUNT(*) FROM likes  l WHERE l.thought_id=t.id) AS like_count,\n\
         (SELECT COUNT(*) FROM boosts b WHERE b.thought_id=t.id) AS boost_count,\n\
         (SELECT COUNT(*) FROM thoughts r WHERE r.in_reply_to_id=t.id) AS reply_count,\n\
         {viewer_checks}\n\
     FROM thoughts t JOIN users u ON u.id=t.user_id"
    )
}

fn row_to_entry(r: FeedRow, viewer: Option<uuid::Uuid>) -> Result<FeedEntry, DomainError> {
    let thought = Thought {
        id: ThoughtId::from_uuid(r.thought_id),
        user_id: UserId::from_uuid(r.t_user_id),
        content: Content::new_remote(r.content),
        in_reply_to_id: r.in_reply_to_id.map(ThoughtId::from_uuid),
        visibility: Visibility::from_db_str(&r.visibility)?,
        content_warning: r.content_warning,
        sensitive: r.sensitive,
        local: r.t_local,
        created_at: r.thought_created_at,
        updated_at: r.updated_at,
        note_extensions: r.note_extensions,
    };
    let author = User {
        id: UserId::from_uuid(r.author_id),
        username: Username::from_trusted(r.username),
        email: Email::from_trusted(r.email),
        password_hash: PasswordHash(r.password_hash),
        display_name: r.display_name,
        bio: r.bio,
        avatar_url: r.avatar_url,
        header_url: r.header_url,
        custom_css: r.custom_css,
        local: r.author_local,
        created_at: r.author_created_at,
        updated_at: r.author_updated_at,
    };
    Ok(FeedEntry {
        thought,
        author,
        stats: domain::models::feed::EngagementStats {
            like_count: r.like_count,
            boost_count: r.boost_count,
            reply_count: r.reply_count,
        },
        viewer: viewer.map(|_| domain::models::feed::ViewerContext {
            liked: r.liked_by_viewer,
            boosted: r.boosted_by_viewer,
        }),
    })
}

#[async_trait]
impl SearchPort for PgSearchRepository {
    async fn search_thoughts(
        &self,
        query: &str,
        page: &PageParams,
        viewer_id: Option<&UserId>,
    ) -> Result<Paginated<FeedEntry>, DomainError> {
        let viewer = viewer_id.map(|v| v.as_uuid());
        let select = feed_select(viewer);

        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM thoughts t
             WHERE t.content % $1 AND t.visibility='public'",
        )
        .bind(query)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        let sql = format!(
            "{select}
             WHERE t.content % $1 AND t.visibility='public'
             ORDER BY similarity(t.content, $1) DESC
             LIMIT $2 OFFSET $3"
        );
        let rows = sqlx::query_as::<_, FeedRow>(&sql)
            .bind(query)
            .bind(page.limit())
            .bind(page.offset())
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(Paginated {
            items: rows
                .into_iter()
                .map(|r| row_to_entry(r, viewer))
                .collect::<Result<Vec<_>, _>>()?,
            total,
            page: page.page,
            per_page: page.per_page,
        })
    }

    async fn search_users(
        &self,
        query: &str,
        page: &PageParams,
    ) -> Result<Paginated<User>, DomainError> {
        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM users u
             WHERE u.local=true AND (u.username % $1 OR u.display_name % $1)",
        )
        .bind(query)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        let sql = format!(
            "{USER_SELECT}
             WHERE local=true AND (username % $1 OR display_name % $1)
             ORDER BY similarity(username || ' ' || COALESCE(display_name,''), $1) DESC
             LIMIT $2 OFFSET $3"
        );
        let rows = sqlx::query_as::<_, UserRow>(&sql)
            .bind(query)
            .bind(page.limit())
            .bind(page.offset())
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

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
