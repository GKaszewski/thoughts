
use super::*;
use activitypub_base::ActivityPubRepository;

#[sqlx::test(migrations = "./migrations")]
async fn intern_remote_actor_is_idempotent(pool: sqlx::PgPool) {
    let repo = PgActivityPubRepository::new(pool);
    let url = "https://mastodon.social/users/alice";
    let id1 = repo.intern_remote_actor(url).await.unwrap();
    let id2 = repo.intern_remote_actor(url).await.unwrap();
    assert_eq!(id1, id2);
}

#[sqlx::test(migrations = "./migrations")]
async fn accept_and_retract_note(pool: sqlx::PgPool) {
    let repo = PgActivityPubRepository::new(pool);
    let actor_url = "https://remote.example/users/bob";
    let ap_id = "https://remote.example/notes/1";
    let author = repo.intern_remote_actor(actor_url).await.unwrap();
    repo.accept_note(
        ap_id,
        &author,
        "hello from remote",
        chrono::Utc::now(),
        false,
        None,
        "public",
        None,
    )
    .await
    .unwrap();
    repo.retract_note(ap_id).await.unwrap();
}

#[sqlx::test(migrations = "./migrations")]
async fn count_local_notes_excludes_remote(pool: sqlx::PgPool) {
    let repo = PgActivityPubRepository::new(pool);
    assert_eq!(repo.count_local_notes().await.unwrap(), 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn accept_note_returns_thought_id(pool: sqlx::PgPool) {
    let repo = PgActivityPubRepository::new(pool.clone());
    let actor_user_id = repo
        .intern_remote_actor("https://remote.example/users/alice")
        .await
        .unwrap();

    let thought_id = repo
        .accept_note(
            "https://remote.example/notes/1",
            &actor_user_id,
            "Hello #rust world",
            chrono::Utc::now(),
            false,
            None,
            "public",
            None,
        )
        .await
        .unwrap();

    let row: (uuid::Uuid,) = sqlx::query_as("SELECT id FROM thoughts WHERE ap_id=$1")
        .bind("https://remote.example/notes/1")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(thought_id.as_uuid(), row.0);
}
