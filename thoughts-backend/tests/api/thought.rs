use crate::api::main::{create_user_with_password, login_user};

use super::main::setup;
use app::persistence::follow;
use axum::{http::StatusCode, Router};
use http_body_util::BodyExt;
use sea_orm::prelude::Uuid;
use serde_json::{json, Value};
use utils::testing::{make_delete_request, make_get_request, make_jwt_request, make_post_request};

#[tokio::test]
async fn test_thought_endpoints() {
    let app = setup().await;
    let user1 =
        create_user_with_password(&app.db, "user1", "password123", "user1@example.com").await; // AuthUser is ID 1
    let _user2 =
        create_user_with_password(&app.db, "user2", "password123", "user2@example.com").await; // Other user is ID 2

    // 1. Post a new thought as user 1
    let body = json!({ "content": "My first thought!" }).to_string();
    let response = make_post_request(app.router.clone(), "/thoughts", body, Some(user1.id)).await;
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["content"], "My first thought!");
    assert_eq!(v["author_username"], "user1");
    let thought_id = v["id"].as_str().unwrap().to_string();

    // 2. Post a thought with invalid content
    let body = json!({ "content": "" }).to_string(); // Too short
    let response = make_post_request(app.router.clone(), "/thoughts", body, Some(user1.id)).await;
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    // 3. Attempt to delete another user's thought (user1 tries to delete a non-existent thought, but let's pretend it's user2's)
    let response = make_delete_request(
        app.router.clone(),
        &format!("/thoughts/{}", Uuid::new_v4()),
        Some(user1.id),
    )
    .await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // 4. Delete the thought created in step 1
    let response = make_delete_request(
        app.router.clone(),
        &format!("/thoughts/{}", thought_id),
        Some(user1.id),
    )
    .await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_thought_replies() {
    let app = setup().await;
    let user1 =
        create_user_with_password(&app.db, "user1", "password123", "user1@example.com").await;
    let user2 =
        create_user_with_password(&app.db, "user2", "password123", "user2@example.com").await;

    // 1. User 1 posts an original thought
    let body = json!({ "content": "This is the original post!" }).to_string();
    let response = make_post_request(app.router.clone(), "/thoughts", body, Some(user1.id)).await;
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let original_thought: Value = serde_json::from_slice(&body).unwrap();
    let original_thought_id = original_thought["id"].as_str().unwrap();

    // 2. User 2 replies to the original thought
    let reply_body = json!({
        "content": "This is a reply.",
        "reply_to_id": original_thought_id
    })
    .to_string();
    let response =
        make_post_request(app.router.clone(), "/thoughts", reply_body, Some(user2.id)).await;
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let reply_thought: Value = serde_json::from_slice(&body).unwrap();

    // 3. Verify the reply is linked correctly
    assert_eq!(reply_thought["reply_to_id"], original_thought_id);
    assert_eq!(reply_thought["author_username"], "user2");
}

#[tokio::test]
async fn test_thought_visibility() {
    let app = setup().await;
    let author = create_user_with_password(&app.db, "author", "password123", "a@a.com").await;
    let friend = create_user_with_password(&app.db, "friend", "password123", "f@f.com").await;
    let _stranger = create_user_with_password(&app.db, "stranger", "password123", "s@s.com").await;

    // Make author and friend follow each other
    follow::follow_user(&app.db, author.id, friend.id)
        .await
        .unwrap();
    follow::follow_user(&app.db, friend.id, author.id)
        .await
        .unwrap();

    let author_jwt = login_user(app.router.clone(), "author", "password123").await;
    let friend_jwt = login_user(app.router.clone(), "friend", "password123").await;
    let stranger_jwt = login_user(app.router.clone(), "stranger", "password123").await;

    // Author posts one of each visibility
    make_jwt_request(
        app.router.clone(),
        "/thoughts",
        "POST",
        Some(json!({"content": "public", "visibility": "Public"}).to_string()),
        &author_jwt,
    )
    .await;
    make_jwt_request(
        app.router.clone(),
        "/thoughts",
        "POST",
        Some(json!({"content": "friends", "visibility": "FriendsOnly"}).to_string()),
        &author_jwt,
    )
    .await;
    make_jwt_request(
        app.router.clone(),
        "/thoughts",
        "POST",
        Some(json!({"content": "private", "visibility": "Private"}).to_string()),
        &author_jwt,
    )
    .await;

    // Helper to get thoughts and count them
    async fn get_thought_count(router: Router, jwt: Option<&str>) -> usize {
        let response = if let Some(token) = jwt {
            make_jwt_request(router, "/users/author/thoughts", "GET", None, token).await
        } else {
            make_get_request(router, "/users/author/thoughts", None).await
        };
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let v: Value = serde_json::from_slice(&body).unwrap();

        v["thoughts"].as_array().unwrap().len()
    }

    // Assertions
    assert_eq!(
        get_thought_count(app.router.clone(), Some(&author_jwt)).await,
        3,
        "Author should see all their posts"
    );
    assert_eq!(
        get_thought_count(app.router.clone(), Some(&friend_jwt)).await,
        2,
        "Friend should see public and friends_only posts"
    );
    assert_eq!(
        get_thought_count(app.router.clone(), Some(&stranger_jwt)).await,
        1,
        "Stranger should see only public posts"
    );
    assert_eq!(
        get_thought_count(app.router.clone(), None).await,
        1,
        "Unauthenticated guest should see only public posts"
    );
}
