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
        #[derive(sqlx::FromRow)]
        struct Row {
            id: i32,
            name: String,
        }
        let row = sqlx::query_as::<_, Row>("SELECT id,name FROM tags WHERE name=$1")
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
        #[derive(sqlx::FromRow)]
        struct Row {
            id: i32,
            name: String,
        }
        sqlx::query_as::<_, Row>(
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
mod tests {
    use super::*;
    use crate::{thought::PgThoughtRepository, user::PgUserRepository};
    use domain::ports::{ThoughtRepository, UserWriter};
    use domain::{
        models::{
            thought::{Thought, Visibility},
            user::User,
        },
        value_objects::*,
    };

    #[sqlx::test(migrations = "./migrations")]
    async fn find_or_create_tag(pool: sqlx::PgPool) {
        let repo = PgTagRepository::new(pool);
        let t1 = repo.find_or_create("rust").await.unwrap();
        let t2 = repo.find_or_create("rust").await.unwrap();
        assert_eq!(t1.id, t2.id);
        assert_eq!(t1.name, "rust");
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn attach_and_list(pool: sqlx::PgPool) {
        let urepo = PgUserRepository::new(pool.clone());
        let trepo = PgThoughtRepository::new(pool.clone());
        let u = User::new_local(
            UserId::new(),
            Username::new("alice").unwrap(),
            Email::new("alice@ex.com").unwrap(),
            PasswordHash("h".into()),
        );
        urepo.save(&u).await.unwrap();
        let t = Thought::new_local(
            ThoughtId::new(),
            u.id.clone(),
            Content::new_local("hi").unwrap(),
            None,
            Visibility::Public,
            None,
            false,
        );
        trepo.save(&t).await.unwrap();
        let repo = PgTagRepository::new(pool);
        let tag = repo.find_or_create("greetings").await.unwrap();
        repo.attach_to_thought(&t.id, tag.id).await.unwrap();
        let tags = repo.list_for_thought(&t.id).await.unwrap();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "greetings");
    }
}
