use crate::api::main::{create_user_with_password, login_user, setup};
use axum::http::StatusCode;
use http_body_util::BodyExt;
use serde_json::{json, Value};
use utils::testing::{make_get_request, make_jwt_request};

#[tokio::test]
async fn test_search_all() {
    let app = setup().await;

    // 1. Setup users and data
    let user1 =
        create_user_with_password(&app.db, "search_user1", "password123", "s1@test.com").await;
    let user2 =
        create_user_with_password(&app.db, "search_user2", "password123", "s2@test.com").await;
    let _user3 =
        create_user_with_password(&app.db, "stranger_user", "password123", "s3@test.com").await;

    // Make user1 and user2 friends
    app::persistence::follow::follow_user(&app.db, user1.id, user2.id)
        .await
        .unwrap();
    app::persistence::follow::follow_user(&app.db, user2.id, user1.id)
        .await
        .unwrap();

    let token1 = login_user(app.router.clone(), "search_user1", "password123").await;
    let token2 = login_user(app.router.clone(), "search_user2", "password123").await;
    let token3 = login_user(app.router.clone(), "stranger_user", "password123").await;

    // User1 posts thoughts with different visibilities
    let thought_public =
        json!({ "content": "A very public thought about Rust.", "visibility": "Public" })
            .to_string();
    let thought_friends =
        json!({ "content": "A friendly thought, just for pals.", "visibility": "FriendsOnly" })
            .to_string();
    let thought_private =
        json!({ "content": "A private thought, for my eyes only.", "visibility": "Private" })
            .to_string();

    make_jwt_request(
        app.router.clone(),
        "/thoughts",
        "POST",
        Some(thought_public),
        &token1,
    )
    .await;
    make_jwt_request(
        app.router.clone(),
        "/thoughts",
        "POST",
        Some(thought_friends),
        &token1,
    )
    .await;
    make_jwt_request(
        app.router.clone(),
        "/thoughts",
        "POST",
        Some(thought_private),
        &token1,
    )
    .await;

    // 2. Run search tests

    // -- User Search --
    let response = make_get_request(app.router.clone(), "/search?q=search_user1", None).await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let v: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["users"]["users"].as_array().unwrap().len(), 1);
    assert_eq!(v["users"]["users"][0]["username"], "search_user1");

    // -- Thought Search (Public) --
    let response = make_get_request(app.router.clone(), "/search?q=public", None).await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let v: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        v["thoughts"]["thoughts"].as_array().unwrap().len(),
        1,
        "Guest should find public thought"
    );
    assert!(v["thoughts"]["thoughts"][0]["content"]
        .as_str()
        .unwrap()
        .contains("public"));

    // -- Thought Search (FriendsOnly) --
    let response = make_jwt_request(
        app.router.clone(),
        "/search?q=friendly",
        "GET",
        None,
        &token1,
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let v: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        v["thoughts"]["thoughts"].as_array().unwrap().len(),
        1,
        "Author should find friends thought"
    );

    let response = make_jwt_request(
        app.router.clone(),
        "/search?q=friendly",
        "GET",
        None,
        &token2,
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let v: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        v["thoughts"]["thoughts"].as_array().unwrap().len(),
        1,
        "Friend should find friends thought"
    );

    let response = make_jwt_request(
        app.router.clone(),
        "/search?q=friendly",
        "GET",
        None,
        &token3,
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let v: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        v["thoughts"]["thoughts"].as_array().unwrap().len(),
        0,
        "Stranger should NOT find friends thought"
    );

    let response = make_get_request(app.router.clone(), "/search?q=friendly", None).await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let v: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        v["thoughts"]["thoughts"].as_array().unwrap().len(),
        0,
        "Guest should NOT find friends thought"
    );

    // -- Thought Search (Private) --
    let response = make_jwt_request(
        app.router.clone(),
        "/search?q=private",
        "GET",
        None,
        &token1,
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let v: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        v["thoughts"]["thoughts"].as_array().unwrap().len(),
        1,
        "Author should find private thought"
    );

    let response = make_jwt_request(
        app.router.clone(),
        "/search?q=private",
        "GET",
        None,
        &token2,
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let v: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        v["thoughts"]["thoughts"].as_array().unwrap().len(),
        0,
        "Friend should NOT find private thought"
    );

    let response = make_get_request(app.router.clone(), "/search?q=private", None).await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let v: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        v["thoughts"]["thoughts"].as_array().unwrap().len(),
        0,
        "Guest should NOT find private thought"
    );
}
