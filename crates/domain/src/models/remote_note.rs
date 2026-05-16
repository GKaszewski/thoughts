use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct RemoteNote {
    pub ap_id: String,
    pub content: String,
    pub published: DateTime<Utc>,
    pub sensitive: bool,
    pub content_warning: Option<String>,
}
