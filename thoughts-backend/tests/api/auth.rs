use crate::api::main::setup;
use axum::http::StatusCode;
use http_body_util::BodyExt;
use serde_json::{json, Value};
use utils::testing::{make_jwt_request, make_post_request};

#[tokio::test]
async fn test_auth_flow() {
    std::env::set_var("AUTH_SECRET", "test-secret");
    let app = setup().await;

    let register_body = json!({
        "username": "testuser",
        "password": "password123"
    })
    .to_string();
    let response =
        make_post_request(app.router.clone(), "/auth/register", register_body, None).await;
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let v: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["username"], "testuser");
    assert!(v["id"].is_number());

    let response = make_post_request(
        app.router.clone(),
        "/auth/register",
        json!({
            "username": "testuser",
            "password": "password456"
        })
        .to_string(),
        None,
    )
    .await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let login_body = json!({
        "username": "testuser",
        "password": "password123"
    })
    .to_string();
    let response = make_post_request(app.router.clone(), "/auth/login", login_body, None).await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let v: Value = serde_json::from_slice(&body).unwrap();
    let token = v["token"].as_str().expect("token not found").to_string();
    assert!(!token.is_empty());

    let bad_login_body = json!({
        "username": "testuser",
        "password": "wrongpassword"
    })
    .to_string();
    let response = make_post_request(app.router.clone(), "/auth/login", bad_login_body, None).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let response = make_jwt_request(app.router.clone(), "/feed", "GET", None, &token).await;
    assert_eq!(response.status(), StatusCode::OK);
}
