use crate::db_error::IntoDbResult;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

use domain::{
    errors::DomainError,
    models::{
        feed::{FeedEntry, Paginated},
        thought::{Thought, Visibility},
        user::User,
    },
    ports::{FeedQuery, FeedRepository, FeedScope},
    value_objects::{Content, Email, PasswordHash, ThoughtId, UserId, Username},
};
use sqlx::PgPool;

pub struct PgFeedRepository {
    pool: PgPool,
}
impl PgFeedRepository {
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
    note_extensions: Option<serde_json::Value>,
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
}

fn federation_following_clause(follower: Option<uuid::Uuid>) -> String {
    match follower {
        Some(fid) => format!(
            " OR t.user_id IN (
                SELECT u2.id FROM users u2
                JOIN federation_following ff ON u2.ap_id = ff.remote_actor_url
                WHERE ff.local_user_id = '{fid}'
            )"
        ),
        None => String::new(),
    }
}

fn feed_select(viewer: Option<uuid::Uuid>) -> String {
    let viewer_checks = match viewer {
        Some(uid) => format!(
            "EXISTS(SELECT 1 FROM likes WHERE user_id='{uid}' AND thought_id=t.id) AS liked_by_viewer,
             EXISTS(SELECT 1 FROM boosts WHERE user_id='{uid}' AND thought_id=t.id) AS boosted_by_viewer"
        ),
        None => "false AS liked_by_viewer, false AS boosted_by_viewer".to_string(),
    };
    format!(
        "
    SELECT
        t.id AS thought_id, t.user_id AS t_user_id, t.content,
        t.in_reply_to_id,
        t.visibility, t.content_warning, t.sensitive, t.local AS t_local,
        t.created_at AS thought_created_at, t.updated_at,
        t.note_extensions,
        u.id AS author_id,
        CASE WHEN NOT u.local AND ra.handle IS NOT NULL AND ra.handle != ''
             THEN '@' || ra.handle ||
                  CASE WHEN ra.handle NOT LIKE '%@%'
                       THEN '@' || SPLIT_PART(ra.url, '/', 3)
                       ELSE '' END
             ELSE u.username END AS username,
        u.email, u.password_hash,
        COALESCE(ra.display_name, u.display_name) AS display_name,
        u.bio,
        COALESCE(ra.avatar_url, u.avatar_url) AS avatar_url,
        u.header_url, u.custom_css,
        u.local AS author_local,
        u.created_at AS author_created_at, u.updated_at AS author_updated_at,
        (SELECT COUNT(*) FROM likes l WHERE l.thought_id=t.id) AS like_count,
        (SELECT COUNT(*) FROM boosts b WHERE b.thought_id=t.id) AS boost_count,
        (SELECT COUNT(*) FROM thoughts r WHERE r.in_reply_to_id=t.id) AS reply_count,
        {viewer_checks}
    FROM thoughts t
    JOIN users u ON u.id=t.user_id
    LEFT JOIN remote_actors ra ON u.ap_id = ra.url"
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
impl FeedRepository for PgFeedRepository {
    async fn query(&self, q: &FeedQuery) -> Result<Paginated<FeedEntry>, DomainError> {
        let viewer = q.viewer_id.as_ref().map(|v| v.as_uuid());
        let page = &q.page;

        match &q.scope {
            FeedScope::Home { following_ids } => {
                let ids: Vec<uuid::Uuid> = following_ids.iter().map(|id| id.as_uuid()).collect();
                let fed_clause = federation_following_clause(viewer);
                let count_sql = format!(
                    "SELECT COUNT(*) FROM thoughts t WHERE (t.user_id=ANY($1){}) AND t.visibility != 'direct'",
                    fed_clause
                );
                let total: i64 = sqlx::query_scalar(&count_sql)
                    .bind(&ids)
                    .fetch_one(&self.pool)
                    .await
                    .into_domain()?;

                let sel = feed_select(viewer);
                let sql = format!("{sel} WHERE (t.user_id=ANY($1){}) AND t.visibility != 'direct' ORDER BY t.created_at DESC LIMIT $2 OFFSET $3", fed_clause);
                let rows = sqlx::query_as::<_, FeedRow>(&sql)
                    .bind(&ids)
                    .bind(page.limit())
                    .bind(page.offset())
                    .fetch_all(&self.pool)
                    .await
                    .into_domain()?;

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

            FeedScope::Public => {
                let total: i64 = sqlx::query_scalar(
                    "SELECT COUNT(*) FROM thoughts t WHERE t.local=true AND t.visibility='public'",
                )
                .fetch_one(&self.pool)
                .await
                .into_domain()?;

                let sel = feed_select(viewer);
                let sql = format!("{sel} WHERE t.local=true AND t.visibility='public' ORDER BY t.created_at DESC LIMIT $1 OFFSET $2");
                let rows = sqlx::query_as::<_, FeedRow>(&sql)
                    .bind(page.limit())
                    .bind(page.offset())
                    .fetch_all(&self.pool)
                    .await
                    .into_domain()?;

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

            FeedScope::Search { query } => {
                let total: i64 = sqlx::query_scalar(
                    "SELECT COUNT(*) FROM thoughts t WHERE t.content % $1 AND t.visibility='public'",
                )
                .bind(query)
                .fetch_one(&self.pool)
                .await
                .into_domain()?;

                let sel = feed_select(viewer);
                let sql = format!("{sel} WHERE t.content % $1 AND t.visibility='public' ORDER BY similarity(t.content, $1) DESC LIMIT $2 OFFSET $3");
                let rows = sqlx::query_as::<_, FeedRow>(&sql)
                    .bind(query)
                    .bind(page.limit())
                    .bind(page.offset())
                    .fetch_all(&self.pool)
                    .await
                    .into_domain()?;

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

            FeedScope::Tag { tag_name } => {
                let total: i64 = sqlx::query_scalar(
                    "SELECT COUNT(*) FROM thoughts t
                     JOIN thought_tags tt ON tt.thought_id = t.id
                     JOIN tags tg ON tg.id = tt.tag_id
                     WHERE tg.name = $1 AND t.visibility = 'public'",
                )
                .bind(tag_name)
                .fetch_one(&self.pool)
                .await
                .into_domain()?;

                let sel = feed_select(viewer);
                let sql = format!(
                    "{sel}
                     JOIN thought_tags tt ON tt.thought_id = t.id
                     JOIN tags tg ON tg.id = tt.tag_id
                     WHERE tg.name = $1 AND t.visibility = 'public'
                     ORDER BY t.created_at DESC LIMIT $2 OFFSET $3"
                );
                let rows = sqlx::query_as::<_, FeedRow>(&sql)
                    .bind(tag_name)
                    .bind(page.limit())
                    .bind(page.offset())
                    .fetch_all(&self.pool)
                    .await
                    .into_domain()?;

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

            FeedScope::User { user_id } => {
                let uid = user_id.as_uuid();
                // Use nil UUID for unauthenticated viewers — won't match owner or follower checks.
                let viewer_uuid = viewer.unwrap_or(uuid::Uuid::nil());

                let total: i64 = sqlx::query_scalar(
                    "SELECT COUNT(*) FROM thoughts t WHERE t.user_id = $1 AND ($2::uuid = $1 OR (t.visibility != 'direct' AND (t.visibility IN ('public', 'unlisted') OR (t.visibility = 'followers' AND EXISTS(SELECT 1 FROM follows WHERE follower_id = $2 AND following_id = $1 AND state = 'accepted')))))",
                )
                .bind(uid)
                .bind(viewer_uuid)
                .fetch_one(&self.pool)
                .await
                .into_domain()?;

                let sel = feed_select(viewer);
                let sql = format!("{sel} WHERE t.user_id = $1 AND ($4::uuid = $1 OR (t.visibility != 'direct' AND (t.visibility IN ('public', 'unlisted') OR (t.visibility = 'followers' AND EXISTS(SELECT 1 FROM follows WHERE follower_id = $4 AND following_id = $1 AND state = 'accepted'))))) ORDER BY t.created_at DESC LIMIT $2 OFFSET $3");
                let rows = sqlx::query_as::<_, FeedRow>(&sql)
                    .bind(uid)
                    .bind(page.limit())
                    .bind(page.offset())
                    .bind(viewer_uuid)
                    .fetch_all(&self.pool)
                    .await
                    .into_domain()?;

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
        }
    }
}

#[cfg(test)]
mod tests;
