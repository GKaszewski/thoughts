use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

#[derive(Deserialize, Validate, ToSchema)]
pub struct CreateUserParams {
    #[validate(length(min = 2))]
    pub username: String,
    #[validate(length(min = 6))]
    pub password: String,
}
