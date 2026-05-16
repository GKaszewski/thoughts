use domain::{
    errors::DomainError,
    events::DomainEvent,
    models::{
        feed::{EngagementStats, FeedEntry},
        thought::{Thought, Visibility},
    },
    ports::{EngagementRepository, EventPublisher, OutboxWriter, TagRepository, ThoughtRepository, UserReader},
    value_objects::{Content, ThoughtId, UserId},
};

fn require_owner(thought: &Thought, user_id: &UserId) -> Result<(), DomainError> {
    if thought.user_id != *user_id {
        return Err(DomainError::NotFound);
    }
    Ok(())
}

pub struct CreateThoughtInput {
    pub user_id: UserId,
    pub content: String,
    pub in_reply_to_id: Option<ThoughtId>,
    pub visibility: Option<String>,
    pub content_warning: Option<String>,
    pub sensitive: bool,
}
pub struct CreateThoughtOutput {
    pub thought: Thought,
}

pub async fn create_thought(
    thoughts: &dyn ThoughtRepository,
    _users: &dyn UserReader,
    tags: &dyn TagRepository,
    _events: &dyn EventPublisher,
    outbox: &dyn OutboxWriter,
    input: CreateThoughtInput,
) -> Result<CreateThoughtOutput, DomainError> {
    let content = Content::new_local(input.content)?;
    let visibility = match input.visibility.as_deref() {
        Some("followers") => Visibility::Followers,
        Some("unlisted") => Visibility::Unlisted,
        Some("direct") => Visibility::Direct,
        _ => Visibility::Public,
    };
    let thought = Thought::new_local(
        ThoughtId::new(),
        input.user_id,
        content.clone(),
        input.in_reply_to_id.clone(),
        visibility,
        input.content_warning,
        input.sensitive,
    );
    thoughts.save(&thought).await?;

    // Extract and attach hashtags from content.
    for h in domain::hashtag::extract(content.as_str()) {
        if let Ok(tag) = tags.find_or_create(&h.normalized).await {
            let _ = tags.attach_to_thought(&thought.id, tag.id).await;
        }
    }

    outbox
        .append(&DomainEvent::ThoughtCreated {
            thought_id: thought.id.clone(),
            user_id: thought.user_id.clone(),
            in_reply_to_id: input.in_reply_to_id,
        })
        .await?;
    Ok(CreateThoughtOutput { thought })
}

pub async fn delete_thought(
    thoughts: &dyn ThoughtRepository,
    _events: &dyn EventPublisher,
    outbox: &dyn OutboxWriter,
    id: &ThoughtId,
    user_id: &UserId,
) -> Result<(), DomainError> {
    let thought = thoughts
        .find_by_id(id)
        .await?
        .ok_or(DomainError::NotFound)?;
    require_owner(&thought, user_id)?;
    thoughts.delete(id, user_id).await?;
    outbox
        .append(&DomainEvent::ThoughtDeleted {
            thought_id: id.clone(),
            user_id: user_id.clone(),
        })
        .await?;
    Ok(())
}

pub async fn edit_thought(
    thoughts: &dyn ThoughtRepository,
    events: &dyn EventPublisher,
    id: &ThoughtId,
    user_id: &UserId,
    new_content: String,
) -> Result<(), DomainError> {
    let thought = thoughts
        .find_by_id(id)
        .await?
        .ok_or(DomainError::NotFound)?;
    require_owner(&thought, user_id)?;
    let content = Content::new_local(new_content)?;
    thoughts.update_content(id, &content).await?;
    events
        .publish(&DomainEvent::ThoughtUpdated {
            thought_id: id.clone(),
            user_id: user_id.clone(),
        })
        .await?;
    Ok(())
}

/// Fetches a single thought enriched with author + real engagement stats.
pub async fn get_thought_view(
    thoughts: &dyn ThoughtRepository,
    users: &dyn UserReader,
    engagement: &dyn EngagementRepository,
    id: &ThoughtId,
    viewer: Option<&UserId>,
) -> Result<FeedEntry, DomainError> {
    let thought = thoughts
        .find_by_id(id)
        .await?
        .ok_or(DomainError::NotFound)?;
    let author = users
        .find_by_id(&thought.user_id)
        .await?
        .ok_or(DomainError::NotFound)?;
    let mut map = engagement.get_for_thoughts(&[id.clone()], viewer).await?;
    let (stats, viewer_ctx) = map.remove(id).unwrap_or(
        (EngagementStats { like_count: 0, boost_count: 0, reply_count: 0 }, None)
    );
    Ok(FeedEntry { thought, author, stats, viewer: viewer_ctx })
}

/// Fetches a thread (root + replies) enriched with authors + real engagement stats.
/// Batches all DB lookups — one query per resource type regardless of thread length.
pub async fn get_thread_views(
    thoughts: &dyn ThoughtRepository,
    users: &dyn UserReader,
    engagement: &dyn EngagementRepository,
    root_id: &ThoughtId,
    viewer: Option<&UserId>,
) -> Result<Vec<FeedEntry>, DomainError> {
    let thread = thoughts.get_thread(root_id).await?;
    if thread.is_empty() {
        return Ok(vec![]);
    }

    let thought_ids: Vec<ThoughtId> = thread.iter().map(|t| t.id.clone()).collect();
    let user_ids: Vec<UserId> = thread.iter().map(|t| t.user_id.clone()).collect();

    let (authors_map, engagement_map) = tokio::join!(
        users.find_by_ids(&user_ids),
        engagement.get_for_thoughts(&thought_ids, viewer),
    );
    let authors_map = authors_map?;
    let mut engagement_map = engagement_map?;

    let mut entries = Vec::with_capacity(thread.len());
    for thought in thread {
        let author = authors_map
            .get(&thought.user_id)
            .cloned()
            .ok_or(DomainError::NotFound)?;
        let (stats, viewer_ctx) = engagement_map.remove(&thought.id).unwrap_or(
            (EngagementStats { like_count: 0, boost_count: 0, reply_count: 0 }, None)
        );
        entries.push(FeedEntry { thought, author, stats, viewer: viewer_ctx });
    }
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::{
        models::user::User,
        testing::{NoOpEventPublisher, NoOpOutboxWriter, TestOutbox, TestStore},
        value_objects::*,
    };

    fn user() -> User {
        User::new_local(
            UserId::new(),
            Username::new("alice").unwrap(),
            Email::new("alice@ex.com").unwrap(),
            PasswordHash("h".into()),
        )
    }

    fn input(uid: UserId) -> CreateThoughtInput {
        CreateThoughtInput {
            user_id: uid,
            content: "hello".into(),
            in_reply_to_id: None,
            visibility: None,
            content_warning: None,
            sensitive: false,
        }
    }

    #[tokio::test]
    async fn create_thought_saves_and_stages_outbox_event() {
        let store = TestStore::default();
        let outbox = TestOutbox::default();
        let u = user();
        store.users.lock().unwrap().push(u.clone());
        let out = create_thought(&store, &store, &store, &NoOpEventPublisher, &outbox, input(u.id.clone()))
            .await
            .unwrap();
        assert_eq!(out.thought.content.as_str(), "hello");
        let staged = outbox.staged();
        assert_eq!(staged.len(), 1);
        assert!(matches!(staged[0], DomainEvent::ThoughtCreated { .. }));
    }

    #[tokio::test]
    async fn delete_thought_stages_outbox_event() {
        let store = TestStore::default();
        let outbox = TestOutbox::default();
        let u = user();
        store.users.lock().unwrap().push(u.clone());
        let out = create_thought(
            &store,
            &store,
            &store,
            &NoOpEventPublisher,
            &NoOpOutboxWriter,
            input(u.id.clone()),
        )
        .await
        .unwrap();
        let tid = out.thought.id.clone();

        delete_thought(&store, &NoOpEventPublisher, &outbox, &tid, &u.id)
            .await
            .unwrap();

        let staged = outbox.staged();
        assert_eq!(staged.len(), 1);
        assert!(matches!(&staged[0], DomainEvent::ThoughtDeleted { thought_id, .. } if *thought_id == tid));
    }

    #[tokio::test]
    async fn delete_own_thought_succeeds() {
        let store = TestStore::default();
        let u = user();
        store.users.lock().unwrap().push(u.clone());
        let out = create_thought(
            &store,
            &store,
            &store,
            &NoOpEventPublisher,
            &NoOpOutboxWriter,
            input(u.id.clone()),
        )
        .await
        .unwrap();
        delete_thought(&store, &NoOpEventPublisher, &NoOpOutboxWriter, &out.thought.id, &u.id)
            .await
            .unwrap();
        assert!(store.thoughts.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn delete_other_thought_returns_not_found() {
        let store = TestStore::default();
        let alice = user();
        let bob = User::new_local(
            UserId::new(),
            Username::new("bob").unwrap(),
            Email::new("bob@ex.com").unwrap(),
            PasswordHash("h".into()),
        );
        store
            .users
            .lock()
            .unwrap()
            .extend([alice.clone(), bob.clone()]);
        let out = create_thought(
            &store,
            &store,
            &store,
            &NoOpEventPublisher,
            &NoOpOutboxWriter,
            input(alice.id.clone()),
        )
        .await
        .unwrap();
        let err = delete_thought(&store, &NoOpEventPublisher, &NoOpOutboxWriter, &out.thought.id, &bob.id)
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::NotFound));
    }

    #[tokio::test]
    async fn edit_thought_changes_content_and_emits_event() {
        let store = TestStore::default();
        let alice = user();
        store.users.lock().unwrap().push(alice.clone());
        let out = create_thought(&store, &store, &store, &NoOpEventPublisher, &NoOpOutboxWriter, input(alice.id.clone()))
            .await
            .unwrap();
        let tid = out.thought.id.clone();

        edit_thought(&store, &store, &tid, &alice.id, "updated".to_string())
            .await
            .unwrap();

        let saved = store
            .thoughts
            .lock()
            .unwrap()
            .iter()
            .find(|t| t.id == tid)
            .unwrap()
            .clone();
        assert_eq!(saved.content.as_str(), "updated");

        let events = store.events.lock().unwrap();
        assert!(events.iter().any(
            |e| matches!(e, DomainEvent::ThoughtUpdated { thought_id, .. } if thought_id == &tid)
        ));
    }

    #[tokio::test]
    async fn create_reply_sets_in_reply_to_id() {
        let store = TestStore::default();
        let alice = user();
        store.users.lock().unwrap().push(alice.clone());
        let original = create_thought(
            &store,
            &store,
            &store,
            &NoOpEventPublisher,
            &NoOpOutboxWriter,
            input(alice.id.clone()),
        )
        .await
        .unwrap()
        .thought;

        create_thought(
            &store,
            &store,
            &store,
            &NoOpEventPublisher,
            &NoOpOutboxWriter,
            CreateThoughtInput {
                user_id: alice.id.clone(),
                content: "reply".into(),
                in_reply_to_id: Some(original.id.clone()),
                visibility: None,
                content_warning: None,
                sensitive: false,
            },
        )
        .await
        .unwrap();

        let thoughts = store.thoughts.lock().unwrap();
        let reply = thoughts
            .iter()
            .find(|t| t.content.as_str() == "reply")
            .unwrap();
        assert_eq!(reply.in_reply_to_id, Some(original.id.clone()));
    }
}

#[cfg(test)]
mod enrichment_tests {
    use super::*;
    use domain::testing::TestStore;
    use domain::models::user::User;
    use domain::models::thought::{Thought, Visibility};
    use domain::value_objects::*;
    use domain::ports::{ThoughtRepository, UserWriter};

    fn make_user() -> User {
        User::new_local(
            UserId::new(),
            Username::new("alice").unwrap(),
            Email::new("a@a.com").unwrap(),
            PasswordHash("h".into()),
        )
    }

    fn make_thought(user_id: UserId) -> Thought {
        Thought::new_local(
            ThoughtId::new(),
            user_id,
            Content::new_local(String::from("hello")).unwrap(),
            None,
            Visibility::Public,
            None,
            false,
        )
    }

    #[tokio::test]
    async fn get_thought_view_returns_feed_entry() {
        let store = TestStore::default();
        let user = make_user();
        <TestStore as UserWriter>::save(&store, &user).await.unwrap();
        let thought = make_thought(user.id.clone());
        <TestStore as ThoughtRepository>::save(&store, &thought).await.unwrap();

        let entry = get_thought_view(&store, &store, &store, &thought.id, None)
            .await
            .unwrap();
        assert_eq!(entry.thought.id, thought.id);
        assert_eq!(entry.author.id, user.id);
        assert_eq!(entry.stats.like_count, 0);
        assert!(entry.viewer.is_none());
    }

    #[tokio::test]
    async fn get_thought_view_returns_not_found_for_missing_thought() {
        let store = TestStore::default();
        let err = get_thought_view(&store, &store, &store, &ThoughtId::new(), None)
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::NotFound));
    }

    #[tokio::test]
    async fn get_thread_views_batches_correctly() {
        let store = TestStore::default();
        let user = make_user();
        <TestStore as UserWriter>::save(&store, &user).await.unwrap();
        let root = make_thought(user.id.clone());
        <TestStore as ThoughtRepository>::save(&store, &root).await.unwrap();
        let reply = Thought::new_local(
            ThoughtId::new(),
            user.id.clone(),
            Content::new_local(String::from("reply")).unwrap(),
            Some(root.id.clone()),
            Visibility::Public,
            None,
            false,
        );
        <TestStore as ThoughtRepository>::save(&store, &reply).await.unwrap();

        let entries = get_thread_views(&store, &store, &store, &root.id, None)
            .await
            .unwrap();
        assert_eq!(entries.len(), 2);
    }
}
