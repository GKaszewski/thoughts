use crate::api::main::{create_user_with_password, setup};
use axum::http::{header, StatusCode};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use utils::testing::{
    make_get_request, make_jwt_request, make_post_request, make_request_with_headers,
};

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

#[tokio::test]
async fn test_user_inbox_follow() {
    let app = setup().await;
    // user1 will be followed
    create_user_with_password(&app.db, "user1", "password123").await;
    // user2 will be the follower
    create_user_with_password(&app.db, "user2", "password123").await;

    // Construct a follow activity from user2, targeting user1
    let follow_activity = json!({
      "@context": "https://www.w3.org/ns/activitystreams",
      "id": "http://localhost:3000/some-unique-id",
      "type": "Follow",
      "actor": "http://localhost:3000/users/user2", // The actor is user2
      "object": "http://localhost:3000/users/user1"
    })
    .to_string();

    // POST the activity to user1's inbox
    let response = make_post_request(
        app.router.clone(),
        "/users/user1/inbox",
        follow_activity,
        None,
    )
    .await;

    assert_eq!(response.status(), StatusCode::ACCEPTED);

    // Verify that user2 is now following user1 in the database
    let followers = app::persistence::follow::get_followed_ids(&app.db, 2)
        .await
        .unwrap();
    assert!(followers.contains(&1), "User2 should be following user1");

    let following = app::persistence::follow::get_followed_ids(&app.db, 1)
        .await
        .unwrap();
    assert!(
        !following.contains(&2),
        "User1 should now be followed by user2"
    );
    assert!(following.is_empty(), "User1 should not be following anyone");
}

#[tokio::test]
async fn test_user_outbox_get() {
    let app = setup().await;
    create_user_with_password(&app.db, "testuser", "password123").await;
    let token = super::main::login_user(app.router.clone(), "testuser", "password123").await;

    // Create a thought first
    let thought_body = json!({ "content": "This is a federated thought!" }).to_string();
    make_jwt_request(
        app.router.clone(),
        "/thoughts",
        "POST",
        Some(thought_body),
        &token,
    )
    .await;

    // Now, fetch the outbox
    let response = make_request_with_headers(
        app.router.clone(),
        "/users/testuser/outbox",
        "GET",
        None,
        vec![(header::ACCEPT, "application/activity+json")],
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let v: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(v["type"], "OrderedCollection");
    assert_eq!(v["totalItems"], 1);
    assert_eq!(v["orderedItems"][0]["type"], "Create");
    assert_eq!(
        v["orderedItems"][0]["object"]["content"],
        "This is a federated thought!"
    );
}
