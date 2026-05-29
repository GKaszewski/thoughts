use serde::Deserialize;
use uuid::Uuid;

pub const DEFAULT_PAGE: u64 = 1;
pub const DEFAULT_PER_PAGE: u64 = 20;
pub const MAX_PER_PAGE: u64 = 100;

#[derive(Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RegisterRequest {
    /// Username (1-32 chars, alphanumeric + underscore)
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateThoughtRequest {
    /// Up to 128 characters
    pub content: String,
    pub in_reply_to_id: Option<Uuid>,
    /// One of: "public", "followers", "unlisted", "direct"
    pub visibility: Option<String>,
    pub content_warning: Option<String>,
    pub sensitive: Option<bool>,
    pub mood: Option<String>,
}

#[derive(Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct EditThoughtRequest {
    pub content: String,
}

#[derive(Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProfileRequest {
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub header_url: Option<String>,
    pub custom_css: Option<String>,
    pub profile_fields: Option<Vec<crate::responses::ProfileField>>,
    pub custom_moods: Option<Vec<crate::responses::ProfileField>>,
}

#[derive(Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SetTopFriendsRequest {
    /// Ordered list of user UUIDs, max 8
    pub friend_ids: Vec<Uuid>,
}

#[derive(Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateApiKeyRequest {
    pub name: String,
}

#[derive(Deserialize, utoipa::IntoParams)]
pub struct PaginationQuery {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

impl PaginationQuery {
    pub fn page(&self) -> u64 {
        self.page.unwrap_or(DEFAULT_PAGE).max(DEFAULT_PAGE)
    }

    pub fn per_page(&self) -> u64 {
        self.per_page.unwrap_or(DEFAULT_PER_PAGE).min(MAX_PER_PAGE)
    }
}

#[derive(Deserialize, utoipa::IntoParams)]
pub struct SearchQuery {
    pub q: String,
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NotificationUpdateRequest {
    pub read: bool,
}
