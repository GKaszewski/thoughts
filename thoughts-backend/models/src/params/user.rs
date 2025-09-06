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

#[derive(Deserialize, Validate, ToSchema, Default)]
pub struct UpdateUserParams {
    #[validate(length(max = 50))]
    #[schema(example = "Frutiger Aero Fan")]
    pub display_name: Option<String>,

    #[validate(length(max = 160))]
    #[schema(example = "Est. 2004")]
    pub bio: Option<String>,

    #[validate(url)]
    pub avatar_url: Option<String>,

    #[validate(url)]
    pub header_url: Option<String>,

    pub custom_css: Option<String>,

    #[validate(length(max = 8))]
    #[schema(example = json!(["username1", "username2"]))]
    pub top_friends: Option<Vec<String>>,
}
