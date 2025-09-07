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
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,

    #[validate(length(max = 4000))]
    #[schema(example = "Est. 2004")]
    pub bio: Option<String>,

    #[validate(url)]
    #[serde(rename = "avatarUrl")]
    pub avatar_url: Option<String>,

    #[validate(url)]
    #[serde(rename = "headerUrl")]
    pub header_url: Option<String>,
    #[serde(rename = "customCss")]
    pub custom_css: Option<String>,

    #[validate(length(max = 8))]
    #[schema(example = json!(["username1", "username2"]))]
    #[serde(rename = "topFriends")]
    pub top_friends: Option<Vec<String>>,
}
