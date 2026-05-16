use crate::models::{thought::Thought, user::User};
use crate::value_objects::UserId;

#[derive(Debug, Clone)]
pub struct EngagementStats {
    pub like_count: i64,
    pub boost_count: i64,
    pub reply_count: i64,
}

/// Present only when an authenticated viewer made the request.
/// `liked`/`boosted` are the viewer's interaction state with this thought.
/// `None` means anonymous request or viewer context unavailable.
#[derive(Debug, Clone)]
pub struct ViewerContext {
    pub liked: bool,
    pub boosted: bool,
}

#[derive(Debug, Clone)]
pub struct FeedEntry {
    pub thought: Thought,
    pub author: User,
    pub stats: EngagementStats,
    pub viewer: Option<ViewerContext>,
}

#[derive(Debug, Clone)]
pub struct UserSummary {
    pub id: UserId,
    pub username: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub thought_count: i64,
    pub follower_count: i64,
    pub following_count: i64,
}

#[derive(Debug, Clone)]
pub struct PageParams {
    pub page: u64,
    pub per_page: u64,
}
impl PageParams {
    pub fn offset(&self) -> i64 {
        ((self.page.saturating_sub(1)) * self.per_page) as i64
    }
    pub fn limit(&self) -> i64 {
        self.per_page as i64
    }
}

#[derive(Debug, Clone)]
pub struct Paginated<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: u64,
    pub per_page: u64,
}
