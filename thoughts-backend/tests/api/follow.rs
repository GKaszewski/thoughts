use super::main::{create_test_user, setup};
use axum::http::StatusCode;
use utils::testing::{make_delete_request, make_post_request};

#[tokio::test]
async fn test_follow_endpoints() {
    let app = setup().await;
    create_test_user(&app.db, "user1").await; // AuthUser is ID 1
    create_test_user(&app.db, "user2").await;

    // 1. user1 follows user2
    let response =
        make_post_request(app.router.clone(), "/users/user2/follow", "".to_string()).await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // 2. user1 tries to follow user2 again (should fail)
    let response =
        make_post_request(app.router.clone(), "/users/user2/follow", "".to_string()).await;
    assert_eq!(response.status(), StatusCode::CONFLICT);

    // 3. user1 tries to follow a non-existent user
    let response =
        make_post_request(app.router.clone(), "/users/nobody/follow", "".to_string()).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // 4. user1 unfollows user2
    let response = make_delete_request(app.router.clone(), "/users/user2/follow").await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // 5. user1 tries to unfollow user2 again (should fail)
    let response = make_delete_request(app.router.clone(), "/users/user2/follow").await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
