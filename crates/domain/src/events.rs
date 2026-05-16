use crate::value_objects::{BoostId, LikeId, ThoughtId, UserId};

#[derive(Debug, Clone)]
pub enum DomainEvent {
    ThoughtCreated {
        thought_id: ThoughtId,
        user_id: UserId,
        in_reply_to_id: Option<ThoughtId>,
    },
    ThoughtDeleted {
        thought_id: ThoughtId,
        user_id: UserId,
    },
    ThoughtUpdated {
        thought_id: ThoughtId,
        user_id: UserId,
    },
    LikeAdded {
        like_id: LikeId,
        user_id: UserId,
        thought_id: ThoughtId,
    },
    LikeRemoved {
        user_id: UserId,
        thought_id: ThoughtId,
    },
    BoostAdded {
        boost_id: BoostId,
        user_id: UserId,
        thought_id: ThoughtId,
    },
    BoostRemoved {
        user_id: UserId,
        thought_id: ThoughtId,
    },
    FollowRequested {
        follower_id: UserId,
        following_id: UserId,
    },
    FollowAccepted {
        follower_id: UserId,
        following_id: UserId,
    },
    FollowRejected {
        follower_id: UserId,
        following_id: UserId,
    },
    Unfollowed {
        follower_id: UserId,
        following_id: UserId,
    },
    UserBlocked {
        blocker_id: UserId,
        blocked_id: UserId,
    },
    UserUnblocked {
        blocker_id: UserId,
        blocked_id: UserId,
    },
    UserRegistered {
        user_id: UserId,
    },
    ProfileUpdated {
        user_id: UserId,
    },
    MentionReceived {
        thought_id: ThoughtId,
        mentioned_user_id: UserId,
        author_user_id: UserId,
    },
}

pub struct EventEnvelope {
    pub event: DomainEvent,
    pub delivery_count: u64,
    pub ack: Box<dyn Fn() + Send + Sync>,
    pub nack: Box<dyn Fn() + Send + Sync>,
}
impl std::fmt::Debug for EventEnvelope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventEnvelope")
            .field("event", &self.event)
            .field("delivery_count", &self.delivery_count)
            .finish()
    }
}
