use crate::api::main::{create_user_with_password, login_user, setup};
use axum::http::{header, HeaderName, StatusCode};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use utils::testing::{make_jwt_request, make_request_with_headers};

#[tokio::test]
async fn test_api_key_flow() {
    let app = setup().await;
    let _ = create_user_with_password(
        &app.db,
        "apikey_user",
        "password123",
        "apikey_user@example.com",
    )
    .await;
    let jwt = login_user(app.router.clone(), "apikey_user", "password123").await;

    // 1. Create a new API key using JWT auth
    let create_body = json!({ "name": "My Test Key" }).to_string();
    let response = make_jwt_request(
        app.router.clone(),
        "/users/me/api-keys",
        "POST",
        Some(create_body),
        &jwt,
    )
    .await;
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let v: Value = serde_json::from_slice(&body).unwrap();

    let plaintext_key = v["plaintext_key"]
        .as_str()
        .expect("Plaintext key not found")
        .to_string();
    let key_id = v["id"].as_str().expect("Key ID not found").to_string();
    assert!(plaintext_key.starts_with("th_"));

    // 2. Use the new API key to post a thought

    let thought_body = json!({ "content": "Posting with an API key!" }).to_string();
    let key = plaintext_key.clone();
    let api_key_header = format!("ApiKey {}", key);
    let content_type = "application/json";
    let headers: Vec<(HeaderName, &str)> = vec![
        (header::AUTHORIZATION, &api_key_header),
        (header::CONTENT_TYPE, content_type),
    ];

    let response = make_request_with_headers(
        app.router.clone(),
        "/thoughts",
        "POST",
        Some(thought_body),
        headers,
    )
    .await;

    assert_eq!(response.status(), StatusCode::CREATED);

    // 3. Delete the API key using JWT auth
    let response = make_jwt_request(
        app.router.clone(),
        &format!("/users/me/api-keys/{}", key_id),
        "DELETE",
        None,
        &jwt,
    )
    .await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // 4. Try to use the deleted key again, expecting failure
    let body = json!({ "content": "This should fail" }).to_string();
    let headers: Vec<(HeaderName, &str)> = vec![
        (header::AUTHORIZATION, &api_key_header),
        (header::CONTENT_TYPE, content_type),
    ];

    let response =
        make_request_with_headers(app.router.clone(), "/thoughts", "POST", Some(body), headers)
            .await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
