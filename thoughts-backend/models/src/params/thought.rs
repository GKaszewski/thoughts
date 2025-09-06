use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::domains::thought::Visibility;

#[derive(Deserialize, Validate, ToSchema)]
pub struct CreateThoughtParams {
    #[validate(length(
        min = 1,
        max = 128,
        message = "Content must be between 1 and 128 characters"
    ))]
    pub content: String,
    pub visibility: Option<Visibility>,
    pub reply_to_id: Option<Uuid>,
}
