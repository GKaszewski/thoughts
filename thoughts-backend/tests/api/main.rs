use api::setup_router;
use app::persistence::user::create_user;
use axum::Router;
use http_body_util::BodyExt;
use models::params::{auth::RegisterParams, user::CreateUserParams};
use sea_orm::DatabaseConnection;
use serde_json::{json, Value};
use utils::testing::{make_post_request, setup_test_db};

pub struct TestApp {
    pub router: Router,
    pub db: DatabaseConnection,
}

pub async fn setup() -> TestApp {
    std::env::set_var(
        "MANAGEMENT_DATABASE_URL",
        "postgres://postgres:postgres@localhost:5434/postgres",
    );
    std::env::set_var(
        "DATABASE_URL",
        "postgres://postgres:postgres@localhost:5434/postgres",
    );
    std::env::set_var("AUTH_SECRET", "test_secret");
    std::env::set_var("BASE_URL", "http://localhost:3000");
    std::env::set_var("HOST", "localhost");
    std::env::set_var("PORT", "3000");
    std::env::set_var("LOG_LEVEL", "debug");

    let db = setup_test_db().await.expect("Failed to set up test db");

    let db = db.clone();

    let router = setup_router(db.clone(), &app::config::Config::from_env());
    TestApp { router, db }
}

// Helper to create users for tests
pub async fn create_test_user(db: &DatabaseConnection, username: &str) {
    let params = CreateUserParams {
        username: username.to_string(),
        password: "password".to_string(),
    };
    create_user(db, params)
        .await
        .expect("Failed to create test user");
}

pub async fn create_user_with_password(db: &DatabaseConnection, username: &str, password: &str) {
    let params = RegisterParams {
        username: username.to_string(),
        password: password.to_string(),
    };
    app::persistence::auth::register_user(db, params)
        .await
        .expect("Failed to create test user with password");
}

pub async fn login_user(router: Router, username: &str, password: &str) -> String {
    let login_body = json!({ "username": username, "password": password }).to_string();
    let response = make_post_request(router, "/auth/login", login_body, None).await;
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let v: Value = serde_json::from_slice(&body).unwrap();
    v["token"].as_str().unwrap().to_string()
}
