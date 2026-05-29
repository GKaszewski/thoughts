use crate::{thought::PgThoughtRepository, user::PgUserRepository};
use domain::{
    models::{
        thought::{NewThought, Thought, Visibility},
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
    let t = Thought::new_local(NewThought {
        id: ThoughtId::new(),
        user_id: user.id.clone(),
        content: Content::new_local("hi").unwrap(),
        in_reply_to_id: None,
        visibility: Visibility::Public,
        content_warning: None,
        sensitive: false,
        mood: None,
    });
    trepo.save(&t).await.unwrap();
    (user, t)
}
