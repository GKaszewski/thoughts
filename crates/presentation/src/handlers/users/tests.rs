use super::*;
use crate::testing::make_state;
use axum::{
    body::Body,
    http::{header, Request},
    routing::get,
    Router,
};
use tower::ServiceExt;

fn app() -> Router {
    Router::new()
        .route("/users/{username}", get(get_user))
        .route("/users/lookup", get(lookup_handler))
        .with_state(make_state())
}

#[tokio::test]
async fn get_unknown_user_returns_404() {
    let resp = app()
        .oneshot(
            Request::builder()
                .uri("/users/nobody")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn get_user_with_ap_accept_returns_404_when_actor_not_found() {
    let resp = app()
        .oneshot(
            Request::builder()
                .uri("/users/nobody")
                .header(header::ACCEPT, "application/activity+json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn lookup_unknown_handle_returns_404() {
    let resp = app()
        .oneshot(
            Request::builder()
                .uri("/users/lookup?handle=%40alice%40example.com")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
}
