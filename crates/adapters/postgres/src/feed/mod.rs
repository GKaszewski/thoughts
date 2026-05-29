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
    ports::{FeedOptions, FeedRepository, FeedRequest, FeedScope, FeedSort},
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

struct FeedSqlBuilder<'a> {
    options: &'a FeedOptions,
    scope: &'a FeedScope,
    viewer: Option<uuid::Uuid>,
}

impl<'a> FeedSqlBuilder<'a> {
    fn new(options: &'a FeedOptions, scope: &'a FeedScope, viewer: Option<uuid::Uuid>) -> Self {
        Self {
            options,
            scope,
            viewer,
        }
    }

    fn select(&self, viewer_param: &str) -> String {
        let (viewer_cols, viewer_joins) = match self.viewer {
            Some(_) => (
                "(lv.thought_id IS NOT NULL) AS liked_by_viewer,
                 (bv.thought_id IS NOT NULL) AS boosted_by_viewer".to_string(),
                format!(
                    "LEFT JOIN (SELECT thought_id FROM likes WHERE user_id={viewer_param}) lv ON lv.thought_id = t.id
                     LEFT JOIN (SELECT thought_id FROM boosts WHERE user_id={viewer_param}) bv ON bv.thought_id = t.id"
                ),
            ),
            None => (
                "false AS liked_by_viewer, false AS boosted_by_viewer".to_string(),
                String::new(),
            ),
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
        COALESCE(l_agg.cnt, 0) AS like_count,
        COALESCE(b_agg.cnt, 0) AS boost_count,
        COALESCE(r_agg.cnt, 0) AS reply_count,
        {viewer_cols}
    FROM thoughts t
    JOIN users u ON u.id=t.user_id
    LEFT JOIN remote_actors ra ON u.ap_id = ra.url
    LEFT JOIN (SELECT thought_id, COUNT(*) AS cnt FROM likes GROUP BY thought_id) l_agg ON l_agg.thought_id = t.id
    LEFT JOIN (SELECT thought_id, COUNT(*) AS cnt FROM boosts GROUP BY thought_id) b_agg ON b_agg.thought_id = t.id
    LEFT JOIN (SELECT in_reply_to_id, COUNT(*) AS cnt FROM thoughts WHERE in_reply_to_id IS NOT NULL GROUP BY in_reply_to_id) r_agg ON r_agg.in_reply_to_id = t.id
    {viewer_joins}"
        )
    }

    fn fed_clause(&self, viewer_param: &str) -> String {
        match self.viewer {
            Some(_) => format!(
                " OR t.user_id IN (
                SELECT u2.id FROM users u2
                JOIN federation_following ff ON u2.ap_id = ff.remote_actor_url
                WHERE ff.local_user_id = {viewer_param}
            )"
            ),
            None => String::new(),
        }
    }

    fn filter_sql(&self) -> String {
        let f = &self.options.filter;
        let mut s = String::new();
        if f.originals_only {
            s += " AND t.in_reply_to_id IS NULL";
        }
        if f.replies_only {
            s += " AND t.in_reply_to_id IS NOT NULL";
        }
        if f.local_only {
            s += " AND t.local = true";
        }
        if f.hide_sensitive {
            s += " AND t.sensitive = false";
        }
        s
    }

    fn order_sql(&self) -> &'static str {
        if matches!(self.scope, FeedScope::Search { .. }) {
            return "ORDER BY similarity(t.content, $1) DESC";
        }
        match &self.options.sort {
            FeedSort::Newest => "ORDER BY t.created_at DESC",
            FeedSort::Oldest => "ORDER BY t.created_at ASC",
            FeedSort::MostLiked => "ORDER BY like_count DESC, t.created_at DESC",
            FeedSort::MostBoosted => "ORDER BY boost_count DESC, t.created_at DESC",
            FeedSort::MostDiscussed => "ORDER BY reply_count DESC, t.created_at DESC",
        }
    }

    fn public(&self) -> (String, String) {
        let filter = self.filter_sql();
        let order = self.order_sql();
        let count = format!(
            "SELECT COUNT(*) FROM thoughts t WHERE t.local=true AND t.visibility='public'{}",
            filter
        );
        let data = format!(
            "{} WHERE t.local=true AND t.visibility='public'{} {} LIMIT $1 OFFSET $2",
            self.select("$3"),
            filter,
            order
        );
        (count, data)
    }

    fn home(&self) -> (String, String) {
        let filter = self.filter_sql();
        let order = self.order_sql();
        let count = format!(
            "SELECT COUNT(*) FROM thoughts t WHERE (t.user_id=ANY($1){}) AND t.visibility != 'direct'{}",
            self.fed_clause("$2"), filter
        );
        let data =
            format!(
            "{} WHERE (t.user_id=ANY($1){}) AND t.visibility != 'direct'{} {} LIMIT $2 OFFSET $3",
            self.select("$4"), self.fed_clause("$4"), filter, order
        );
        (count, data)
    }

    fn search(&self) -> (String, String) {
        let filter = self.filter_sql();
        let order = self.order_sql();
        let count = format!(
            "SELECT COUNT(*) FROM thoughts t WHERE t.content % $1 AND t.visibility='public'{}",
            filter
        );
        let data = format!(
            "{} WHERE t.content % $1 AND t.visibility='public'{} {} LIMIT $2 OFFSET $3",
            self.select("$4"),
            filter,
            order
        );
        (count, data)
    }

    fn tag(&self) -> (String, String) {
        let filter = self.filter_sql();
        let order = self.order_sql();
        let count = format!(
            "SELECT COUNT(*) FROM thoughts t
             JOIN thought_tags tt ON tt.thought_id = t.id
             JOIN tags tg ON tg.id = tt.tag_id
             WHERE tg.name = $1 AND t.visibility = 'public'{}",
            filter
        );
        let data = format!(
            "{}
             JOIN thought_tags tt ON tt.thought_id = t.id
             JOIN tags tg ON tg.id = tt.tag_id
             WHERE tg.name = $1 AND t.visibility = 'public'{} {} LIMIT $2 OFFSET $3",
            self.select("$4"),
            filter,
            order
        );
        (count, data)
    }

    fn user(&self) -> (String, String) {
        let filter = self.filter_sql();
        let order = self.order_sql();
        let count  = format!(
            "SELECT COUNT(*) FROM thoughts t WHERE t.user_id = $1 AND ($2::uuid = $1 OR (t.visibility != 'direct' AND (t.visibility IN ('public', 'unlisted') OR (t.visibility = 'followers' AND EXISTS(SELECT 1 FROM follows WHERE follower_id = $2 AND following_id = $1 AND state = 'accepted'))))){}",
            filter
        );
        let data = format!(
            "{} WHERE t.user_id = $1 AND ($4::uuid = $1 OR (t.visibility != 'direct' AND (t.visibility IN ('public', 'unlisted') OR (t.visibility = 'followers' AND EXISTS(SELECT 1 FROM follows WHERE follower_id = $4 AND following_id = $1 AND state = 'accepted'))))){} {} LIMIT $2 OFFSET $3",
            self.select("$4"), filter, order
        );
        (count, data)
    }
}

#[async_trait]
impl FeedRepository for PgFeedRepository {
    async fn query(&self, req: &FeedRequest) -> Result<Paginated<FeedEntry>, DomainError> {
        let viewer = req.query.viewer_id.as_ref().map(|v| v.as_uuid());
        let page = &req.query.page;
        let builder = FeedSqlBuilder::new(&req.options, &req.query.scope, viewer);

        let viewer_uuid = viewer.unwrap_or(uuid::Uuid::nil());

        match &req.query.scope {
            FeedScope::Home { following_ids } => {
                let ids: Vec<uuid::Uuid> = following_ids.iter().map(|id| id.as_uuid()).collect();
                let (count_sql, data_sql) = builder.home();
                let total: i64 = sqlx::query_scalar(&count_sql)
                    .bind(&ids)
                    .bind(viewer_uuid)
                    .fetch_one(&self.pool)
                    .await
                    .into_domain()?;
                let rows = sqlx::query_as::<_, FeedRow>(&data_sql)
                    .bind(&ids)
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

            FeedScope::Public => {
                let (count_sql, data_sql) = builder.public();
                let total: i64 = sqlx::query_scalar(&count_sql)
                    .fetch_one(&self.pool)
                    .await
                    .into_domain()?;
                let rows = sqlx::query_as::<_, FeedRow>(&data_sql)
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

            FeedScope::Search { query } => {
                let (count_sql, data_sql) = builder.search();
                let total: i64 = sqlx::query_scalar(&count_sql)
                    .bind(query)
                    .fetch_one(&self.pool)
                    .await
                    .into_domain()?;
                let rows = sqlx::query_as::<_, FeedRow>(&data_sql)
                    .bind(query)
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

            FeedScope::Tag { tag_name } => {
                let (count_sql, data_sql) = builder.tag();
                let total: i64 = sqlx::query_scalar(&count_sql)
                    .bind(tag_name)
                    .fetch_one(&self.pool)
                    .await
                    .into_domain()?;
                let rows = sqlx::query_as::<_, FeedRow>(&data_sql)
                    .bind(tag_name)
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

            FeedScope::User { user_id } => {
                let uid = user_id.as_uuid();
                let (count_sql, data_sql) = builder.user();
                let total: i64 = sqlx::query_scalar(&count_sql)
                    .bind(uid)
                    .bind(viewer_uuid)
                    .fetch_one(&self.pool)
                    .await
                    .into_domain()?;
                let rows = sqlx::query_as::<_, FeedRow>(&data_sql)
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
