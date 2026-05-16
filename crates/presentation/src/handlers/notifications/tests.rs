use super::*;
use crate::testing::make_state;
use axum::{
    body::Body,
    http::{header, Request},
    routing::{get, patch},
    Router,
};
use tower::ServiceExt;

fn app() -> Router {
    Router::new()
        .route("/notifications", patch(mark_all_read))
        .route("/notifications/{id}", patch(mark_notification_read))
        .with_state(make_state())
}

#[tokio::test]
async fn patch_notification_without_auth_returns_401() {
    let resp = app()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri("/notifications/00000000-0000-0000-0000-000000000001")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"read":true}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn patch_all_without_auth_returns_401() {
    let resp = app()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri("/notifications")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"read":true}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
}
