use crate::db_error::IntoDbResult;
use async_trait::async_trait;
use domain::{
    errors::DomainError,
    models::{
        feed::{PageParams, Paginated},
        tag::Tag,
        thought::Thought,
    },
    ports::TagRepository,
    value_objects::ThoughtId,
};
use sqlx::PgPool;

#[derive(sqlx::FromRow)]
struct TagRow {
    id: i32,
    name: String,
}

pub struct PgTagRepository {
    pool: PgPool,
}
impl PgTagRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TagRepository for PgTagRepository {
    async fn find_or_create(&self, name: &str) -> Result<Tag, DomainError> {
        let name = name.to_lowercase();
        sqlx::query("INSERT INTO tags(name) VALUES($1) ON CONFLICT(name) DO NOTHING")
            .bind(&name)
            .execute(&self.pool)
            .await
            .into_domain()?;
        let row = sqlx::query_as::<_, TagRow>("SELECT id,name FROM tags WHERE name=$1")
            .bind(&name)
            .fetch_one(&self.pool)
            .await
            .into_domain()?;
        Ok(Tag {
            id: row.id,
            name: row.name,
        })
    }

    async fn attach_to_thought(
        &self,
        thought_id: &ThoughtId,
        tag_id: i32,
    ) -> Result<(), DomainError> {
        sqlx::query(
            "INSERT INTO thought_tags(thought_id,tag_id) VALUES($1,$2) ON CONFLICT DO NOTHING",
        )
        .bind(thought_id.as_uuid())
        .bind(tag_id)
        .execute(&self.pool)
        .await
        .into_domain()
        .map(|_| ())
    }

    async fn detach_from_thought(&self, thought_id: &ThoughtId) -> Result<(), DomainError> {
        sqlx::query("DELETE FROM thought_tags WHERE thought_id=$1")
            .bind(thought_id.as_uuid())
            .execute(&self.pool)
            .await
            .into_domain()
            .map(|_| ())
    }

    async fn list_for_thought(&self, thought_id: &ThoughtId) -> Result<Vec<Tag>, DomainError> {
        sqlx::query_as::<_, TagRow>(
            "SELECT t.id,t.name FROM tags t JOIN thought_tags tt ON tt.tag_id=t.id WHERE tt.thought_id=$1"
        ).bind(thought_id.as_uuid()).fetch_all(&self.pool).await
        .into_domain()
        .map(|rows| rows.into_iter().map(|r| Tag { id: r.id, name: r.name }).collect())
    }

    async fn list_thoughts_by_tag(
        &self,
        tag_name: &str,
        page: &PageParams,
    ) -> Result<Paginated<Thought>, DomainError> {
        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM thought_tags tt JOIN tags t ON t.id=tt.tag_id WHERE t.name=$1",
        )
        .bind(tag_name)
        .fetch_one(&self.pool)
        .await
        .into_domain()?;

        let rows = sqlx::query_as::<_, crate::thought::ThoughtRow>(
            "SELECT th.id,th.user_id,th.content,th.in_reply_to_id,th.in_reply_to_url,th.ap_id,th.visibility,th.content_warning,th.sensitive,th.local,th.created_at,th.updated_at
             FROM thoughts th JOIN thought_tags tt ON tt.thought_id=th.id JOIN tags t ON t.id=tt.tag_id
             WHERE t.name=$1 ORDER BY th.created_at DESC LIMIT $2 OFFSET $3"
        ).bind(tag_name).bind(page.limit()).bind(page.offset())
        .fetch_all(&self.pool).await.into_domain()?;

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

    async fn popular_tags(&self, limit: usize) -> Result<Vec<(String, i64)>, DomainError> {
        sqlx::query_as::<_, (String, i64)>(
            "SELECT t.name, COUNT(tt.thought_id) AS thought_count
             FROM tags t
             JOIN thought_tags tt ON t.id = tt.tag_id
             GROUP BY t.id, t.name
             ORDER BY thought_count DESC
             LIMIT $1",
        )
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .into_domain()
    }
}

#[cfg(test)]
mod tests;
