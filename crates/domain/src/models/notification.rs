use crate::value_objects::{NotificationId, ThoughtId, UserId};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq)]
pub enum NotificationKind {
    Like {
        thought_id: ThoughtId,
        from_user_id: UserId,
    },
    Boost {
        thought_id: ThoughtId,
        from_user_id: UserId,
    },
    Reply {
        thought_id: ThoughtId,
        from_user_id: UserId,
    },
    Mention {
        thought_id: ThoughtId,
        from_user_id: UserId,
    },
    Follow {
        from_user_id: UserId,
    },
}

impl NotificationKind {
    pub fn from_user_id(&self) -> &UserId {
        match self {
            Self::Like { from_user_id, .. } => from_user_id,
            Self::Boost { from_user_id, .. } => from_user_id,
            Self::Reply { from_user_id, .. } => from_user_id,
            Self::Mention { from_user_id, .. } => from_user_id,
            Self::Follow { from_user_id } => from_user_id,
        }
    }

    pub fn thought_id(&self) -> Option<&ThoughtId> {
        match self {
            Self::Like { thought_id, .. } => Some(thought_id),
            Self::Boost { thought_id, .. } => Some(thought_id),
            Self::Reply { thought_id, .. } => Some(thought_id),
            Self::Mention { thought_id, .. } => Some(thought_id),
            Self::Follow { .. } => None,
        }
    }

    pub fn kind_str(&self) -> &'static str {
        match self {
            Self::Like { .. } => "like",
            Self::Boost { .. } => "boost",
            Self::Reply { .. } => "reply",
            Self::Mention { .. } => "mention",
            Self::Follow { .. } => "follow",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Notification {
    pub id: NotificationId,
    pub user_id: UserId,
    pub kind: NotificationKind,
    pub read: bool,
    pub created_at: DateTime<Utc>,
}
