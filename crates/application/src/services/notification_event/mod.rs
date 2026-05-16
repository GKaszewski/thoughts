use chrono::Utc;
use domain::{
    errors::DomainError,
    events::DomainEvent,
    models::notification::{Notification, NotificationKind},
    ports::{NotificationRepository, ThoughtRepository},
    value_objects::NotificationId,
};
use std::sync::Arc;

pub struct NotificationEventService {
    pub thoughts: Arc<dyn ThoughtRepository>,
    pub notifications: Arc<dyn NotificationRepository>,
}

fn is_self_action(
    thought_author: &domain::value_objects::UserId,
    actor: &domain::value_objects::UserId,
) -> bool {
    thought_author == actor
}

impl NotificationEventService {
    pub async fn process(&self, event: &DomainEvent) -> Result<(), DomainError> {
        match event {
            DomainEvent::LikeAdded {
                like_id: _,
                user_id,
                thought_id,
            } => {
                let thought = match self.thoughts.find_by_id(thought_id).await? {
                    Some(t) => t,
                    None => return Ok(()),
                };
                if is_self_action(&thought.user_id, user_id) {
                    return Ok(());
                }
                self.notifications
                    .save(&Notification {
                        id: NotificationId::new(),
                        user_id: thought.user_id,
                        kind: NotificationKind::Like {
                            thought_id: thought_id.clone(),
                            from_user_id: user_id.clone(),
                        },
                        read: false,
                        created_at: Utc::now(),
                    })
                    .await
            }
            DomainEvent::BoostAdded {
                boost_id: _,
                user_id,
                thought_id,
            } => {
                let thought = match self.thoughts.find_by_id(thought_id).await? {
                    Some(t) => t,
                    None => return Ok(()),
                };
                if is_self_action(&thought.user_id, user_id) {
                    return Ok(());
                }
                self.notifications
                    .save(&Notification {
                        id: NotificationId::new(),
                        user_id: thought.user_id,
                        kind: NotificationKind::Boost {
                            thought_id: thought_id.clone(),
                            from_user_id: user_id.clone(),
                        },
                        read: false,
                        created_at: Utc::now(),
                    })
                    .await
            }
            DomainEvent::FollowAccepted {
                follower_id,
                following_id,
            } => {
                self.notifications
                    .save(&Notification {
                        id: NotificationId::new(),
                        user_id: following_id.clone(),
                        kind: NotificationKind::Follow {
                            from_user_id: follower_id.clone(),
                        },
                        read: false,
                        created_at: Utc::now(),
                    })
                    .await
            }
            DomainEvent::ThoughtCreated {
                thought_id,
                user_id,
                in_reply_to_id,
            } => {
                let reply_to_id = match in_reply_to_id {
                    Some(id) => id,
                    None => return Ok(()),
                };
                let original = match self.thoughts.find_by_id(reply_to_id).await? {
                    Some(t) => t,
                    None => return Ok(()),
                };
                if is_self_action(&original.user_id, user_id) {
                    return Ok(());
                }
                self.notifications
                    .save(&Notification {
                        id: NotificationId::new(),
                        user_id: original.user_id,
                        kind: NotificationKind::Reply {
                            thought_id: thought_id.clone(),
                            from_user_id: user_id.clone(),
                        },
                        read: false,
                        created_at: Utc::now(),
                    })
                    .await
            }
            DomainEvent::MentionReceived {
                thought_id,
                mentioned_user_id,
                author_user_id,
            } => {
                self.notifications
                    .save(&Notification {
                        id: NotificationId::new(),
                        user_id: mentioned_user_id.clone(),
                        kind: NotificationKind::Mention {
                            thought_id: thought_id.clone(),
                            from_user_id: author_user_id.clone(),
                        },
                        read: false,
                        created_at: Utc::now(),
                    })
                    .await
            }
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests;
