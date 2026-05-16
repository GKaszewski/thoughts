use crate::{thought::PgThoughtRepository, user::PgUserRepository};
use domain::{
    models::{
        thought::{Thought, Visibility},
        user::User,
    },
    ports::{ThoughtRepository, UserWriter},
    value_objects::{Content, Email, PasswordHash, ThoughtId, UserId, Username},
};

pub async fn seed_user(pool: &sqlx::PgPool, username: &str, email: &str) -> User {
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

pub async fn seed_user_and_thought(pool: &sqlx::PgPool) -> (User, Thought) {
    let user = seed_user(pool, "alice", "alice@ex.com").await;
    let trepo = PgThoughtRepository::new(pool.clone());
    let t = Thought::new_local(
        ThoughtId::new(),
        user.id.clone(),
        Content::new_local("hi").unwrap(),
        None,
        Visibility::Public,
        None,
        false,
    );
    trepo.save(&t).await.unwrap();
    (user, t)
}
