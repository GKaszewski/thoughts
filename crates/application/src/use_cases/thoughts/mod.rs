use domain::{
    errors::DomainError,
    events::DomainEvent,
    models::{
        feed::{EngagementStats, FeedEntry},
        thought::{Thought, Visibility},
    },
    ports::{
        EngagementRepository, EventPublisher, OutboxWriter, TagRepository, ThoughtRepository,
        UserReader,
    },
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
    let mut map = engagement
        .get_for_thoughts(std::slice::from_ref(id), viewer)
        .await?;
    let (stats, viewer_ctx) = map.remove(id).unwrap_or((
        EngagementStats {
            like_count: 0,
            boost_count: 0,
            reply_count: 0,
        },
        None,
    ));
    Ok(FeedEntry {
        thought,
        author,
        stats,
        viewer: viewer_ctx,
    })
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
        let (stats, viewer_ctx) = engagement_map.remove(&thought.id).unwrap_or((
            EngagementStats {
                like_count: 0,
                boost_count: 0,
                reply_count: 0,
            },
            None,
        ));
        entries.push(FeedEntry {
            thought,
            author,
            stats,
            viewer: viewer_ctx,
        });
    }
    Ok(entries)
}

#[cfg(test)]
mod tests;
