use axum::http::StatusCode;
use http_body_util::BodyExt;
use models::domains::top_friends;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde_json::{json, Value};

use utils::testing::{make_get_request, make_jwt_request, make_post_request};

use crate::api::main::{create_user_with_password, login_user, setup};

#[tokio::test]
async fn test_post_users() {
    let app = setup().await;

    let body = r#"{"username": "test", "email": "test@example.com", "password": "password123"}"#
        .to_owned();
    let response = make_post_request(app.router, "/auth/register", body, None).await;

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let v: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(v["username"], "test");
    assert!(v["displayName"].is_string());
}

#[tokio::test]
pub(super) async fn test_post_users_error() {
    let app = setup().await;

    let body =
        r#"{"username": "1", "email": "test@example.com", "password": "password123"}"#.to_owned();
    let response = make_post_request(app.router, "/auth/register", body, None).await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let result: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result["message"], "Validation error");
    assert_eq!(result["details"]["username"][0]["code"], "length");
}

#[tokio::test]
pub async fn test_get_users() {
    let app = setup().await;

    let body = r#"{"username": "test", "email": "test@example.com", "password": "password123"}"#
        .to_owned();
    make_post_request(app.router.clone(), "/auth/register", body, None).await;

    let response = make_get_request(app.router, "/users", None).await;
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let v: Value = serde_json::from_slice(&body).unwrap();

    assert!(v["users"].is_array());
    let users_array = v["users"].as_array().unwrap();
    assert_eq!(users_array.len(), 1);
    assert_eq!(users_array[0]["username"], "test");
}

#[tokio::test]
async fn test_me_endpoints() {
    let app = setup().await;

    // 1. Register a new user
    let register_body = json!({
        "username": "me_user",
        "email": "me_user@example.com",
        "password": "password123"
    })
    .to_string();
    let response =
        make_post_request(app.router.clone(), "/auth/register", register_body, None).await;
    assert_eq!(response.status(), StatusCode::CREATED);

    // 2. Log in to get a token
    let token = login_user(app.router.clone(), "me_user", "password123").await;

    // 3. GET /users/me to fetch initial profile
    let response = make_jwt_request(app.router.clone(), "/users/me", "GET", None, &token).await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let v: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["username"], "me_user");
    assert!(v["bio"].is_null());
    assert!(v["displayName"].is_string());

    // 4. PUT /users/me to update the profile
    let update_body = json!({
        "displayName": "Me User",
        "bio": "This is my updated bio.",
        "avatarUrl": "https://example.com/avatar.png"
    })
    .to_string();
    let response = make_jwt_request(
        app.router.clone(),
        "/users/me",
        "PUT",
        Some(update_body),
        &token,
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let v_updated: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v_updated["displayName"], "Me User");
    assert_eq!(v_updated["bio"], "This is my updated bio.");

    // 5. GET /users/me again to verify the update was persisted
    let response = make_jwt_request(app.router.clone(), "/users/me", "GET", None, &token).await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let v_verify: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v_verify["displayName"], "Me User");
    assert_eq!(v_verify["bio"], "This is my updated bio.");
}

#[tokio::test]
async fn test_update_me_top_friends() {
    let app = setup().await;

    // 1. Create users for the test
    let user_me =
        create_user_with_password(&app.db, "me_user", "password123", "me_user@example.com").await;
    let friend1 =
        create_user_with_password(&app.db, "friend1", "password123", "friend1@example.com").await;
    let friend2 =
        create_user_with_password(&app.db, "friend2", "password123", "friend2@example.com").await;
    let _friend3 =
        create_user_with_password(&app.db, "friend3", "password123", "friend3@example.com").await;

    // 2. Log in as "me_user"
    let token = login_user(app.router.clone(), "me_user", "password123").await;

    // 3. Update profile to set top friends
    let update_body = json!({
        "topFriends": ["friend1", "friend2"]
    })
    .to_string();

    let response = make_jwt_request(
        app.router.clone(),
        "/users/me",
        "PUT",
        Some(update_body),
        &token,
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);

    // 4. Verify the database state directly
    let top_friends_list = top_friends::Entity::find()
        .filter(top_friends::Column::UserId.eq(user_me.id))
        .all(&app.db)
        .await
        .unwrap();

    assert_eq!(top_friends_list.len(), 2);
    assert_eq!(top_friends_list[0].friend_id, friend1.id);
    assert_eq!(top_friends_list[0].position, 1);
    assert_eq!(top_friends_list[1].friend_id, friend2.id);
    assert_eq!(top_friends_list[1].position, 2);

    // 5. Update again with a different list to test replacement
    let update_body_2 = json!({
        "topFriends": ["friend2"]
    })
    .to_string();

    let response = make_jwt_request(
        app.router.clone(),
        "/users/me",
        "PUT",
        Some(update_body_2),
        &token,
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);

    // 6. Verify the new state
    let top_friends_list_2 = top_friends::Entity::find()
        .filter(top_friends::Column::UserId.eq(user_me.id))
        .all(&app.db)
        .await
        .unwrap();

    assert_eq!(top_friends_list_2.len(), 1);
    assert_eq!(top_friends_list_2[0].friend_id, friend2.id);
    assert_eq!(top_friends_list_2[0].position, 1);
}

#[tokio::test]
async fn test_update_me_css_and_images() {
    let app = setup().await;

    // 1. Create and log in as a user
    let _ =
        create_user_with_password(&app.db, "css_user", "password123", "css_user@example.com").await;
    let token = login_user(app.router.clone(), "css_user", "password123").await;

    // 2. Attempt to update with an invalid avatar URL
    let invalid_body = json!({
        "avatarUrl": "not-a-valid-url"
    })
    .to_string();

    let response = make_jwt_request(
        app.router.clone(),
        "/users/me",
        "PUT",
        Some(invalid_body),
        &token,
    )
    .await;
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    // 3. Update profile with valid URLs and custom CSS
    let valid_body = json!({
        "avatarUrl": "https://example.com/new-avatar.png",
        "headerUrl": "https://example.com/new-header.jpg",
        "customCss": "body { color: blue; }"
    })
    .to_string();

    let response = make_jwt_request(
        app.router.clone(),
        "/users/me",
        "PUT",
        Some(valid_body),
        &token,
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);

    // 4. Verify the changes were persisted by fetching the profile again
    let response = make_jwt_request(app.router.clone(), "/users/me", "GET", None, &token).await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let v: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(v["avatarUrl"], "https://example.com/new-avatar.png");
    assert_eq!(v["headerUrl"], "https://example.com/new-header.jpg");
    assert_eq!(v["customCss"], "body { color: blue; }");
}

#[tokio::test]
async fn test_get_all_users_paginated() {
    let app = setup().await;

    for i in 0..25 {
        create_user_with_password(
            &app.db,
            &format!("user{}", i),
            "password123",
            &format!("u{}@e.com", i),
        )
        .await;
    }

    let response_p1 = make_get_request(app.router.clone(), "/users/all", None).await;
    assert_eq!(response_p1.status(), StatusCode::OK);
    let body_p1 = response_p1.into_body().collect().await.unwrap().to_bytes();
    let v_p1: Value = serde_json::from_slice(&body_p1).unwrap();

    assert_eq!(
        v_p1["items"].as_array().unwrap().len(),
        20,
        "First page should have 20 items"
    );
    assert_eq!(v_p1["page"], 1);
    assert_eq!(v_p1["pageSize"], 20);
    assert_eq!(v_p1["totalPages"], 2);
    assert_eq!(v_p1["totalItems"], 25);

    let response_p2 = make_get_request(app.router.clone(), "/users/all?page=2", None).await;
    assert_eq!(response_p2.status(), StatusCode::OK);
    let body_p2 = response_p2.into_body().collect().await.unwrap().to_bytes();
    let v_p2: Value = serde_json::from_slice(&body_p2).unwrap();

    assert_eq!(
        v_p2["items"].as_array().unwrap().len(),
        5,
        "Second page should have 5 items"
    );
    assert_eq!(v_p2["page"], 2);
    assert_eq!(v_p2["totalPages"], 2);
}

#[tokio::test]
async fn test_get_all_users_count() {
    let app = setup().await;

    for i in 0..25 {
        create_user_with_password(
            &app.db,
            &format!("user{}", i),
            "password123",
            &format!("u{}@e.com", i),
        )
        .await;
    }

    let response = make_get_request(app.router.clone(), "/users/count", None).await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let v: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(v["count"], 25);
}
