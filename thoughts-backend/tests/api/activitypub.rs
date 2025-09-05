use crate::api::main::{create_user_with_password, setup};
use axum::http::{header, StatusCode};
use http_body_util::BodyExt;
use serde_json::Value;
use utils::testing::{make_get_request, make_request_with_headers};

#[tokio::test]
async fn test_webfinger_discovery() {
    let app = setup().await;
    create_user_with_password(&app.db, "testuser", "password123").await;

    // 1. Valid WebFinger lookup for existing user
    let url = "/.well-known/webfinger?resource=acct:testuser@localhost:3000";
    let response = make_get_request(app.router.clone(), url, None).await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let v: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["subject"], "acct:testuser@localhost:3000");
    assert_eq!(
        v["links"][0]["href"],
        "http://localhost:3000/users/testuser"
    );

    // 2. WebFinger lookup for a non-existent user
    let response = make_get_request(
        app.router.clone(),
        "/.well-known/webfinger?resource=acct:nobody@localhost:3000",
        None,
    )
    .await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_user_actor_endpoint() {
    let app = setup().await;
    create_user_with_password(&app.db, "testuser", "password123").await;

    let response = make_request_with_headers(
        app.router.clone(),
        "/users/testuser",
        "GET",
        None,
        vec![(
            header::ACCEPT,
            "application/activity+json, application/ld+json; profile=\"https://www.w3.org/ns/activitystreams\"",
        )],
    ).await;

    assert_eq!(response.status(), StatusCode::OK);
    let content_type = response.headers().get(header::CONTENT_TYPE).unwrap();
    assert_eq!(content_type, "application/activity+json");

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let v: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["type"], "Person");
    assert_eq!(v["preferredUsername"], "testuser");
    assert_eq!(v["id"], "http://localhost:3000/users/testuser");
}
