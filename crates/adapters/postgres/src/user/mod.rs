use crate::db_error::IntoDbResult;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::{
    errors::DomainError,
    models::feed::{PageParams, Paginated, UserSummary},
    models::user::User,
    ports::{UserReader, UserWriter},
    value_objects::{Email, PasswordHash, UserId, Username},
};
use sqlx::PgPool;
use std::collections::HashMap;

pub struct PgUserRepository {
    pool: PgPool,
}
impl PgUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
pub struct UserRow {
    pub id: uuid::Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub header_url: Option<String>,
    pub custom_css: Option<String>,
    pub local: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<UserRow> for User {
    fn from(r: UserRow) -> Self {
        User {
            id: UserId::from_uuid(r.id),
            username: Username::from_trusted(r.username),
            email: Email::from_trusted(r.email),
            password_hash: PasswordHash(r.password_hash),
            display_name: r.display_name,
            bio: r.bio,
            avatar_url: r.avatar_url,
            header_url: r.header_url,
            custom_css: r.custom_css,
            local: r.local,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

pub const USER_SELECT: &str =
    "SELECT id,username,email,password_hash,display_name,bio,avatar_url,header_url,\
     custom_css,local,created_at,updated_at FROM users";

#[async_trait]
impl UserReader for PgUserRepository {
    async fn find_by_id(&self, id: &UserId) -> Result<Option<User>, DomainError> {
        sqlx::query_as::<_, UserRow>(&format!("{USER_SELECT} WHERE id=$1"))
            .bind(id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .into_domain()
            .map(|o| o.map(User::from))
    }

    async fn find_by_username(&self, username: &Username) -> Result<Option<User>, DomainError> {
        sqlx::query_as::<_, UserRow>(&format!("{USER_SELECT} WHERE username=$1"))
            .bind(username.as_str())
            .fetch_optional(&self.pool)
            .await
            .into_domain()
            .map(|o| o.map(User::from))
    }

    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, DomainError> {
        sqlx::query_as::<_, UserRow>(&format!("{USER_SELECT} WHERE email=$1"))
            .bind(email.as_str())
            .fetch_optional(&self.pool)
            .await
            .into_domain()
            .map(|o| o.map(User::from))
    }

    async fn list_with_stats(&self) -> Result<Vec<UserSummary>, DomainError> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: uuid::Uuid,
            username: String,
            display_name: Option<String>,
            avatar_url: Option<String>,
            bio: Option<String>,
            thought_count: i64,
            follower_count: i64,
            following_count: i64,
        }
        let rows = sqlx::query_as::<_, Row>(
            "SELECT u.id, u.username, u.display_name, u.avatar_url, u.bio,
             COUNT(DISTINCT t.id) AS thought_count,
             COUNT(DISTINCT f1.follower_id) AS follower_count,
             COUNT(DISTINCT f2.following_id) AS following_count
             FROM users u
             LEFT JOIN thoughts t ON t.user_id=u.id AND t.local=true
             LEFT JOIN follows f1 ON f1.following_id=u.id AND f1.state='accepted'
             LEFT JOIN follows f2 ON f2.follower_id=u.id AND f2.state='accepted'
             WHERE u.local=true
             GROUP BY u.id
             ORDER BY u.username",
        )
        .fetch_all(&self.pool)
        .await
        .into_domain()?;

        Ok(rows
            .into_iter()
            .map(|r| UserSummary {
                id: UserId::from_uuid(r.id),
                username: r.username,
                display_name: r.display_name,
                avatar_url: r.avatar_url,
                bio: r.bio,
                thought_count: r.thought_count,
                follower_count: r.follower_count,
                following_count: r.following_count,
            })
            .collect())
    }

    async fn count(&self) -> Result<i64, DomainError> {
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE local = true")
            .fetch_one(&self.pool)
            .await
            .into_domain()
    }

    async fn list_paginated(&self, page: PageParams) -> Result<Paginated<UserSummary>, DomainError> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: uuid::Uuid,
            username: String,
            display_name: Option<String>,
            avatar_url: Option<String>,
            bio: Option<String>,
            thought_count: i64,
            follower_count: i64,
            following_count: i64,
            total: i64,
        }
        let rows = sqlx::query_as::<_, Row>(
            "SELECT u.id, u.username, u.display_name, u.avatar_url, u.bio,
             COUNT(DISTINCT t.id) AS thought_count,
             COUNT(DISTINCT f1.follower_id) AS follower_count,
             COUNT(DISTINCT f2.following_id) AS following_count,
             COUNT(*) OVER() AS total
             FROM users u
             LEFT JOIN thoughts t ON t.user_id=u.id AND t.local=true
             LEFT JOIN follows f1 ON f1.following_id=u.id AND f1.state='accepted'
             LEFT JOIN follows f2 ON f2.follower_id=u.id AND f2.state='accepted'
             WHERE u.local=true
             GROUP BY u.id
             ORDER BY u.username
             LIMIT $1 OFFSET $2",
        )
        .bind(page.limit())
        .bind(page.offset())
        .fetch_all(&self.pool)
        .await
        .into_domain()?;

        let total = rows.first().map(|r| r.total).unwrap_or(0);
        let items = rows
            .into_iter()
            .map(|r| UserSummary {
                id: UserId::from_uuid(r.id),
                username: r.username,
                display_name: r.display_name,
                avatar_url: r.avatar_url,
                bio: r.bio,
                thought_count: r.thought_count,
                follower_count: r.follower_count,
                following_count: r.following_count,
            })
            .collect();
        Ok(Paginated { items, total, page: page.page, per_page: page.per_page })
    }

    async fn find_by_ids(&self, ids: &[UserId]) -> Result<HashMap<UserId, User>, DomainError> {
        if ids.is_empty() {
            return Ok(HashMap::new());
        }
        let uuids: Vec<uuid::Uuid> = ids.iter().map(|id| id.as_uuid()).collect();
        let rows = sqlx::query_as::<_, UserRow>(
            &format!("{USER_SELECT} WHERE id = ANY($1)")
        )
        .bind(&uuids[..])
        .fetch_all(&self.pool)
        .await
        .into_domain()?;

        Ok(rows.into_iter().map(|r| {
            let user = User::from(r);
            (user.id.clone(), user)
        }).collect())
    }
}

#[async_trait]
impl UserWriter for PgUserRepository {
    async fn save(&self, user: &User) -> Result<(), DomainError> {
        sqlx::query(
            "INSERT INTO users (id,username,email,password_hash,display_name,bio,avatar_url,header_url,custom_css,local,created_at,updated_at)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)
             ON CONFLICT(id) DO UPDATE SET
               username=EXCLUDED.username, email=EXCLUDED.email,
               password_hash=EXCLUDED.password_hash, display_name=EXCLUDED.display_name,
               bio=EXCLUDED.bio, avatar_url=EXCLUDED.avatar_url,
               header_url=EXCLUDED.header_url, custom_css=EXCLUDED.custom_css,
               local=EXCLUDED.local,
               updated_at=NOW()"
        )
        .bind(user.id.as_uuid())
        .bind(user.username.as_str())
        .bind(user.email.as_str())
        .bind(&user.password_hash.0)
        .bind(&user.display_name)
        .bind(&user.bio)
        .bind(&user.avatar_url)
        .bind(&user.header_url)
        .bind(&user.custom_css)
        .bind(user.local)
        .bind(user.created_at)
        .bind(user.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(ref db) = e {
                if db.code().as_deref() == Some("23505") {
                    return match db.constraint().unwrap_or("") {
                        "users_username_key" => DomainError::UniqueViolation { field: "username" },
                        "users_email_key"    => DomainError::UniqueViolation { field: "email" },
                        _                   => DomainError::UniqueViolation { field: "unknown" },
                    };
                }
            }
            DomainError::Internal(e.to_string())
        })
        .map(|_| ())
    }

    async fn update_profile(
        &self,
        user_id: &UserId,
        display_name: Option<String>,
        bio: Option<String>,
        avatar_url: Option<String>,
        header_url: Option<String>,
        custom_css: Option<String>,
    ) -> Result<(), DomainError> {
        sqlx::query(
            "UPDATE users SET display_name=$2,bio=$3,avatar_url=$4,header_url=$5,custom_css=$6,updated_at=NOW() WHERE id=$1"
        )
        .bind(user_id.as_uuid())
        .bind(display_name)
        .bind(bio)
        .bind(avatar_url)
        .bind(header_url)
        .bind(custom_css)
        .execute(&self.pool)
        .await
        .into_domain()
        .map(|_| ())
    }
}

#[cfg(test)]
mod tests;
