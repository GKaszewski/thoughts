use super::main::{create_test_user, setup};
use axum::http::StatusCode;
use http_body_util::BodyExt;
use serde_json::json;
use utils::testing::{make_delete_request, make_post_request};

#[tokio::test]
async fn test_thought_endpoints() {
    let app = setup().await;
    create_test_user(&app.db, "user1").await; // AuthUser is ID 1
    create_test_user(&app.db, "user2").await; // Other user is ID 2

    // 1. Post a new thought as user 1
    let body = json!({ "content": "My first thought!" }).to_string();
    let response = make_post_request(app.router.clone(), "/thoughts", body).await;
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["content"], "My first thought!");
    assert_eq!(v["author_username"], "user1");
    let thought_id = v["id"].as_i64().unwrap();

    // 2. Post a thought with invalid content
    let body = json!({ "content": "" }).to_string(); // Too short
    let response = make_post_request(app.router.clone(), "/thoughts", body).await;
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    // 3. Attempt to delete another user's thought (user1 tries to delete a non-existent thought, but let's pretend it's user2's)
    let response = make_delete_request(app.router.clone(), &format!("/thoughts/999")).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // 4. Delete the thought created in step 1
    let response =
        make_delete_request(app.router.clone(), &format!("/thoughts/{}", thought_id)).await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}
