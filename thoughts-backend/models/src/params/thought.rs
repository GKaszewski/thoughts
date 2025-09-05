use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

#[derive(Deserialize, Validate, ToSchema)]
pub struct CreateThoughtParams {
    #[validate(length(
        min = 1,
        max = 128,
        message = "Content must be between 1 and 128 characters"
    ))]
    pub content: String,
}
