use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

#[derive(Deserialize, Validate, ToSchema)]
pub struct CreateThoughtParams {
    pub author_id: i32,

    #[validate(length(min = 1, max = 128))]
    pub content: String,
}
