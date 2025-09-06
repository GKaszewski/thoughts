use utils::testing::setup_test_db;

mod user;

use user::test_user;

#[tokio::test]
async fn user_main() {
    std::env::set_var(
        "MANAGEMENT_DATABASE_URL",
        "postgres://postgres:postgres@localhost:5434/postgres",
    );
    std::env::set_var(
        "DATABASE_URL",
        "postgres://postgres:postgres@localhost:5434/postgres",
    );
    let db = setup_test_db().await.expect("Failed to set up test db");

    test_user(&db).await;
}
