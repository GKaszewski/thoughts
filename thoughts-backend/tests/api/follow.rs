use crate::api::main::login_user;

use super::main::{create_user_with_password, setup};
use axum::http::StatusCode;
use http_body_util::BodyExt;
use serde_json::Value;
use utils::testing::{make_get_request, make_jwt_request};

#[tokio::test]
async fn test_follow_endpoints() {
    std::env::set_var("AUTH_SECRET", "test-secret");
    let app = setup().await;

    create_user_with_password(&app.db, "user1", "password1", "user1@example.com").await;
    create_user_with_password(&app.db, "user2", "password2", "user2@example.com").await;

    let token = super::main::login_user(app.router.clone(), "user1", "password1").await;

    // 1. user1 follows user2
    let response = make_jwt_request(
        app.router.clone(),
        "/users/user2/follow",
        "POST",
        None,
        &token,
    )
    .await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // 2. user1 tries to follow user2 again (should fail)
    let response = make_jwt_request(
        app.router.clone(),
        "/users/user2/follow",
        "POST",
        None,
        &token,
    )
    .await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // 3. user1 tries to follow a non-existent user
    let response = make_jwt_request(
        app.router.clone(),
        "/users/nobody/follow",
        "POST",
        None,
        &token,
    )
    .await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // 4. user1 unfollows user2
    let response = make_jwt_request(
        app.router.clone(),
        "/users/user2/follow",
        "DELETE",
        None,
        &token,
    )
    .await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // 5. user1 tries to unfollow user2 again (should fail)
    let response = make_jwt_request(
        app.router.clone(),
        "/users/user2/follow",
        "DELETE",
        None,
        &token,
    )
    .await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_follow_lists() {
    let app = setup().await;

    let user_a = create_user_with_password(&app.db, "userA", "password123", "a@a.com").await;
    let user_b = create_user_with_password(&app.db, "userB", "password123", "b@b.com").await;
    let user_c = create_user_with_password(&app.db, "userC", "password123", "c@c.com").await;

    // A follows B, C follows A
    app::persistence::follow::follow_user(&app.db, user_a.id, user_b.id)
        .await
        .unwrap();
    app::persistence::follow::follow_user(&app.db, user_c.id, user_a.id)
        .await
        .unwrap();

    // 1. Check user A's lists
    let response_following =
        make_get_request(app.router.clone(), "/users/userA/following", None).await;
    let body_following = response_following
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes();
    let v: Value = serde_json::from_slice(&body_following).unwrap();
    assert_eq!(v["users"].as_array().unwrap().len(), 1);
    assert_eq!(v["users"][0]["username"], "userB");

    let response_followers =
        make_get_request(app.router.clone(), "/users/userA/followers", None).await;
    let body_followers = response_followers
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes();
    let v: Value = serde_json::from_slice(&body_followers).unwrap();
    assert_eq!(v["users"].as_array().unwrap().len(), 1);
    assert_eq!(v["users"][0]["username"], "userC");

    // 2. Check user A's /me endpoint
    let jwt_a = login_user(app.router.clone(), "userA", "password123").await;
    let response_me = make_jwt_request(app.router.clone(), "/users/me", "GET", None, &jwt_a).await;
    let body_me = response_me.into_body().collect().await.unwrap().to_bytes();
    let v: Value = serde_json::from_slice(&body_me).unwrap();
    assert_eq!(v["username"], "userA");
    assert_eq!(v["following"].as_array().unwrap().len(), 1);
    assert_eq!(v["following"][0]["username"], "userB");
}

#[tokio::test]
async fn test_get_friends_list() {
    let app = setup().await;

    let user_a = create_user_with_password(&app.db, "userA", "password123", "a@a.com").await;
    let user_b = create_user_with_password(&app.db, "userB", "password123", "b@b.com").await;
    let user_c = create_user_with_password(&app.db, "userC", "password123", "c@c.com").await;

    // --- Create relationships ---
    // A and B are friends (reciprocal follow)
    app::persistence::follow::follow_user(&app.db, user_a.id, user_b.id)
        .await
        .unwrap();
    app::persistence::follow::follow_user(&app.db, user_b.id, user_a.id)
        .await
        .unwrap();

    // A follows C, but C does not follow A back
    app::persistence::follow::follow_user(&app.db, user_a.id, user_c.id)
        .await
        .unwrap();

    // --- Test as user_a ---
    let jwt_a = login_user(app.router.clone(), "userA", "password123").await;
    let response = make_jwt_request(app.router.clone(), "/friends", "GET", None, &jwt_a).await;
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let v: Value = serde_json::from_slice(&body).unwrap();
    let friends_list = v["users"].as_array().unwrap();

    assert_eq!(friends_list.len(), 1, "User A should only have one friend");
    assert_eq!(
        friends_list[0]["username"], "userB",
        "User B should be in User A's friend list"
    );
}
