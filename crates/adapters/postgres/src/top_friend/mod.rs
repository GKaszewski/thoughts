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
        struct TopFriendRow {
            tf_user_id: uuid::Uuid,
            friend_id: uuid::Uuid,
            position: i16,
            #[sqlx(flatten)]
            user: crate::user::UserRow,
        }
        let rows = sqlx::query_as::<_, TopFriendRow>(
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
                let tf = TopFriend {
                    user_id: UserId::from_uuid(r.tf_user_id),
                    friend_id: UserId::from_uuid(r.friend_id),
                    position: r.position,
                };
                (tf, User::from(r.user))
            })
            .collect())
    }
}

#[cfg(test)]
mod tests;
