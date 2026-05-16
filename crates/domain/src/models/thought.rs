use crate::value_objects::{Content, ThoughtId, UserId};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Visibility {
    Public,
    Followers,
    Unlisted,
    Direct,
}

#[derive(Debug, Clone)]
pub struct Thought {
    pub id: ThoughtId,
    pub user_id: UserId,
    pub content: Content,
    pub in_reply_to_id: Option<ThoughtId>,
    pub visibility: Visibility,
    pub content_warning: Option<String>,
    pub sensitive: bool,
    pub local: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Visibility {
    pub fn as_str(&self) -> &'static str {
        match self {
            Visibility::Public => "public",
            Visibility::Followers => "followers",
            Visibility::Unlisted => "unlisted",
            Visibility::Direct => "direct",
        }
    }

    pub fn from_db_str(s: &str) -> Result<Self, crate::errors::DomainError> {
        match s {
            "public" => Ok(Self::Public),
            "followers" => Ok(Self::Followers),
            "unlisted" => Ok(Self::Unlisted),
            "direct" => Ok(Self::Direct),
            other => Err(crate::errors::DomainError::Internal(format!(
                "unknown visibility: '{other}'"
            ))),
        }
    }
}

impl Thought {
    pub fn new_local(
        id: ThoughtId,
        user_id: UserId,
        content: Content,
        in_reply_to_id: Option<ThoughtId>,
        visibility: Visibility,
        content_warning: Option<String>,
        sensitive: bool,
    ) -> Self {
        Self {
            id,
            user_id,
            content,
            in_reply_to_id,
            visibility,
            content_warning,
            sensitive,
            local: true,
            created_at: Utc::now(),
            updated_at: None,
        }
    }
}
