use std::time::Duration;

use super::main::{create_user_with_password, setup};
use axum::http::StatusCode;
use http_body_util::BodyExt;
use serde_json::json;
use tokio::time::sleep;
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
    assert_eq!(v["thoughts"][0]["authorUsername"], "user1");
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
    assert_eq!(v["thoughts"][0]["authorUsername"], "user2");
    assert_eq!(v["thoughts"][0]["content"], "user2 was here");
    assert_eq!(v["thoughts"][1]["authorUsername"], "user1");
    assert_eq!(v["thoughts"][1]["content"], "A thought from user1");
}

#[tokio::test]
async fn test_feed_strict_chronological_order() {
    let app = setup().await;
    create_user_with_password(&app.db, "user1", "password123", "u1@e.com").await;
    create_user_with_password(&app.db, "user2", "password123", "u2@e.com").await;
    create_user_with_password(&app.db, "user3", "password123", "u3@e.com").await;

    let token1 = super::main::login_user(app.router.clone(), "user1", "password123").await;
    let token2 = super::main::login_user(app.router.clone(), "user2", "password123").await;
    let token3 = super::main::login_user(app.router.clone(), "user3", "password123").await;

    make_jwt_request(
        app.router.clone(),
        "/users/user2/follow",
        "POST",
        None,
        &token1,
    )
    .await;
    make_jwt_request(
        app.router.clone(),
        "/users/user3/follow",
        "POST",
        None,
        &token1,
    )
    .await;

    let body_t1 = json!({ "content": "Thought 1 from user2" }).to_string();
    make_jwt_request(
        app.router.clone(),
        "/thoughts",
        "POST",
        Some(body_t1),
        &token2,
    )
    .await;
    sleep(Duration::from_millis(10)).await;

    let body_t2 = json!({ "content": "Thought 2 from user3" }).to_string();
    make_jwt_request(
        app.router.clone(),
        "/thoughts",
        "POST",
        Some(body_t2),
        &token3,
    )
    .await;
    sleep(Duration::from_millis(10)).await;

    let body_t3 = json!({ "content": "Thought 3 from user2" }).to_string();
    make_jwt_request(
        app.router.clone(),
        "/thoughts",
        "POST",
        Some(body_t3),
        &token2,
    )
    .await;

    let response = make_jwt_request(app.router.clone(), "/feed", "GET", None, &token1).await;
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let thoughts = v["thoughts"].as_array().unwrap();

    assert_eq!(
        thoughts.len(),
        3,
        "Feed should contain 3 thoughts from followed users"
    );
    assert_eq!(thoughts[0]["content"], "Thought 3 from user2");
    assert_eq!(thoughts[1]["content"], "Thought 2 from user3");
    assert_eq!(thoughts[2]["content"], "Thought 1 from user2");
}
