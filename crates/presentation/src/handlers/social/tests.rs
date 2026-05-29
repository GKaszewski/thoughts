use super::get_friends_handler;
use super::*;
use crate::testing::make_state;
use axum::{
    body::Body,
    http::Request,
    routing::{get, post},
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
async fn get_friends_without_auth_returns_401() {
    let app = Router::new()
        .route("/users/me/friends", get(get_friends_handler))
        .with_state(make_state());
    let resp = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/users/me/friends")
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
