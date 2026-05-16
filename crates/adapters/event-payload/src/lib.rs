use domain::{
    errors::DomainError,
    events::DomainEvent,
    value_objects::{BoostId, LikeId, ThoughtId, UserId},
};
use serde::{Deserialize, Serialize};

/// Serializable mirror of domain::events::DomainEvent.
/// All IDs are Strings (UUID hex) — no domain type dependencies.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum EventPayload {
    ThoughtCreated {
        thought_id: String,
        user_id: String,
        in_reply_to_id: Option<String>,
    },
    ThoughtDeleted {
        thought_id: String,
        user_id: String,
    },
    ThoughtUpdated {
        thought_id: String,
        user_id: String,
    },
    LikeAdded {
        like_id: String,
        user_id: String,
        thought_id: String,
    },
    LikeRemoved {
        user_id: String,
        thought_id: String,
    },
    BoostAdded {
        boost_id: String,
        user_id: String,
        thought_id: String,
    },
    BoostRemoved {
        user_id: String,
        thought_id: String,
    },
    FollowRequested {
        follower_id: String,
        following_id: String,
    },
    FollowAccepted {
        follower_id: String,
        following_id: String,
    },
    FollowRejected {
        follower_id: String,
        following_id: String,
    },
    Unfollowed {
        follower_id: String,
        following_id: String,
    },
    UserBlocked {
        blocker_id: String,
        blocked_id: String,
    },
    UserUnblocked {
        blocker_id: String,
        blocked_id: String,
    },
    UserRegistered {
        user_id: String,
    },
    ProfileUpdated {
        user_id: String,
    },
    MentionReceived {
        thought_id: String,
        mentioned_user_id: String,
        author_user_id: String,
    },
}

impl EventPayload {
    /// Returns the NATS subject for this event.
    pub fn subject(&self) -> &'static str {
        match self {
            Self::ThoughtCreated { .. } => "thoughts.created",
            Self::ThoughtDeleted { .. } => "thoughts.deleted",
            Self::ThoughtUpdated { .. } => "thoughts.updated",
            Self::LikeAdded { .. } => "likes.added",
            Self::LikeRemoved { .. } => "likes.removed",
            Self::BoostAdded { .. } => "boosts.added",
            Self::BoostRemoved { .. } => "boosts.removed",
            Self::FollowRequested { .. } => "follows.requested",
            Self::FollowAccepted { .. } => "follows.accepted",
            Self::FollowRejected { .. } => "follows.rejected",
            Self::Unfollowed { .. } => "follows.removed",
            Self::UserBlocked { .. } => "users.blocked",
            Self::UserUnblocked { .. } => "users.unblocked",
            Self::UserRegistered { .. } => "users.registered",
            Self::ProfileUpdated { .. } => "users.profile_updated",
            Self::MentionReceived { .. } => "mentions.received",
        }
    }
}

// ── DomainEvent → EventPayload ─────────────────────────────────────────────

impl From<&DomainEvent> for EventPayload {
    fn from(e: &DomainEvent) -> Self {
        match e {
            DomainEvent::ThoughtCreated {
                thought_id,
                user_id,
                in_reply_to_id,
            } => Self::ThoughtCreated {
                thought_id: thought_id.to_string(),
                user_id: user_id.to_string(),
                in_reply_to_id: in_reply_to_id.as_ref().map(|x| x.to_string()),
            },
            DomainEvent::ThoughtDeleted {
                thought_id,
                user_id,
            } => Self::ThoughtDeleted {
                thought_id: thought_id.to_string(),
                user_id: user_id.to_string(),
            },
            DomainEvent::ThoughtUpdated {
                thought_id,
                user_id,
            } => Self::ThoughtUpdated {
                thought_id: thought_id.to_string(),
                user_id: user_id.to_string(),
            },
            DomainEvent::LikeAdded {
                like_id,
                user_id,
                thought_id,
            } => Self::LikeAdded {
                like_id: like_id.to_string(),
                user_id: user_id.to_string(),
                thought_id: thought_id.to_string(),
            },
            DomainEvent::LikeRemoved {
                user_id,
                thought_id,
            } => Self::LikeRemoved {
                user_id: user_id.to_string(),
                thought_id: thought_id.to_string(),
            },
            DomainEvent::BoostAdded {
                boost_id,
                user_id,
                thought_id,
            } => Self::BoostAdded {
                boost_id: boost_id.to_string(),
                user_id: user_id.to_string(),
                thought_id: thought_id.to_string(),
            },
            DomainEvent::BoostRemoved {
                user_id,
                thought_id,
            } => Self::BoostRemoved {
                user_id: user_id.to_string(),
                thought_id: thought_id.to_string(),
            },
            DomainEvent::FollowRequested {
                follower_id,
                following_id,
            } => Self::FollowRequested {
                follower_id: follower_id.to_string(),
                following_id: following_id.to_string(),
            },
            DomainEvent::FollowAccepted {
                follower_id,
                following_id,
            } => Self::FollowAccepted {
                follower_id: follower_id.to_string(),
                following_id: following_id.to_string(),
            },
            DomainEvent::FollowRejected {
                follower_id,
                following_id,
            } => Self::FollowRejected {
                follower_id: follower_id.to_string(),
                following_id: following_id.to_string(),
            },
            DomainEvent::Unfollowed {
                follower_id,
                following_id,
            } => Self::Unfollowed {
                follower_id: follower_id.to_string(),
                following_id: following_id.to_string(),
            },
            DomainEvent::UserBlocked {
                blocker_id,
                blocked_id,
            } => Self::UserBlocked {
                blocker_id: blocker_id.to_string(),
                blocked_id: blocked_id.to_string(),
            },
            DomainEvent::UserUnblocked {
                blocker_id,
                blocked_id,
            } => Self::UserUnblocked {
                blocker_id: blocker_id.to_string(),
                blocked_id: blocked_id.to_string(),
            },
            DomainEvent::UserRegistered { user_id } => Self::UserRegistered {
                user_id: user_id.to_string(),
            },
            DomainEvent::ProfileUpdated { user_id } => Self::ProfileUpdated {
                user_id: user_id.to_string(),
            },
            DomainEvent::MentionReceived {
                thought_id,
                mentioned_user_id,
                author_user_id,
            } => Self::MentionReceived {
                thought_id: thought_id.to_string(),
                mentioned_user_id: mentioned_user_id.to_string(),
                author_user_id: author_user_id.to_string(),
            },
        }
    }
}

// ── EventPayload → DomainEvent ─────────────────────────────────────────────

fn parse_uuid(s: &str, field: &str) -> Result<uuid::Uuid, DomainError> {
    uuid::Uuid::parse_str(s)
        .map_err(|_| DomainError::Internal(format!("invalid uuid for {field}: {s}")))
}

impl TryFrom<EventPayload> for DomainEvent {
    type Error = DomainError;

    fn try_from(p: EventPayload) -> Result<Self, DomainError> {
        Ok(match p {
            EventPayload::ThoughtCreated {
                thought_id,
                user_id,
                in_reply_to_id,
            } => DomainEvent::ThoughtCreated {
                thought_id: ThoughtId::from_uuid(parse_uuid(&thought_id, "thought_id")?),
                user_id: UserId::from_uuid(parse_uuid(&user_id, "user_id")?),
                in_reply_to_id: in_reply_to_id
                    .map(|s| parse_uuid(&s, "in_reply_to_id").map(ThoughtId::from_uuid))
                    .transpose()?,
            },
            EventPayload::ThoughtDeleted {
                thought_id,
                user_id,
            } => DomainEvent::ThoughtDeleted {
                thought_id: ThoughtId::from_uuid(parse_uuid(&thought_id, "thought_id")?),
                user_id: UserId::from_uuid(parse_uuid(&user_id, "user_id")?),
            },
            EventPayload::ThoughtUpdated {
                thought_id,
                user_id,
            } => DomainEvent::ThoughtUpdated {
                thought_id: ThoughtId::from_uuid(parse_uuid(&thought_id, "thought_id")?),
                user_id: UserId::from_uuid(parse_uuid(&user_id, "user_id")?),
            },
            EventPayload::LikeAdded {
                like_id,
                user_id,
                thought_id,
            } => DomainEvent::LikeAdded {
                like_id: LikeId::from_uuid(parse_uuid(&like_id, "like_id")?),
                user_id: UserId::from_uuid(parse_uuid(&user_id, "user_id")?),
                thought_id: ThoughtId::from_uuid(parse_uuid(&thought_id, "thought_id")?),
            },
            EventPayload::LikeRemoved {
                user_id,
                thought_id,
            } => DomainEvent::LikeRemoved {
                user_id: UserId::from_uuid(parse_uuid(&user_id, "user_id")?),
                thought_id: ThoughtId::from_uuid(parse_uuid(&thought_id, "thought_id")?),
            },
            EventPayload::BoostAdded {
                boost_id,
                user_id,
                thought_id,
            } => DomainEvent::BoostAdded {
                boost_id: BoostId::from_uuid(parse_uuid(&boost_id, "boost_id")?),
                user_id: UserId::from_uuid(parse_uuid(&user_id, "user_id")?),
                thought_id: ThoughtId::from_uuid(parse_uuid(&thought_id, "thought_id")?),
            },
            EventPayload::BoostRemoved {
                user_id,
                thought_id,
            } => DomainEvent::BoostRemoved {
                user_id: UserId::from_uuid(parse_uuid(&user_id, "user_id")?),
                thought_id: ThoughtId::from_uuid(parse_uuid(&thought_id, "thought_id")?),
            },
            EventPayload::FollowRequested {
                follower_id,
                following_id,
            } => DomainEvent::FollowRequested {
                follower_id: UserId::from_uuid(parse_uuid(&follower_id, "follower_id")?),
                following_id: UserId::from_uuid(parse_uuid(&following_id, "following_id")?),
            },
            EventPayload::FollowAccepted {
                follower_id,
                following_id,
            } => DomainEvent::FollowAccepted {
                follower_id: UserId::from_uuid(parse_uuid(&follower_id, "follower_id")?),
                following_id: UserId::from_uuid(parse_uuid(&following_id, "following_id")?),
            },
            EventPayload::FollowRejected {
                follower_id,
                following_id,
            } => DomainEvent::FollowRejected {
                follower_id: UserId::from_uuid(parse_uuid(&follower_id, "follower_id")?),
                following_id: UserId::from_uuid(parse_uuid(&following_id, "following_id")?),
            },
            EventPayload::Unfollowed {
                follower_id,
                following_id,
            } => DomainEvent::Unfollowed {
                follower_id: UserId::from_uuid(parse_uuid(&follower_id, "follower_id")?),
                following_id: UserId::from_uuid(parse_uuid(&following_id, "following_id")?),
            },
            EventPayload::UserBlocked {
                blocker_id,
                blocked_id,
            } => DomainEvent::UserBlocked {
                blocker_id: UserId::from_uuid(parse_uuid(&blocker_id, "blocker_id")?),
                blocked_id: UserId::from_uuid(parse_uuid(&blocked_id, "blocked_id")?),
            },
            EventPayload::UserUnblocked {
                blocker_id,
                blocked_id,
            } => DomainEvent::UserUnblocked {
                blocker_id: UserId::from_uuid(parse_uuid(&blocker_id, "blocker_id")?),
                blocked_id: UserId::from_uuid(parse_uuid(&blocked_id, "blocked_id")?),
            },
            EventPayload::UserRegistered { user_id } => DomainEvent::UserRegistered {
                user_id: UserId::from_uuid(parse_uuid(&user_id, "user_id")?),
            },
            EventPayload::ProfileUpdated { user_id } => DomainEvent::ProfileUpdated {
                user_id: UserId::from_uuid(parse_uuid(&user_id, "user_id")?),
            },
            EventPayload::MentionReceived {
                thought_id,
                mentioned_user_id,
                author_user_id,
            } => DomainEvent::MentionReceived {
                thought_id: ThoughtId::from_uuid(parse_uuid(&thought_id, "thought_id")?),
                mentioned_user_id: UserId::from_uuid(parse_uuid(
                    &mentioned_user_id,
                    "mentioned_user_id",
                )?),
                author_user_id: UserId::from_uuid(parse_uuid(&author_user_id, "author_user_id")?),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thought_created_roundtrip() {
        let p = EventPayload::ThoughtCreated {
            thought_id: "abc".into(),
            user_id: "def".into(),
            in_reply_to_id: None,
        };
        let json = serde_json::to_string(&p).unwrap();
        let back: EventPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(back.subject(), "thoughts.created");
    }

    #[test]
    fn all_subjects_are_unique() {
        let samples: &[EventPayload] = &[
            EventPayload::ThoughtCreated {
                thought_id: "a".into(),
                user_id: "b".into(),
                in_reply_to_id: None,
            },
            EventPayload::ThoughtDeleted {
                thought_id: "a".into(),
                user_id: "b".into(),
            },
            EventPayload::ThoughtUpdated {
                thought_id: "a".into(),
                user_id: "b".into(),
            },
            EventPayload::LikeAdded {
                like_id: "a".into(),
                user_id: "b".into(),
                thought_id: "c".into(),
            },
            EventPayload::LikeRemoved {
                user_id: "b".into(),
                thought_id: "c".into(),
            },
            EventPayload::BoostAdded {
                boost_id: "a".into(),
                user_id: "b".into(),
                thought_id: "c".into(),
            },
            EventPayload::BoostRemoved {
                user_id: "b".into(),
                thought_id: "c".into(),
            },
            EventPayload::FollowRequested {
                follower_id: "a".into(),
                following_id: "b".into(),
            },
            EventPayload::FollowAccepted {
                follower_id: "a".into(),
                following_id: "b".into(),
            },
            EventPayload::FollowRejected {
                follower_id: "a".into(),
                following_id: "b".into(),
            },
            EventPayload::Unfollowed {
                follower_id: "a".into(),
                following_id: "b".into(),
            },
            EventPayload::UserBlocked {
                blocker_id: "a".into(),
                blocked_id: "b".into(),
            },
            EventPayload::UserUnblocked {
                blocker_id: "a".into(),
                blocked_id: "b".into(),
            },
            EventPayload::UserRegistered {
                user_id: "a".into(),
            },
        ];
        let mut subjects: Vec<&str> = samples.iter().map(|p| p.subject()).collect();
        subjects.sort();
        subjects.dedup();
        assert_eq!(
            subjects.len(),
            samples.len(),
            "each event must have a unique subject"
        );
    }
}
