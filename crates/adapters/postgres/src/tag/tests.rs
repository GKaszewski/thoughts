
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
