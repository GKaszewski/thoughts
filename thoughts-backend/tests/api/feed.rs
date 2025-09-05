use super::main::{create_test_user, setup};
use axum::http::StatusCode;
use http_body_util::BodyExt;
use serde_json::json;
use utils::testing::{make_get_request, make_post_request};

#[tokio::test]
async fn test_feed_and_user_thoughts() {
    let app = setup().await;
    create_test_user(&app.db, "user1").await; // AuthUser is ID 1
    create_test_user(&app.db, "user2").await;
    create_test_user(&app.db, "user3").await;

    // As user1, post a thought
    let body = json!({ "content": "A thought from user1" }).to_string();
    make_post_request(app.router.clone(), "/thoughts", body).await;

    // As a different "user", create thoughts for user2 and user3 (we cheat here since auth is hardcoded)
    app::persistence::thought::create_thought(
        &app.db,
        2,
        models::params::thought::CreateThoughtParams {
            content: "user2 was here".to_string(),
        },
    )
    .await
    .unwrap();
    app::persistence::thought::create_thought(
        &app.db,
        3,
        models::params::thought::CreateThoughtParams {
            content: "user3 checking in".to_string(),
        },
    )
    .await
    .unwrap();

    // 1. Get thoughts for user2 - should only see their thought
    let response = make_get_request(app.router.clone(), "/users/user2/thoughts").await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["thoughts"].as_array().unwrap().len(), 1);
    assert_eq!(v["thoughts"][0]["content"], "user2 was here");

    // 2. user1's feed is initially empty
    let response = make_get_request(app.router.clone(), "/feed").await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(v["thoughts"].as_array().unwrap().is_empty());

    // 3. user1 follows user2
    make_post_request(app.router.clone(), "/users/user2/follow", "".to_string()).await;

    // 4. user1's feed now has user2's thought
    let response = make_get_request(app.router.clone(), "/feed").await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["thoughts"].as_array().unwrap().len(), 1);
    assert_eq!(v["thoughts"][0]["author_username"], "user2");
}
