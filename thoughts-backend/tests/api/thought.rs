use crate::api::main::create_user_with_password;

use super::main::setup;
use axum::http::StatusCode;
use http_body_util::BodyExt;
use sea_orm::prelude::Uuid;
use serde_json::{json, Value};
use utils::testing::{make_delete_request, make_post_request};

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
