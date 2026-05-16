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
mod tests;
