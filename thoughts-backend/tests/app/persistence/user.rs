use sea_orm::{DatabaseConnection, TryIntoModel};

use app::persistence::user::create_user;
use models::params::user::CreateUserParams;

pub(super) async fn test_user(db: &DatabaseConnection) {
    let params = CreateUserParams {
        username: "test".to_string(),
        password: "password".to_string(),
    };
    let user_model = create_user(db, params)
        .await
        .expect("Create user failed!")
        .try_into_model() // Convert ActiveModel to Model for easier checks
        .unwrap();

    assert_eq!(user_model.id, 1);
    assert_eq!(user_model.username, "test");
}
