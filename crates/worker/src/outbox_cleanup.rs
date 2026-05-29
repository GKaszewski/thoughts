use sqlx::PgPool;
use std::time::Duration;

pub struct OutboxCleanup {
    pub pool: PgPool,
    pub retention_days: i64,
    pub interval: Duration,
}

impl OutboxCleanup {
    pub async fn run(self) {
        loop {
            tokio::time::sleep(self.interval).await;
            match self.cleanup().await {
                Ok(n) if n > 0 => tracing::info!(deleted = n, "outbox cleanup: removed old events"),
                Err(e) => tracing::warn!(error = %e, "outbox cleanup failed"),
                _ => {}
            }
        }
    }

    async fn cleanup(&self) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            "DELETE FROM outbox_events WHERE delivered = true AND delivered_at < NOW() - make_interval(days => $1)",
        )
        .bind(self.retention_days as i32)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }
}
