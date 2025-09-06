use axum::http::StatusCode;
use http_body_util::BodyExt;
use serde_json::{json, Value};

use utils::testing::{make_get_request, make_jwt_request, make_post_request};

use crate::api::main::{login_user, setup};

#[tokio::test]
async fn test_post_users() {
    let app = setup().await;

    let body = r#"{"username": "test", "password": "password123"}"#.to_owned();
    let response = make_post_request(app.router, "/auth/register", body, None).await;

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let v: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(v["id"], 1);
    assert_eq!(v["username"], "test");
    assert!(v["display_name"].is_null());
}

#[tokio::test]
pub(super) async fn test_post_users_error() {
    let app = setup().await;

    let body = r#"{"username": "1", "password": "password123"}"#.to_owned();
    let response = make_post_request(app.router, "/auth/register", body, None).await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let result: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result["message"], "Validation error");
    assert_eq!(result["details"]["username"][0]["code"], "length");
}

#[tokio::test]
pub async fn test_get_users() {
    let app = setup().await;

    let body = r#"{"username": "test", "password": "password123"}"#.to_owned();
    make_post_request(app.router.clone(), "/auth/register", body, None).await;

    let response = make_get_request(app.router, "/users", None).await;
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let v: Value = serde_json::from_slice(&body).unwrap();

    assert!(v["users"].is_array());
    let users_array = v["users"].as_array().unwrap();
    assert_eq!(users_array.len(), 1);
    assert_eq!(users_array[0]["id"], 1);
    assert_eq!(users_array[0]["username"], "test");
}

#[tokio::test]
async fn test_me_endpoints() {
    let app = setup().await;

    // 1. Register a new user
    let register_body = json!({
        "username": "me_user",
        "password": "password123"
    })
    .to_string();
    let response =
        make_post_request(app.router.clone(), "/auth/register", register_body, None).await;
    assert_eq!(response.status(), StatusCode::CREATED);

    // 2. Log in to get a token
    let token = login_user(app.router.clone(), "me_user", "password123").await;

    // 3. GET /users/me to fetch initial profile
    let response = make_jwt_request(app.router.clone(), "/users/me", "GET", None, &token).await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let v: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["username"], "me_user");
    assert!(v["bio"].is_null());
    assert!(v["display_name"].is_null());

    // 4. PUT /users/me to update the profile
    let update_body = json!({
        "display_name": "Me User",
        "bio": "This is my updated bio.",
        "avatar_url": "https://example.com/avatar.png"
    })
    .to_string();
    let response = make_jwt_request(
        app.router.clone(),
        "/users/me",
        "PUT",
        Some(update_body),
        &token,
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let v_updated: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v_updated["display_name"], "Me User");
    assert_eq!(v_updated["bio"], "This is my updated bio.");

    // 5. GET /users/me again to verify the update was persisted
    let response = make_jwt_request(app.router.clone(), "/users/me", "GET", None, &token).await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let v_verify: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v_verify["display_name"], "Me User");
    assert_eq!(v_verify["bio"], "This is my updated bio.");
}
