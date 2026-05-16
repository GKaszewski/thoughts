use activitypub_base::{ActivityPubRepository, OutboundFederationPort};
use domain::{
    errors::DomainError,
    events::DomainEvent,
    models::thought::Visibility,
    ports::{ThoughtRepository, UserReader},
    value_objects::ThoughtId,
};
use std::sync::Arc;

pub struct FederationEventService {
    pub thoughts: Arc<dyn ThoughtRepository>,
    pub users: Arc<dyn UserReader>,
    pub ap: Arc<dyn OutboundFederationPort>,
    pub base_url: String,
    pub ap_repo: Arc<dyn ActivityPubRepository>,
}

impl FederationEventService {
    async fn object_ap_id(&self, thought_id: &ThoughtId) -> Result<String, DomainError> {
        if let Some(ap_id) = self.ap_repo.get_thought_ap_id(thought_id).await? {
            return Ok(ap_id);
        }
        Ok(format!("{}/thoughts/{}", self.base_url, thought_id))
    }

    pub async fn process(&self, event: &DomainEvent) -> Result<(), DomainError> {
        match event {
            DomainEvent::ThoughtCreated {
                thought_id,
                user_id,
                ..
            } => {
                let thought = match self.thoughts.find_by_id(thought_id).await? {
                    Some(t)
                        if t.local
                            && matches!(
                                t.visibility,
                                Visibility::Public | Visibility::Unlisted
                            ) =>
                    {
                        t
                    }
                    _ => return Ok(()),
                };
                let user = match self.users.find_by_id(user_id).await? {
                    Some(u) => u,
                    None => return Ok(()),
                };
                // Resolve in_reply_to_url for the parent thought via AP repo.
                let in_reply_to_url = if let Some(ref reply_id) = thought.in_reply_to_id {
                    let ap_id = self
                        .ap_repo
                        .get_thought_ap_id(reply_id)
                        .await?
                        .unwrap_or_else(|| format!("{}/thoughts/{}", self.base_url, reply_id));
                    Some(ap_id)
                } else {
                    None
                };
                self.ap
                    .broadcast_create(
                        user_id,
                        &thought,
                        user.username.as_str(),
                        in_reply_to_url.as_deref(),
                    )
                    .await
            }

            DomainEvent::ThoughtDeleted {
                thought_id,
                user_id,
            } => {
                // No DB lookup — thought is already deleted when this event fires.
                // No locality guard: delete commands only reach local thoughts via the use case.
                let ap_id = format!("{}/thoughts/{}", self.base_url, thought_id);
                self.ap.broadcast_delete(user_id, &ap_id).await
            }

            DomainEvent::ThoughtUpdated {
                thought_id,
                user_id,
            } => {
                let thought = match self.thoughts.find_by_id(thought_id).await? {
                    Some(t)
                        if t.local
                            && matches!(
                                t.visibility,
                                Visibility::Public | Visibility::Unlisted
                            ) =>
                    {
                        t
                    }
                    _ => return Ok(()),
                };
                let user = match self.users.find_by_id(user_id).await? {
                    Some(u) => u,
                    None => return Ok(()),
                };
                let in_reply_to_url = if let Some(ref reply_id) = thought.in_reply_to_id {
                    self.ap_repo
                        .get_thought_ap_id(reply_id)
                        .await?
                        .or_else(|| Some(format!("{}/thoughts/{}", self.base_url, reply_id)))
                } else {
                    None
                };
                self.ap
                    .broadcast_update(
                        user_id,
                        &thought,
                        user.username.as_str(),
                        in_reply_to_url.as_deref(),
                    )
                    .await
            }

            DomainEvent::BoostAdded {
                boost_id: _,
                user_id,
                thought_id,
            } => {
                // Only fan-out if the booster is a local user. Remote boosts must not be re-broadcast.
                let booster = match self.users.find_by_id(user_id).await? {
                    Some(u) if u.local => u,
                    _ => return Ok(()),
                };
                let _ = booster;
                if self.thoughts.find_by_id(thought_id).await?.is_none() {
                    return Ok(());
                }
                let object_ap_id = self.object_ap_id(thought_id).await?;
                self.ap.broadcast_announce(user_id, &object_ap_id).await
            }

            DomainEvent::BoostRemoved {
                user_id,
                thought_id,
            } => {
                if self.thoughts.find_by_id(thought_id).await?.is_none() {
                    return Ok(());
                }
                let object_ap_id = self.object_ap_id(thought_id).await?;
                self.ap
                    .broadcast_undo_announce(user_id, &object_ap_id)
                    .await
            }

            DomainEvent::LikeAdded {
                like_id: _,
                user_id,
                thought_id,
            } => {
                // Only federate: local liker + remote thought (has ap_id) + author has inbox.
                let liker = match self.users.find_by_id(user_id).await? {
                    Some(u) if u.local => u,
                    _ => return Ok(()),
                };
                let _ = liker;
                let thought = match self.thoughts.find_by_id(thought_id).await? {
                    Some(t) => t,
                    _ => return Ok(()),
                };
                let thought_ap_id = match self.ap_repo.get_thought_ap_id(thought_id).await? {
                    Some(id) => id,
                    None => return Ok(()), // local thought — no federation needed
                };
                let actor_urls = match self.ap_repo.get_actor_ap_urls(&thought.user_id).await? {
                    Some(u) => u,
                    None => return Ok(()),
                };
                self.ap
                    .broadcast_like(user_id, &thought_ap_id, &actor_urls.inbox_url)
                    .await
            }

            DomainEvent::LikeRemoved {
                user_id,
                thought_id,
            } => {
                let liker = match self.users.find_by_id(user_id).await? {
                    Some(u) if u.local => u,
                    _ => return Ok(()),
                };
                let _ = liker;
                let thought = match self.thoughts.find_by_id(thought_id).await? {
                    Some(t) => t,
                    _ => return Ok(()),
                };
                let thought_ap_id = match self.ap_repo.get_thought_ap_id(thought_id).await? {
                    Some(id) => id,
                    None => return Ok(()),
                };
                let actor_urls = match self.ap_repo.get_actor_ap_urls(&thought.user_id).await? {
                    Some(u) => u,
                    None => return Ok(()),
                };
                self.ap
                    .broadcast_undo_like(user_id, &thought_ap_id, &actor_urls.inbox_url)
                    .await
            }

            DomainEvent::ProfileUpdated { user_id } => {
                self.ap.broadcast_actor_update(user_id).await
            }

            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use activitypub_base::{ActorApUrls, OutboundFederationPort};
    use async_trait::async_trait;
    use crate::testing::TestApRepo;
    use domain::{
        errors::DomainError,
        events::DomainEvent,
        models::thought::{Thought, Visibility},
        models::user::User,
        testing::TestStore,
        value_objects::*,
    };
    use std::sync::{Arc, Mutex};

    // ── Spy port ─────────────────────────────────────────────────────────────

    #[derive(Default)]
    struct SpyPort {
        created: Mutex<Vec<ThoughtId>>,
        deleted: Mutex<Vec<String>>,
        updated: Mutex<Vec<ThoughtId>>,
        announced: Mutex<Vec<String>>,
        undo_announced: Mutex<Vec<String>>,
        liked: Mutex<Vec<String>>,
        undo_liked: Mutex<Vec<String>>,
        actor_updated: Mutex<Vec<UserId>>,
    }

    #[async_trait]
    impl OutboundFederationPort for SpyPort {
        async fn broadcast_create(
            &self,
            _: &UserId,
            thought: &Thought,
            _: &str,
            _in_reply_to_url: Option<&str>,
        ) -> Result<(), DomainError> {
            self.created.lock().unwrap().push(thought.id.clone());
            Ok(())
        }
        async fn broadcast_delete(&self, _: &UserId, ap_id: &str) -> Result<(), DomainError> {
            self.deleted.lock().unwrap().push(ap_id.to_string());
            Ok(())
        }
        async fn broadcast_update(
            &self,
            _: &UserId,
            thought: &Thought,
            _: &str,
            _in_reply_to_url: Option<&str>,
        ) -> Result<(), DomainError> {
            self.updated.lock().unwrap().push(thought.id.clone());
            Ok(())
        }
        async fn broadcast_announce(&self, _: &UserId, ap_id: &str) -> Result<(), DomainError> {
            self.announced.lock().unwrap().push(ap_id.to_string());
            Ok(())
        }
        async fn broadcast_undo_announce(
            &self,
            _: &UserId,
            ap_id: &str,
        ) -> Result<(), DomainError> {
            self.undo_announced.lock().unwrap().push(ap_id.to_string());
            Ok(())
        }

        async fn broadcast_like(
            &self,
            _: &UserId,
            ap_id: &str,
            _: &str,
        ) -> Result<(), DomainError> {
            self.liked.lock().unwrap().push(ap_id.to_string());
            Ok(())
        }

        async fn broadcast_undo_like(
            &self,
            _: &UserId,
            ap_id: &str,
            _: &str,
        ) -> Result<(), DomainError> {
            self.undo_liked.lock().unwrap().push(ap_id.to_string());
            Ok(())
        }

        async fn broadcast_actor_update(&self, user_id: &UserId) -> Result<(), DomainError> {
            self.actor_updated.lock().unwrap().push(user_id.clone());
            Ok(())
        }
    }

    fn alice() -> User {
        User::new_local(
            UserId::new(),
            Username::new("alice").unwrap(),
            Email::new("alice@ex.com").unwrap(),
            PasswordHash("h".into()),
        )
    }

    fn local_thought(author_id: UserId) -> Thought {
        Thought::new_local(
            ThoughtId::new(),
            author_id,
            Content::new_local("hello").unwrap(),
            None,
            Visibility::Public,
            None,
            false,
        )
    }

    fn svc(store: &TestStore, spy: Arc<SpyPort>) -> FederationEventService {
        let ap_repo = TestApRepo::new(store.clone());
        FederationEventService {
            thoughts: Arc::new(store.clone()),
            users: Arc::new(store.clone()),
            ap: spy,
            base_url: "https://example.com".to_string(),
            ap_repo: Arc::new(ap_repo),
        }
    }

    fn svc_with_ap(store: &TestStore, ap_repo: TestApRepo, spy: Arc<SpyPort>) -> FederationEventService {
        FederationEventService {
            thoughts: Arc::new(store.clone()),
            users: Arc::new(store.clone()),
            ap: spy,
            base_url: "https://example.com".to_string(),
            ap_repo: Arc::new(ap_repo),
        }
    }

    #[tokio::test]
    async fn thought_created_broadcasts_create() {
        let store = TestStore::default();
        let alice = alice();
        let thought = local_thought(alice.id.clone());
        store.users.lock().unwrap().push(alice.clone());
        store.thoughts.lock().unwrap().push(thought.clone());

        let spy = Arc::new(SpyPort::default());
        svc(&store, spy.clone())
            .process(&DomainEvent::ThoughtCreated {
                thought_id: thought.id.clone(),
                user_id: alice.id.clone(),
                in_reply_to_id: None,
            })
            .await
            .unwrap();

        assert_eq!(spy.created.lock().unwrap().len(), 1);
        assert_eq!(spy.created.lock().unwrap()[0], thought.id);
    }

    #[tokio::test]
    async fn remote_thought_created_does_not_broadcast() {
        let store = TestStore::default();
        let alice = alice();
        // Remote thought: local = false
        let mut thought = local_thought(alice.id.clone());
        thought.local = false;
        store.users.lock().unwrap().push(alice.clone());
        store.thoughts.lock().unwrap().push(thought.clone());

        let spy = Arc::new(SpyPort::default());
        svc(&store, spy.clone())
            .process(&DomainEvent::ThoughtCreated {
                thought_id: thought.id.clone(),
                user_id: alice.id.clone(),
                in_reply_to_id: None,
            })
            .await
            .unwrap();

        assert!(spy.created.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn thought_deleted_broadcasts_delete_with_constructed_ap_id() {
        let store = TestStore::default();
        let alice = alice();
        let tid = ThoughtId::new();

        let spy = Arc::new(SpyPort::default());
        svc(&store, spy.clone())
            .process(&DomainEvent::ThoughtDeleted {
                thought_id: tid.clone(),
                user_id: alice.id.clone(),
            })
            .await
            .unwrap();

        let deleted = spy.deleted.lock().unwrap();
        assert_eq!(deleted.len(), 1);
        assert_eq!(deleted[0], format!("https://example.com/thoughts/{}", tid));
    }

    #[tokio::test]
    async fn thought_updated_broadcasts_update() {
        let store = TestStore::default();
        let alice = alice();
        let thought = local_thought(alice.id.clone());
        store.users.lock().unwrap().push(alice.clone());
        store.thoughts.lock().unwrap().push(thought.clone());

        let spy = Arc::new(SpyPort::default());
        svc(&store, spy.clone())
            .process(&DomainEvent::ThoughtUpdated {
                thought_id: thought.id.clone(),
                user_id: alice.id.clone(),
            })
            .await
            .unwrap();

        assert_eq!(spy.updated.lock().unwrap().len(), 1);
        assert_eq!(spy.updated.lock().unwrap()[0], thought.id);
    }

    #[tokio::test]
    async fn boost_of_local_thought_announces_constructed_url() {
        let store = TestStore::default();
        let alice = alice();
        let thought = local_thought(alice.id.clone()); // ap_id = None
        store.users.lock().unwrap().push(alice.clone());
        store.thoughts.lock().unwrap().push(thought.clone());

        let spy = Arc::new(SpyPort::default());
        svc(&store, spy.clone())
            .process(&DomainEvent::BoostAdded {
                boost_id: BoostId::new(),
                user_id: alice.id.clone(),
                thought_id: thought.id.clone(),
            })
            .await
            .unwrap();

        let announced = spy.announced.lock().unwrap();
        assert_eq!(announced.len(), 1);
        assert_eq!(
            announced[0],
            format!("https://example.com/thoughts/{}", thought.id)
        );
    }

    #[tokio::test]
    async fn boost_of_remote_thought_announces_remote_ap_id() {
        let store = TestStore::default();
        let alice = alice();
        let mut thought = local_thought(alice.id.clone());
        thought.local = false;
        let ap_repo = TestApRepo::new(store.clone());
        ap_repo.inner.thought_ap_ids.lock().unwrap().insert(
            thought.id.clone(),
            "https://mastodon.social/users/bob/statuses/123".into(),
        );
        store.users.lock().unwrap().push(alice.clone());
        store.thoughts.lock().unwrap().push(thought.clone());

        let spy = Arc::new(SpyPort::default());
        svc_with_ap(&store, ap_repo, spy.clone())
            .process(&DomainEvent::BoostAdded {
                boost_id: BoostId::new(),
                user_id: alice.id.clone(),
                thought_id: thought.id.clone(),
            })
            .await
            .unwrap();

        let announced = spy.announced.lock().unwrap();
        assert_eq!(
            announced[0],
            "https://mastodon.social/users/bob/statuses/123"
        );
    }

    #[tokio::test]
    async fn direct_thought_created_does_not_broadcast() {
        let store = TestStore::default();
        let alice = alice();
        let thought = Thought::new_local(
            ThoughtId::new(),
            alice.id.clone(),
            Content::new_local("private").unwrap(),
            None,
            Visibility::Direct,
            None,
            false,
        );
        store.users.lock().unwrap().push(alice.clone());
        store.thoughts.lock().unwrap().push(thought.clone());

        let spy = Arc::new(SpyPort::default());
        svc(&store, spy.clone())
            .process(&DomainEvent::ThoughtCreated {
                thought_id: thought.id.clone(),
                user_id: alice.id.clone(),
                in_reply_to_id: None,
            })
            .await
            .unwrap();

        assert!(spy.created.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn followers_only_thought_does_not_broadcast_publicly() {
        let store = TestStore::default();
        let alice = alice();
        let thought = Thought::new_local(
            ThoughtId::new(),
            alice.id.clone(),
            Content::new_local("for followers").unwrap(),
            None,
            Visibility::Followers,
            None,
            false,
        );
        store.users.lock().unwrap().push(alice.clone());
        store.thoughts.lock().unwrap().push(thought.clone());

        let spy = Arc::new(SpyPort::default());
        svc(&store, spy.clone())
            .process(&DomainEvent::ThoughtCreated {
                thought_id: thought.id.clone(),
                user_id: alice.id.clone(),
                in_reply_to_id: None,
            })
            .await
            .unwrap();

        assert!(spy.created.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn unrelated_events_are_noop() {
        let store = TestStore::default();
        let spy = Arc::new(SpyPort::default());
        let svc = svc(&store, spy.clone());

        svc.process(&DomainEvent::UserBlocked {
            blocker_id: UserId::new(),
            blocked_id: UserId::new(),
        })
        .await
        .unwrap();

        assert!(spy.created.lock().unwrap().is_empty());
        assert!(spy.deleted.lock().unwrap().is_empty());
        assert!(spy.updated.lock().unwrap().is_empty());
        assert!(spy.announced.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn thought_created_does_not_broadcast_if_user_missing() {
        let store = TestStore::default();
        let alice = alice();
        let thought = local_thought(alice.id.clone());
        // Don't push alice into users — simulates user deleted before handler runs
        store.thoughts.lock().unwrap().push(thought.clone());

        let spy = Arc::new(SpyPort::default());
        svc(&store, spy.clone())
            .process(&DomainEvent::ThoughtCreated {
                thought_id: thought.id.clone(),
                user_id: alice.id.clone(),
                in_reply_to_id: None,
            })
            .await
            .unwrap();

        assert!(spy.created.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn boost_removed_sends_undo_announce_for_local_thought() {
        let store = TestStore::default();
        let alice = alice();
        let thought = local_thought(alice.id.clone()); // ap_id = None → constructed URL
        store.thoughts.lock().unwrap().push(thought.clone());

        let spy = Arc::new(SpyPort::default());
        svc(&store, spy.clone())
            .process(&DomainEvent::BoostRemoved {
                user_id: alice.id.clone(),
                thought_id: thought.id.clone(),
            })
            .await
            .unwrap();

        let undo_announced = spy.undo_announced.lock().unwrap();
        assert_eq!(undo_announced.len(), 1);
        assert_eq!(
            undo_announced[0],
            format!("https://example.com/thoughts/{}", thought.id)
        );
    }

    #[tokio::test]
    async fn boost_removed_sends_undo_announce_for_remote_thought() {
        let store = TestStore::default();
        let alice = alice();
        let mut thought = local_thought(alice.id.clone());
        thought.local = false;
        let ap_repo = TestApRepo::new(store.clone());
        ap_repo.inner.thought_ap_ids.lock().unwrap().insert(
            thought.id.clone(),
            "https://mastodon.social/users/bob/statuses/456".into(),
        );
        store.thoughts.lock().unwrap().push(thought.clone());

        let spy = Arc::new(SpyPort::default());
        svc_with_ap(&store, ap_repo, spy.clone())
            .process(&DomainEvent::BoostRemoved {
                user_id: alice.id.clone(),
                thought_id: thought.id.clone(),
            })
            .await
            .unwrap();

        let undo_announced = spy.undo_announced.lock().unwrap();
        assert_eq!(undo_announced.len(), 1);
        assert_eq!(
            undo_announced[0],
            "https://mastodon.social/users/bob/statuses/456"
        );
    }

    #[tokio::test]
    async fn boost_removed_does_not_broadcast_if_thought_missing() {
        let store = TestStore::default();
        let alice = alice();
        let spy = Arc::new(SpyPort::default());
        svc(&store, spy.clone())
            .process(&DomainEvent::BoostRemoved {
                user_id: alice.id.clone(),
                thought_id: ThoughtId::new(), // doesn't exist in store
            })
            .await
            .unwrap();
        assert!(spy.undo_announced.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn thought_updated_does_not_broadcast_if_user_missing() {
        let store = TestStore::default();
        let alice = alice();
        let thought = local_thought(alice.id.clone());
        // Don't push alice into users
        store.thoughts.lock().unwrap().push(thought.clone());

        let spy = Arc::new(SpyPort::default());
        svc(&store, spy.clone())
            .process(&DomainEvent::ThoughtUpdated {
                thought_id: thought.id.clone(),
                user_id: alice.id.clone(),
            })
            .await
            .unwrap();

        assert!(spy.updated.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn like_added_local_user_remote_thought_broadcasts_like() {
        let store = TestStore::default();

        let mut author = User::new_local(
            UserId::new(),
            Username::new("remote_author").unwrap(),
            Email::new("r@remote.example").unwrap(),
            PasswordHash("h".into()),
        );
        author.local = false;
        let thought = local_thought(author.id.clone());
        let liker = alice();

        store.users.lock().unwrap().push(author.clone());
        store.users.lock().unwrap().push(liker.clone());
        store.thoughts.lock().unwrap().push(thought.clone());

        let ap_repo = TestApRepo::new(store.clone());
        ap_repo.actor_ap_urls.lock().unwrap().insert(
            author.id.clone(),
            ActorApUrls {
                ap_id: "https://mastodon.social/users/author".into(),
                inbox_url: "https://mastodon.social/users/author/inbox".into(),
            },
        );
        ap_repo.inner.thought_ap_ids.lock().unwrap().insert(
            thought.id.clone(),
            "https://mastodon.social/posts/123".into(),
        );

        let spy = Arc::new(SpyPort::default());
        svc_with_ap(&store, ap_repo, spy.clone())
            .process(&DomainEvent::LikeAdded {
                like_id: LikeId::new(),
                user_id: liker.id,
                thought_id: thought.id,
            })
            .await
            .unwrap();

        assert_eq!(spy.liked.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn like_added_remote_user_skips_broadcast() {
        let store = TestStore::default();

        let author = alice();
        let thought = local_thought(author.id.clone()); // local thought — no ap_id

        let mut remote_liker = User::new_local(
            UserId::new(),
            Username::new("bob").unwrap(),
            Email::new("bob@remote").unwrap(),
            PasswordHash("h".into()),
        );
        remote_liker.local = false;

        store.users.lock().unwrap().push(author);
        store.users.lock().unwrap().push(remote_liker.clone());
        store.thoughts.lock().unwrap().push(thought.clone());

        let spy = Arc::new(SpyPort::default());
        svc(&store, spy.clone())
            .process(&DomainEvent::LikeAdded {
                like_id: LikeId::new(),
                user_id: remote_liker.id,
                thought_id: thought.id,
            })
            .await
            .unwrap();

        assert!(spy.liked.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn boost_added_remote_user_skips_broadcast() {
        let store = TestStore::default();

        let author = alice();
        let thought = local_thought(author.id.clone());

        let mut remote_booster = User::new_local(
            UserId::new(),
            Username::new("bob").unwrap(),
            Email::new("bob@remote").unwrap(),
            PasswordHash("h".into()),
        );
        remote_booster.local = false;

        store.users.lock().unwrap().push(author);
        store.users.lock().unwrap().push(remote_booster.clone());
        store.thoughts.lock().unwrap().push(thought.clone());

        let spy = Arc::new(SpyPort::default());
        svc(&store, spy.clone())
            .process(&DomainEvent::BoostAdded {
                boost_id: BoostId::new(),
                user_id: remote_booster.id,
                thought_id: thought.id,
            })
            .await
            .unwrap();

        assert!(spy.announced.lock().unwrap().is_empty());
    }
}
