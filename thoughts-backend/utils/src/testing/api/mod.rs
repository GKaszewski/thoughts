use axum::{body::Body, http::Request, response::Response, Router};
use tower::ServiceExt;

pub async fn make_get_request(app: Router, url: &str, user_id: Option<i32>) -> Response {
    let mut builder = Request::builder()
        .uri(url)
        .header("Content-Type", "application/json");

    if let Some(user_id) = user_id {
        builder = builder.header("x-test-user-id", user_id.to_string());
    }

    app.oneshot(builder.body(Body::empty()).unwrap())
        .await
        .unwrap()
}

pub async fn make_post_request(
    app: Router,
    url: &str,
    body: String,
    user_id: Option<i32>,
) -> Response {
    let mut builder = Request::builder()
        .method("POST")
        .uri(url)
        .header("Content-Type", "application/json");

    if let Some(user_id) = user_id {
        builder = builder.header("x-test-user-id", user_id.to_string());
    }

    app.oneshot(builder.body(Body::from(body)).unwrap())
        .await
        .unwrap()
}

pub async fn make_delete_request(app: Router, url: &str, user_id: Option<i32>) -> Response {
    let mut builder = Request::builder()
        .method("DELETE")
        .uri(url)
        .header("Content-Type", "application/json");

    if let Some(user_id) = user_id {
        builder = builder.header("x-test-user-id", user_id.to_string());
    }

    app.oneshot(builder.body(Body::empty()).unwrap())
        .await
        .unwrap()
}
