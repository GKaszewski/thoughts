use crate::db_error::IntoDbResult;
use async_trait::async_trait;
use domain::{
    errors::DomainError,
    models::feed::{EngagementStats, ViewerContext},
    ports::EngagementRepository,
    value_objects::{ThoughtId, UserId},
};
use sqlx::PgPool;
use std::collections::HashMap;

pub struct PgEngagementRepository {
    pool: PgPool,
}

impl PgEngagementRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EngagementRepository for PgEngagementRepository {
    async fn get_for_thoughts(
        &self,
        thought_ids: &[ThoughtId],
        viewer_id: Option<&UserId>,
    ) -> Result<HashMap<ThoughtId, (EngagementStats, Option<ViewerContext>)>, DomainError> {
        if thought_ids.is_empty() {
            return Ok(HashMap::new());
        }

        #[derive(sqlx::FromRow)]
        struct Row {
            thought_id: uuid::Uuid,
            like_count: i64,
            boost_count: i64,
            reply_count: i64,
            liked_by_viewer: bool,
            boosted_by_viewer: bool,
        }

        let ids: Vec<uuid::Uuid> = thought_ids.iter().map(|t| t.as_uuid()).collect();
        let viewer_uuid: Option<uuid::Uuid> = viewer_id.map(|v| v.as_uuid());

        let rows = sqlx::query_as::<_, Row>(
            "SELECT
                t.id AS thought_id,
                COUNT(DISTINCT l.user_id) AS like_count,
                COUNT(DISTINCT b.user_id) AS boost_count,
                COUNT(DISTINCT r.id) AS reply_count,
                COALESCE(BOOL_OR(l.user_id = $2), false) AS liked_by_viewer,
                COALESCE(BOOL_OR(b.user_id = $2), false) AS boosted_by_viewer
             FROM thoughts t
             LEFT JOIN likes l ON l.thought_id = t.id
             LEFT JOIN boosts b ON b.thought_id = t.id
             LEFT JOIN thoughts r ON r.in_reply_to_id = t.id
             WHERE t.id = ANY($1)
             GROUP BY t.id",
        )
        .bind(&ids[..])
        .bind(viewer_uuid)
        .fetch_all(&self.pool)
        .await
        .into_domain()?;

        let mut result = HashMap::new();
        for row in rows {
            let tid = ThoughtId::from_uuid(row.thought_id);
            let stats = EngagementStats {
                like_count: row.like_count,
                boost_count: row.boost_count,
                reply_count: row.reply_count,
            };
            let viewer = viewer_id.map(|_| ViewerContext {
                liked: row.liked_by_viewer,
                boosted: row.boosted_by_viewer,
            });
            result.insert(tid, (stats, viewer));
        }
        Ok(result)
    }
}
