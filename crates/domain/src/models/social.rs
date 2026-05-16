use crate::value_objects::{BoostId, LikeId, ThoughtId, UserId};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct Like {
    pub id: LikeId,
    pub user_id: UserId,
    pub thought_id: ThoughtId,
    pub ap_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Boost {
    pub id: BoostId,
    pub user_id: UserId,
    pub thought_id: ThoughtId,
    pub ap_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FollowState {
    Pending,
    Accepted,
    Rejected,
}

impl FollowState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
        }
    }

    pub fn from_db_str(s: &str) -> Result<Self, crate::errors::DomainError> {
        match s {
            "pending" => Ok(Self::Pending),
            "accepted" => Ok(Self::Accepted),
            "rejected" => Ok(Self::Rejected),
            other => Err(crate::errors::DomainError::Internal(format!(
                "unknown follow_state: '{other}'"
            ))),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Follow {
    pub follower_id: UserId,
    pub following_id: UserId,
    pub state: FollowState,
    pub ap_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub blocker_id: UserId,
    pub blocked_id: UserId,
    pub created_at: DateTime<Utc>,
}
