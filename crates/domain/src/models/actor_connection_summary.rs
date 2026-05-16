#[derive(Debug, Clone)]
pub struct ActorConnectionSummary {
    pub url: String,
    pub handle: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}
