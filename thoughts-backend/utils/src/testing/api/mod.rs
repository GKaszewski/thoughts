use axum::{
    body::Body,
    http::{header, Request},
    response::Response,
    Router,
};
use tower::ServiceExt;
use uuid::Uuid;

pub async fn make_get_request(app: Router, url: &str, user_id: Option<Uuid>) -> Response {
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
    user_id: Option<Uuid>,
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

pub async fn make_delete_request(app: Router, url: &str, user_id: Option<Uuid>) -> Response {
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

pub async fn make_jwt_request(
    app: Router,
    url: &str,
    method: &str,
    body: Option<String>,
    token: &str,
) -> Response {
    let builder = Request::builder()
        .method(method)
        .uri(url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token));

    let request_body = body.unwrap_or_default();
    app.oneshot(builder.body(Body::from(request_body)).unwrap())
        .await
        .unwrap()
}

pub async fn make_request_with_headers(
    app: Router,
    url: &str,
    method: &str,
    body: Option<String>,
    headers: Vec<(header::HeaderName, &str)>,
) -> Response {
    let mut builder = Request::builder()
        .method(method)
        .uri(url)
        .header("Content-Type", "application/json");

    for (key, value) in headers {
        builder = builder.header(key, value);
    }

    let request_body = body.unwrap_or_default();
    app.oneshot(builder.body(Body::from(request_body)).unwrap())
        .await
        .unwrap()
}
