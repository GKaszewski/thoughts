use super::*;
use crate::testing::make_state;
use axum::{
    body::Body,
    http::Request,
    routing::{delete, post},
    Router,
};
use tower::ServiceExt;

fn app() -> Router {
    Router::new()
        .route(
            "/users/{username}/follow",
            post(post_follow).delete(delete_follow),
        )
        .with_state(make_state())
}

#[tokio::test]
async fn follow_without_auth_returns_401() {
    let resp = app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/users/alice/follow")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn unfollow_remote_without_auth_returns_401() {
    let resp = app()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/users/alice@example.com/follow")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
}
