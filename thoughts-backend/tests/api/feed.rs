use super::main::{create_user_with_password, setup};
use axum::http::StatusCode;
use http_body_util::BodyExt;
use serde_json::json;
use utils::testing::make_jwt_request;

#[tokio::test]
async fn test_feed_and_user_thoughts() {
    let app = setup().await;
    create_user_with_password(&app.db, "user1", "password1", "user1@example.com").await;
    create_user_with_password(&app.db, "user2", "password2", "user2@example.com").await;
    create_user_with_password(&app.db, "user3", "password3", "user3@example.com").await;

    // As user1, post a thought
    let token = super::main::login_user(app.router.clone(), "user1", "password1").await;
    let body = json!({ "content": "A thought from user1" }).to_string();
    make_jwt_request(app.router.clone(), "/thoughts", "POST", Some(body), &token).await;

    // As a different "user", create thoughts for user2 and user3
    let token2 = super::main::login_user(app.router.clone(), "user2", "password2").await;
    let body2 = json!({ "content": "user2 was here" }).to_string();
    make_jwt_request(
        app.router.clone(),
        "/thoughts",
        "POST",
        Some(body2),
        &token2,
    )
    .await;

    let token3 = super::main::login_user(app.router.clone(), "user3", "password3").await;
    let body3 = json!({ "content": "user3 checking in" }).to_string();
    make_jwt_request(
        app.router.clone(),
        "/thoughts",
        "POST",
        Some(body3),
        &token3,
    )
    .await;

    // 1. Get thoughts for user2 - should only see their thought plus their own
    let response = make_jwt_request(
        app.router.clone(),
        "/users/user2/thoughts",
        "GET",
        None,
        &token2,
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["thoughts"].as_array().unwrap().len(), 1);
    assert_eq!(v["thoughts"][0]["content"], "user2 was here");

    // 2. user1's feed has only their own thought (not following anyone)
    let response = make_jwt_request(app.router.clone(), "/feed", "GET", None, &token).await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["thoughts"].as_array().unwrap().len(), 1);
    assert_eq!(v["thoughts"][0]["author_username"], "user1");
    assert_eq!(v["thoughts"][0]["content"], "A thought from user1");

    // 3. user1 follows user2
    make_jwt_request(
        app.router.clone(),
        "/users/user2/follow",
        "POST",
        None,
        &token,
    )
    .await;

    // 4. user1's feed now has user2's thought
    let response = make_jwt_request(app.router.clone(), "/feed", "GET", None, &token).await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["thoughts"].as_array().unwrap().len(), 2);
    assert_eq!(v["thoughts"][0]["author_username"], "user2");
    assert_eq!(v["thoughts"][0]["content"], "user2 was here");
    assert_eq!(v["thoughts"][1]["author_username"], "user1");
    assert_eq!(v["thoughts"][1]["content"], "A thought from user1");
}
