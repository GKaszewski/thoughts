use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct RemoteActor {
    pub url: String,
    pub handle: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub banner_url: Option<String>,
    pub also_known_as: Vec<String>,
    pub outbox_url: Option<String>,
    pub followers_url: Option<String>,
    pub following_url: Option<String>,
    pub inbox_url: Option<String>,
    pub shared_inbox_url: Option<String>,
    pub attachment: Vec<(String, String)>,
    pub last_fetched_at: DateTime<Utc>,
}
