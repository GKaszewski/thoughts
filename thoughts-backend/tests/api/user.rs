use axum::http::StatusCode;
use http_body_util::BodyExt;
use serde_json::Value;

use utils::testing::{make_get_request, make_post_request};

use crate::api::main::setup;

#[tokio::test]
async fn test_post_users() {
    let app = setup().await;
    let response = make_post_request(
        app.router,
        "/users",
        r#"{"username": "test"}"#.to_owned(),
        None,
    )
    .await;
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&body[..], br#"{"id":1,"username":"test"}"#);
}

#[tokio::test]
pub(super) async fn test_post_users_error() {
    let app = setup().await;
    let response = make_post_request(
        app.router,
        "/users",
        r#"{"username": "1"}"#.to_owned(),
        None,
    )
    .await;
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let result: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result["message"], "Validation error");
    assert_eq!(result["details"]["username"][0]["code"], "length");
    assert_eq!(result["details"]["username"][0]["message"], Value::Null);
    assert_eq!(
        result["details"]["username"][0]["params"]["min"],
        Value::Number(2.into())
    )
}

#[tokio::test]
pub async fn test_get_users() {
    let app = setup().await;
    make_post_request(
        app.router.clone(),
        "/users",
        r#"{"username": "test"}"#.to_owned(),
        None,
    )
    .await;

    let response = make_get_request(app.router, "/users", None).await;
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&body[..], br#"{"users":[{"id":1,"username":"test"}]}"#);
}
