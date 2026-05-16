use crate::db_error::IntoDbResult;
use async_trait::async_trait;
use domain::{
    errors::DomainError,
    models::{top_friend::TopFriend, user::User},
    ports::TopFriendRepository,
    value_objects::UserId,
};
use sqlx::PgPool;

pub struct PgTopFriendRepository {
    pool: PgPool,
}
impl PgTopFriendRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TopFriendRepository for PgTopFriendRepository {
    async fn set_top_friends(
        &self,
        user_id: &UserId,
        friends: Vec<(UserId, i16)>,
    ) -> Result<(), DomainError> {
        let mut tx = self.pool.begin().await.into_domain()?;
        sqlx::query("DELETE FROM top_friends WHERE user_id=$1")
            .bind(user_id.as_uuid())
            .execute(&mut *tx)
            .await
            .into_domain()?;
        for (friend_id, pos) in friends {
            sqlx::query("INSERT INTO top_friends(user_id,friend_id,position) VALUES($1,$2,$3)")
                .bind(user_id.as_uuid())
                .bind(friend_id.as_uuid())
                .bind(pos)
                .execute(&mut *tx)
                .await
                .into_domain()?;
        }
        tx.commit().await.into_domain()
    }

    async fn list_for_user(&self, user_id: &UserId) -> Result<Vec<(TopFriend, User)>, DomainError> {
        #[derive(sqlx::FromRow)]
        struct Row {
            tf_user_id: uuid::Uuid,
            friend_id: uuid::Uuid,
            position: i16,
            id: uuid::Uuid,
            username: String,
            email: String,
            password_hash: String,
            display_name: Option<String>,
            bio: Option<String>,
            avatar_url: Option<String>,
            header_url: Option<String>,
            custom_css: Option<String>,
            local: bool,
            created_at: chrono::DateTime<chrono::Utc>,
            updated_at: chrono::DateTime<chrono::Utc>,
        }
        let rows = sqlx::query_as::<_, Row>(
            "SELECT tf.user_id AS tf_user_id, tf.friend_id, tf.position,
             u.id, u.username, u.email, u.password_hash, u.display_name, u.bio,
             u.avatar_url, u.header_url, u.custom_css, u.local,
             u.created_at, u.updated_at
             FROM top_friends tf JOIN users u ON u.id=tf.friend_id
             WHERE tf.user_id=$1 ORDER BY tf.position",
        )
        .bind(user_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .into_domain()?;

        Ok(rows
            .into_iter()
            .map(|r| {
                use domain::value_objects::{Email, PasswordHash, Username};
                let tf = TopFriend {
                    user_id: UserId::from_uuid(r.tf_user_id),
                    friend_id: UserId::from_uuid(r.friend_id),
                    position: r.position,
                };
                let u = User {
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
                };
                (tf, u)
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::user::PgUserRepository;
    use domain::ports::UserWriter;
    use domain::{models::user::User, value_objects::*};

    async fn seed_user(pool: &sqlx::PgPool, username: &str, email: &str) -> User {
        let repo = PgUserRepository::new(pool.clone());
        let u = User::new_local(
            UserId::new(),
            Username::new(username).unwrap(),
            Email::new(email).unwrap(),
            PasswordHash("h".into()),
        );
        repo.save(&u).await.unwrap();
        u
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn set_and_list_top_friends(pool: sqlx::PgPool) {
        let alice = seed_user(&pool, "alice", "alice@ex.com").await;
        let bob = seed_user(&pool, "bob", "bob@ex.com").await;
        let repo = PgTopFriendRepository::new(pool);
        repo.set_top_friends(&alice.id, vec![(bob.id.clone(), 1)])
            .await
            .unwrap();
        let friends = repo.list_for_user(&alice.id).await.unwrap();
        assert_eq!(friends.len(), 1);
        assert_eq!(friends[0].0.position, 1);
        assert_eq!(friends[0].1.username.as_str(), "bob");
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn replace_top_friends(pool: sqlx::PgPool) {
        let alice = seed_user(&pool, "alice", "alice@ex.com").await;
        let bob = seed_user(&pool, "bob", "bob@ex.com").await;
        let carol = seed_user(&pool, "carol", "carol@ex.com").await;
        let repo = PgTopFriendRepository::new(pool);
        repo.set_top_friends(&alice.id, vec![(bob.id.clone(), 1)])
            .await
            .unwrap();
        repo.set_top_friends(&alice.id, vec![(carol.id.clone(), 1)])
            .await
            .unwrap();
        let friends = repo.list_for_user(&alice.id).await.unwrap();
        assert_eq!(friends.len(), 1);
        assert_eq!(friends[0].1.username.as_str(), "carol");
    }
}
