use super::main::{create_user_with_password, setup};
use axum::http::StatusCode;
use utils::testing::make_jwt_request;

#[tokio::test]
async fn test_follow_endpoints() {
    std::env::set_var("AUTH_SECRET", "test-secret");
    let app = setup().await;

    create_user_with_password(&app.db, "user1", "password1").await;
    create_user_with_password(&app.db, "user2", "password2").await;

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
