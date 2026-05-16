use super::*;
use crate::testing::make_state;
use axum::{body::Body, http::Request, routing::get, Router};
use tower::ServiceExt;

fn app() -> Router {
    Router::new()
        .route(
            "/federation/actors/{handle}/posts",
            get(remote_actor_posts_handler),
        )
        .with_state(make_state())
}

#[tokio::test]
async fn unknown_actor_returns_404() {
    let resp = app()
        .oneshot(
            Request::builder()
                .uri("/federation/actors/%40alice%40example.com/posts")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
}
