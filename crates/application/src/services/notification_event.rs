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
mod tests {
    use super::*;
    use domain::{
        models::{
            notification::NotificationKind,
            thought::{Thought, Visibility},
            user::User,
        },
        testing::TestStore,
        value_objects::*,
    };
    use std::sync::Arc;

    fn alice() -> User {
        User::new_local(
            UserId::new(),
            Username::new("alice").unwrap(),
            Email::new("alice@ex.com").unwrap(),
            PasswordHash("h".into()),
        )
    }

    #[tokio::test]
    async fn like_creates_notification_for_thought_author() {
        let store = TestStore::default();
        let alice = alice();
        let bob_id = UserId::new();
        let thought = Thought::new_local(
            ThoughtId::new(),
            alice.id.clone(),
            Content::new_local("hello").unwrap(),
            None,
            Visibility::Public,
            None,
            false,
        );
        store.thoughts.lock().unwrap().push(thought.clone());
        let svc = NotificationEventService {
            thoughts: Arc::new(store.clone()),
            notifications: Arc::new(store.clone()),
        };
        svc.process(&DomainEvent::LikeAdded {
            like_id: LikeId::new(),
            user_id: bob_id,
            thought_id: thought.id.clone(),
        })
        .await
        .unwrap();
        let notifs = store.notifications.lock().unwrap();
        assert_eq!(notifs.len(), 1);
        assert!(matches!(notifs[0].kind, NotificationKind::Like { .. }));
    }

    #[tokio::test]
    async fn self_like_creates_no_notification() {
        let store = TestStore::default();
        let alice = alice();
        let thought = Thought::new_local(
            ThoughtId::new(),
            alice.id.clone(),
            Content::new_local("hello").unwrap(),
            None,
            Visibility::Public,
            None,
            false,
        );
        store.thoughts.lock().unwrap().push(thought.clone());
        let svc = NotificationEventService {
            thoughts: Arc::new(store.clone()),
            notifications: Arc::new(store.clone()),
        };
        svc.process(&DomainEvent::LikeAdded {
            like_id: LikeId::new(),
            user_id: alice.id.clone(),
            thought_id: thought.id.clone(),
        })
        .await
        .unwrap();
        assert!(store.notifications.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn follow_accepted_creates_notification() {
        let store = TestStore::default();
        let alice = alice();
        let bob_id = UserId::new();
        let svc = NotificationEventService {
            thoughts: Arc::new(store.clone()),
            notifications: Arc::new(store.clone()),
        };
        svc.process(&DomainEvent::FollowAccepted {
            follower_id: bob_id,
            following_id: alice.id.clone(),
        })
        .await
        .unwrap();
        let notifs = store.notifications.lock().unwrap();
        assert_eq!(notifs.len(), 1);
        assert!(matches!(notifs[0].kind, NotificationKind::Follow { .. }));
    }

    #[tokio::test]
    async fn reply_creates_notification_for_original_author() {
        let store = TestStore::default();
        let alice = alice();
        let bob_id = UserId::new();
        let original = Thought::new_local(
            ThoughtId::new(),
            alice.id.clone(),
            Content::new_local("original").unwrap(),
            None,
            Visibility::Public,
            None,
            false,
        );
        store.thoughts.lock().unwrap().push(original.clone());
        let svc = NotificationEventService {
            thoughts: Arc::new(store.clone()),
            notifications: Arc::new(store.clone()),
        };
        svc.process(&DomainEvent::ThoughtCreated {
            thought_id: ThoughtId::new(),
            user_id: bob_id,
            in_reply_to_id: Some(original.id.clone()),
        })
        .await
        .unwrap();
        let notifs = store.notifications.lock().unwrap();
        assert_eq!(notifs.len(), 1);
        assert!(matches!(notifs[0].kind, NotificationKind::Reply { .. }));
    }

    #[tokio::test]
    async fn self_reply_creates_no_notification() {
        let store = TestStore::default();
        let alice = alice();
        let original = Thought::new_local(
            ThoughtId::new(),
            alice.id.clone(),
            Content::new_local("original").unwrap(),
            None,
            Visibility::Public,
            None,
            false,
        );
        store.thoughts.lock().unwrap().push(original.clone());
        let svc = NotificationEventService {
            thoughts: Arc::new(store.clone()),
            notifications: Arc::new(store.clone()),
        };
        svc.process(&DomainEvent::ThoughtCreated {
            thought_id: ThoughtId::new(),
            user_id: alice.id.clone(),
            in_reply_to_id: Some(original.id.clone()),
        })
        .await
        .unwrap();
        assert!(store.notifications.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn self_boost_creates_no_notification() {
        let store = TestStore::default();
        let alice = alice();
        let thought = Thought::new_local(
            ThoughtId::new(),
            alice.id.clone(),
            Content::new_local("hello").unwrap(),
            None,
            Visibility::Public,
            None,
            false,
        );
        store.thoughts.lock().unwrap().push(thought.clone());
        let svc = NotificationEventService {
            thoughts: Arc::new(store.clone()),
            notifications: Arc::new(store.clone()),
        };
        svc.process(&DomainEvent::BoostAdded {
            boost_id: BoostId::new(),
            user_id: alice.id.clone(),
            thought_id: thought.id.clone(),
        })
        .await
        .unwrap();
        assert!(store.notifications.lock().unwrap().is_empty());
    }
}
