use crate::api::main::{create_user_with_password, login_user, setup, TestApp};
use axum::http::StatusCode;
use http_body_util::BodyExt;
use serde_json::{json, Value};
use utils::testing::{make_get_request, make_jwt_request};

#[tokio::test]
async fn test_hashtag_flow() {
    let app = setup().await;
    let user =
        create_user_with_password(&app.db, "taguser", "password123", "taguser@example.com").await;
    let token = login_user(app.router.clone(), "taguser", "password123").await;

    // 1. Post a thought with hashtags
    let body = json!({ "content": "Hello #world this is a post about #RustLang" }).to_string();
    let response =
        make_jwt_request(app.router.clone(), "/thoughts", "POST", Some(body), &token).await;
    assert_eq!(response.status(), StatusCode::CREATED);
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let thought_json: Value = serde_json::from_slice(&body_bytes).unwrap();
    let thought_id = thought_json["id"].as_str().unwrap();

    // 2. Post another thought
    let body2 = json!({ "content": "Another post about the #rustlang ecosystem" }).to_string();
    make_jwt_request(app.router.clone(), "/thoughts", "POST", Some(body2), &token).await;

    // 3. Fetch thoughts by tag "rustlang"
    let response = make_get_request(app.router.clone(), "/tags/rustlang", Some(user.id)).await;
    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let v: Value = serde_json::from_slice(&body_bytes).unwrap();

    let thoughts = v["thoughts"].as_array().unwrap();
    assert_eq!(thoughts.len(), 2);
    // Note: The most recent post appears first
    assert_eq!(
        thoughts[0]["content"],
        "Another post about the #rustlang ecosystem"
    );
    assert_eq!(thoughts[1]["id"], thought_id);

    // 4. Fetch thoughts by tag "world"
    let response = make_get_request(app.router.clone(), "/tags/world", Some(user.id)).await;
    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let v: Value = serde_json::from_slice(&body_bytes).unwrap();

    let thoughts = v["thoughts"].as_array().unwrap();
    assert_eq!(thoughts.len(), 1);
    assert_eq!(thoughts[0]["id"], thought_id);
}

#[tokio::test]
async fn test_popular_tags() {
    let app = setup().await;
    let _ = create_user_with_password(&app.db, "poptag_user", "password123", "poptag@example.com")
        .await;
    let token = login_user(app.router.clone(), "poptag_user", "password123").await;

    // Helper async function to post a thought
    async fn post_thought(app: &TestApp, token: &str, content: &str) {
        let body = json!({ "content": content }).to_string();
        let response =
            make_jwt_request(app.router.clone(), "/thoughts", "POST", Some(body), token).await;
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    // 1. Post thoughts to create tag usage data
    // Expected counts: rust (3), web (2), axum (2), testing (1)
    post_thought(&app, &token, "My first post about #rust and the #web").await;
    post_thought(&app, &token, "Another post about #rust and #axum").await;
    post_thought(&app, &token, "I'm really enjoying #rust lately").await;
    post_thought(&app, &token, "Let's talk about #axum and the #web").await;
    post_thought(&app, &token, "Don't forget about #testing").await;

    // 2. Fetch the popular tags
    let response = make_get_request(app.router.clone(), "/tags/popular", None).await;
    println!("Response: {:?}", response);
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let v: Vec<String> = serde_json::from_slice(&body).unwrap();

    // 3. Assert the results
    assert_eq!(v.len(), 4, "Should return the 4 unique tags used");
    assert_eq!(
        v,
        vec!["rust", "axum", "web", "testing"],
        "Tags should be ordered by popularity, then alphabetically"
    );
}
