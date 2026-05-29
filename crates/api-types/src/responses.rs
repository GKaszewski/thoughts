use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuthResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Serialize, Clone, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub header_url: Option<String>,
    pub custom_css: Option<String>,
    pub local: bool,
    pub is_followed_by_viewer: bool,
    #[serde(rename = "joinedAt")]
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Clone, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ThoughtResponse {
    pub id: Uuid,
    pub content: String,
    pub author: UserResponse,
    #[serde(rename = "replyToId")]
    pub in_reply_to_id: Option<Uuid>,
    #[serde(rename = "replyToUrl", skip_serializing_if = "Option::is_none")]
    pub in_reply_to_url: Option<String>,
    pub visibility: String,
    pub content_warning: Option<String>,
    pub sensitive: bool,
    pub like_count: i64,
    pub boost_count: i64,
    pub reply_count: i64,
    pub liked_by_viewer: bool,
    pub boosted_by_viewer: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note_extensions: Option<serde_json::Value>,
}

#[derive(Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PagedResponse<T: Serialize + utoipa::ToSchema> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: u64,
    pub per_page: u64,
}

#[derive(Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeyResponse {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NotificationResponse {
    pub id: Uuid,
    pub notification_type: String,
    pub from_user: Option<UserResponse>,
    pub thought_id: Option<Uuid>,
    pub read: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TopFriendsResponse {
    pub top_friends: Vec<UserResponse>,
}

#[derive(Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreatedApiKeyResponse {
    pub id: Uuid,
    pub name: String,
    /// Raw API key — shown only once at creation
    pub key: String,
}

#[derive(Serialize, Clone, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProfileField {
    pub name: String,
    pub value: String,
}

#[derive(Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RemoteActorResponse {
    pub handle: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub url: String,
    pub bio: Option<String>,
    pub banner_url: Option<String>,
    pub also_known_as: Vec<String>,
    pub outbox_url: Option<String>,
    pub followers_url: Option<String>,
    pub following_url: Option<String>,
    pub attachment: Vec<ProfileField>,
}

#[derive(Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ActorConnectionResponse {
    pub handle: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub url: String,
}

#[derive(Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ActorConnectionPageResponse {
    pub items: Vec<ActorConnectionResponse>,
    pub page: u32,
    pub has_more: bool,
}
