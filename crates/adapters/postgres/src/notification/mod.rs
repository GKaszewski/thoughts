use crate::db_error::IntoDbResult;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

use domain::{
    errors::DomainError,
    models::{
        feed::{PageParams, Paginated},
        notification::{Notification, NotificationKind},
    },
    ports::NotificationRepository,
    value_objects::{NotificationId, ThoughtId, UserId},
};
use sqlx::PgPool;

pub struct PgNotificationRepository {
    pool: PgPool,
}
impl PgNotificationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct NotificationRow {
    id: uuid::Uuid,
    user_id: uuid::Uuid,
    notification_type: String,
    from_user_id: Option<uuid::Uuid>,
    thought_id: Option<uuid::Uuid>,
    read: bool,
    created_at: DateTime<Utc>,
}

fn row_to_notification(r: NotificationRow) -> Result<Notification, DomainError> {
    let from_user_id = r
        .from_user_id
        .map(UserId::from_uuid)
        .ok_or_else(|| DomainError::Internal("notification missing from_user_id".into()))?;

    let kind = match r.notification_type.as_str() {
        "follow" => NotificationKind::Follow { from_user_id },
        other => {
            let thought_id = r.thought_id.map(ThoughtId::from_uuid).ok_or_else(|| {
                DomainError::Internal(format!("notification type '{other}' missing thought_id"))
            })?;
            match other {
                "like" => NotificationKind::Like {
                    thought_id,
                    from_user_id,
                },
                "boost" => NotificationKind::Boost {
                    thought_id,
                    from_user_id,
                },
                "reply" => NotificationKind::Reply {
                    thought_id,
                    from_user_id,
                },
                "mention" => NotificationKind::Mention {
                    thought_id,
                    from_user_id,
                },
                _ => {
                    return Err(DomainError::Internal(format!(
                        "unknown notification type: {other}"
                    )))
                }
            }
        }
    };

    Ok(Notification {
        id: NotificationId::from_uuid(r.id),
        user_id: UserId::from_uuid(r.user_id),
        kind,
        read: r.read,
        created_at: r.created_at,
    })
}

#[async_trait]
impl NotificationRepository for PgNotificationRepository {
    async fn save(&self, n: &Notification) -> Result<(), DomainError> {
        sqlx::query(
            "INSERT INTO notifications(id,user_id,notification_type,from_user_id,thought_id,read,created_at)
             VALUES($1,$2,$3,$4,$5,$6,$7)
             ON CONFLICT(id) DO NOTHING"
        )
        .bind(n.id.as_uuid())
        .bind(n.user_id.as_uuid())
        .bind(n.kind.kind_str())
        .bind(n.kind.from_user_id().as_uuid())
        .bind(n.kind.thought_id().map(|t| t.as_uuid()))
        .bind(n.read)
        .bind(n.created_at)
        .execute(&self.pool)
        .await
        .into_domain()
        .map(|_| ())
    }

    async fn list_for_user(
        &self,
        user_id: &UserId,
        page: &PageParams,
    ) -> Result<Paginated<Notification>, DomainError> {
        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM notifications WHERE user_id=$1")
            .bind(user_id.as_uuid())
            .fetch_one(&self.pool)
            .await
            .into_domain()?;
        let rows = sqlx::query_as::<_, NotificationRow>(
            "SELECT id,user_id,notification_type,from_user_id,thought_id,read,created_at FROM notifications WHERE user_id=$1 ORDER BY created_at DESC LIMIT $2 OFFSET $3"
        ).bind(user_id.as_uuid()).bind(page.limit()).bind(page.offset())
        .fetch_all(&self.pool).await.into_domain()?;
        let items = rows
            .into_iter()
            .map(row_to_notification)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Paginated {
            items,
            total,
            page: page.page,
            per_page: page.per_page,
        })
    }

    async fn count_unread(&self, user_id: &UserId) -> Result<u64, DomainError> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM notifications WHERE user_id=$1 AND read=false",
        )
        .bind(user_id.as_uuid())
        .fetch_one(&self.pool)
        .await
        .into_domain()?;
        Ok(count as u64)
    }

    async fn mark_read(&self, id: &NotificationId, user_id: &UserId) -> Result<(), DomainError> {
        sqlx::query("UPDATE notifications SET read=true WHERE id=$1 AND user_id=$2")
            .bind(id.as_uuid())
            .bind(user_id.as_uuid())
            .execute(&self.pool)
            .await
            .into_domain()
            .map(|_| ())
    }

    async fn mark_all_read(&self, user_id: &UserId) -> Result<(), DomainError> {
        sqlx::query("UPDATE notifications SET read=true WHERE user_id=$1")
            .bind(user_id.as_uuid())
            .execute(&self.pool)
            .await
            .into_domain()
            .map(|_| ())
    }
}

#[cfg(test)]
mod tests;
