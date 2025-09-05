use api::setup_router;
use app::persistence::user::create_user;
use axum::Router;
use models::params::user::CreateUserParams;
use sea_orm::DatabaseConnection;
use utils::testing::setup_test_db;

pub struct TestApp {
    pub router: Router,
    pub db: DatabaseConnection,
}

pub async fn setup() -> TestApp {
    let db = setup_test_db("sqlite::memory:")
        .await
        .expect("Failed to set up test db");
    let router = setup_router(db.clone());
    TestApp { router, db }
}

// Helper to create users for tests
pub async fn create_test_user(db: &DatabaseConnection, username: &str) {
    let params = CreateUserParams {
        username: username.to_string(),
    };
    create_user(db, params)
        .await
        .expect("Failed to create test user");
}
