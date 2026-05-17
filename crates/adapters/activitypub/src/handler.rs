use anyhow::{anyhow, Result};
use async_trait::async_trait;

const USERS_PATH_PREFIX: &str = "/users/";
const THOUGHTS_PATH_PREFIX: &str = "/thoughts/";
use chrono::{DateTime, Utc};
use std::sync::Arc;
use url::Url;

use crate::note::{ThoughtNote, ThoughtNoteInput};
use crate::urls::ThoughtsUrls;
use activitypub_base::{AcceptNoteInput, ActivityPubRepository, ApObjectHandler};
use domain::ports::{EventPublisher, TagRepository};
use domain::value_objects::UserId;

pub struct ThoughtsObjectHandler {
    repo: Arc<dyn ActivityPubRepository>,
    urls: ThoughtsUrls,
    event_publisher: Option<Arc<dyn EventPublisher>>,
    tag_repo: Arc<dyn TagRepository>,
}

impl ThoughtsObjectHandler {
    pub fn new(
        repo: Arc<dyn ActivityPubRepository>,
        base_url: &str,
        event_publisher: Option<Arc<dyn EventPublisher>>,
        tag_repo: Arc<dyn TagRepository>,
    ) -> Self {
        Self {
            repo,
            urls: ThoughtsUrls::new(base_url),
            event_publisher,
            tag_repo,
        }
    }
}

#[async_trait]
impl ApObjectHandler for ThoughtsObjectHandler {
    async fn get_local_objects_for_user(
        &self,
        user_id: uuid::Uuid,
    ) -> Result<Vec<(Url, serde_json::Value)>> {
        let uid = UserId::from_uuid(user_id);
        let entries = self
            .repo
            .outbox_entries_for_actor(&uid)
            .await
            .map_err(|e| anyhow!("{e}"))?;
        entries
            .into_iter()
            .map(|e| {
                let note_url = self.urls.thought_url(e.thought.id.as_uuid());
                let actor_url = self.urls.user_url(e.author_username.as_str());
                let followers = self.urls.user_followers(e.author_username.as_str());
                let in_reply_to = e
                    .thought
                    .in_reply_to_id
                    .map(|id| self.urls.thought_url(id.as_uuid()));
                let note = ThoughtNote::new_public(ThoughtNoteInput {
                    id: note_url.clone(),
                    actor_url,
                    content: e.thought.content.as_str().to_owned(),
                    published: e.thought.created_at,
                    in_reply_to,
                    sensitive: e.thought.sensitive,
                    summary: e.thought.content_warning,
                    followers_url: followers,
                });
                Ok((note_url, serde_json::to_value(&note)?))
            })
            .collect()
    }

    async fn get_local_objects_page(
        &self,
        user_id: uuid::Uuid,
        before: Option<DateTime<Utc>>,
        limit: usize,
    ) -> Result<Vec<(Url, serde_json::Value, DateTime<Utc>)>> {
        let uid = UserId::from_uuid(user_id);
        let entries = self
            .repo
            .outbox_page_for_actor(&uid, before, limit)
            .await
            .map_err(|e| anyhow!("{e}"))?;
        entries
            .into_iter()
            .map(|e| {
                let created_at = e.thought.created_at;
                let note_url = self.urls.thought_url(e.thought.id.as_uuid());
                let actor_url = self.urls.user_url(e.author_username.as_str());
                let followers = self.urls.user_followers(e.author_username.as_str());
                let in_reply_to = e
                    .thought
                    .in_reply_to_id
                    .map(|id| self.urls.thought_url(id.as_uuid()));
                let note = ThoughtNote::new_public(ThoughtNoteInput {
                    id: note_url.clone(),
                    actor_url,
                    content: e.thought.content.as_str().to_owned(),
                    published: created_at,
                    in_reply_to,
                    sensitive: e.thought.sensitive,
                    summary: e.thought.content_warning,
                    followers_url: followers,
                });
                Ok((note_url, serde_json::to_value(&note)?, created_at))
            })
            .collect()
    }

    async fn on_create(
        &self,
        ap_id: &Url,
        actor_url: &Url,
        object: serde_json::Value,
    ) -> Result<()> {
        let note: ThoughtNote = serde_json::from_value(object)?;
        let author_id = self
            .repo
            .intern_remote_actor(actor_url.as_str())
            .await
            .map_err(|e| anyhow!("{e}"))?;

        // Derive visibility from AP addressing conventions.
        let as_public = "https://www.w3.org/ns/activitystreams#Public";
        let in_to = note.to.iter().any(|s| s == as_public);
        let in_cc = note.cc.iter().any(|s| s == as_public);
        let has_followers = note.to.iter().any(|s| s.ends_with("/followers"))
            || note.cc.iter().any(|s| s.ends_with("/followers"));

        let visibility = if in_to {
            "public"
        } else if in_cc {
            "unlisted"
        } else if has_followers {
            "followers"
        } else {
            "direct"
        };

        let thought_id = self
            .repo
            .accept_note(AcceptNoteInput {
                ap_id: ap_id.as_str(),
                author_id: &author_id,
                content: &note.content,
                published: note.published,
                sensitive: note.sensitive,
                content_warning: note.summary,
                visibility,
                in_reply_to: note.in_reply_to.as_ref().map(|u| u.as_str()),
            })
            .await
            .map_err(|e| anyhow!("{e}"))?;

        // Extract and index hashtags from the AP tag array.
        let hashtag_names: Vec<String> = note
            .tag
            .iter()
            .filter(|t| t.get("type").and_then(|v| v.as_str()) == Some("Hashtag"))
            .filter_map(|t| t.get("name").and_then(|v| v.as_str()))
            .map(|name| name.trim_start_matches('#').to_lowercase())
            .filter(|name| !name.is_empty())
            .collect();

        for name in hashtag_names {
            if let Ok(tag) = self.tag_repo.find_or_create(&name).await {
                let _ = self.tag_repo.attach_to_thought(&thought_id, tag.id).await;
            }
        }

        // Fire mention notifications for local @mentions in the note's tag array.
        let base_url = url::Url::parse(&self.urls.base_url)
            .ok()
            .and_then(|u| u.host_str().map(|h| h.to_string()))
            .unwrap_or_default();

        for tag in &note.tag {
            if tag.get("type").and_then(|t| t.as_str()) != Some("Mention") {
                continue;
            }
            let href = match tag.get("href").and_then(|h| h.as_str()) {
                Some(h) => h,
                None => continue,
            };
            let href_url = match url::Url::parse(href) {
                Ok(u) => u,
                Err(_) => continue,
            };
            if href_url.host_str().unwrap_or("") != base_url {
                continue;
            }
            let user_uuid = href_url
                .path()
                .strip_prefix(USERS_PATH_PREFIX)
                .and_then(|s| s.split('/').next())
                .and_then(|s| uuid::Uuid::parse_str(s).ok());
            if let Some(uuid) = user_uuid {
                self.on_mention(ap_id, uuid, actor_url)
                    .await
                    .unwrap_or_else(|e| {
                        tracing::warn!(error = %e, "failed to process mention notification");
                    });
            }
        }

        Ok(())
    }

    async fn on_update(
        &self,
        ap_id: &Url,
        _actor_url: &Url,
        object: serde_json::Value,
    ) -> Result<()> {
        let note: ThoughtNote = serde_json::from_value(object)?;
        self.repo
            .apply_note_update(ap_id.as_str(), &note.content)
            .await
            .map_err(|e| anyhow!("{e}"))
    }

    async fn on_delete(&self, ap_id: &Url, _actor_url: &Url) -> Result<()> {
        self.repo
            .retract_note(ap_id.as_str())
            .await
            .map_err(|e| anyhow!("{e}"))
    }

    async fn on_actor_removed(&self, actor_url: &Url) -> Result<()> {
        self.repo
            .retract_actor_notes(actor_url.as_str())
            .await
            .map_err(|e| anyhow!("{e}"))
    }

    async fn on_like(&self, object_url: &Url, actor_url: &Url) -> Result<()> {
        let thought_uuid = object_url
            .path()
            .strip_prefix(THOUGHTS_PATH_PREFIX)
            .and_then(|s| s.split('/').next())
            .and_then(|s| uuid::Uuid::parse_str(s).ok());

        let thought_uuid = match thought_uuid {
            Some(u) => u,
            None => {
                tracing::debug!(object = %object_url, "on_like: not a local thought URL, skipping");
                return Ok(());
            }
        };

        let actor_user_id = self
            .repo
            .find_remote_actor_id(actor_url.as_str())
            .await
            .map_err(|e| anyhow!("{e}"))?;

        let actor_user_id = match actor_user_id {
            Some(id) => id,
            None => {
                tracing::debug!(actor = %actor_url, "on_like: remote actor not interned, skipping notification");
                return Ok(());
            }
        };

        if let Some(ep) = &self.event_publisher {
            let thought_id = domain::value_objects::ThoughtId::from_uuid(thought_uuid);
            let like_id = domain::value_objects::LikeId::new();
            ep.publish(&domain::events::DomainEvent::LikeAdded {
                like_id,
                user_id: actor_user_id,
                thought_id,
            })
            .await
            .map_err(|e| anyhow!("{e}"))?;
        }

        Ok(())
    }

    async fn on_unlike(&self, object_url: &url::Url, actor_url: &url::Url) -> anyhow::Result<()> {
        let thought_uuid = object_url
            .path()
            .strip_prefix(THOUGHTS_PATH_PREFIX)
            .and_then(|s| s.split('/').next())
            .and_then(|s| uuid::Uuid::parse_str(s).ok());

        let thought_uuid = match thought_uuid {
            Some(u) => u,
            None => {
                tracing::debug!(object = %object_url, "on_unlike: not a local thought URL, skipping");
                return Ok(());
            }
        };

        let actor_user_id = self
            .repo
            .find_remote_actor_id(actor_url.as_str())
            .await
            .map_err(|e| anyhow!("{e}"))?;

        let actor_user_id = match actor_user_id {
            Some(id) => id,
            None => {
                tracing::debug!(actor = %actor_url, "on_unlike: remote actor not interned, skipping");
                return Ok(());
            }
        };

        if let Some(ep) = &self.event_publisher {
            ep.publish(&domain::events::DomainEvent::LikeRemoved {
                user_id: actor_user_id,
                thought_id: domain::value_objects::ThoughtId::from_uuid(thought_uuid),
            })
            .await
            .map_err(|e| anyhow!("{e}"))?;
        }

        Ok(())
    }

    async fn on_mention(
        &self,
        thought_ap_id: &url::Url,
        mentioned_user_uuid: uuid::Uuid,
        actor_url: &url::Url,
    ) -> anyhow::Result<()> {
        let author_user_id = match self
            .repo
            .find_remote_actor_id(actor_url.as_str())
            .await
            .map_err(|e| anyhow!("{e}"))?
        {
            Some(id) => id,
            None => return Ok(()),
        };

        let thought_uuid = thought_ap_id
            .path()
            .strip_prefix(THOUGHTS_PATH_PREFIX)
            .and_then(|s| s.split('/').next())
            .and_then(|s| uuid::Uuid::parse_str(s).ok());

        let thought_uuid = match thought_uuid {
            Some(u) => u,
            None => return Ok(()),
        };

        if let Some(ep) = &self.event_publisher {
            ep.publish(&domain::events::DomainEvent::MentionReceived {
                thought_id: domain::value_objects::ThoughtId::from_uuid(thought_uuid),
                mentioned_user_id: domain::value_objects::UserId::from_uuid(mentioned_user_uuid),
                author_user_id,
            })
            .await
            .map_err(|e| anyhow!("{e}"))?;
        }

        Ok(())
    }

    async fn on_announce_received(&self, object_url: &Url, actor_url: &Url) -> Result<()> {
        let thought_uuid = object_url
            .path()
            .strip_prefix(THOUGHTS_PATH_PREFIX)
            .and_then(|s| s.split('/').next())
            .and_then(|s| uuid::Uuid::parse_str(s).ok());

        let thought_uuid = match thought_uuid {
            Some(u) => u,
            None => return Ok(()),
        };

        let actor_user_id = self
            .repo
            .find_remote_actor_id(actor_url.as_str())
            .await
            .map_err(|e| anyhow!("{e}"))?;

        let actor_user_id = match actor_user_id {
            Some(id) => id,
            None => return Ok(()),
        };

        if let Some(ep) = &self.event_publisher {
            let thought_id = domain::value_objects::ThoughtId::from_uuid(thought_uuid);
            let boost_id = domain::value_objects::BoostId::new();
            ep.publish(&domain::events::DomainEvent::BoostAdded {
                boost_id,
                user_id: actor_user_id,
                thought_id,
            })
            .await
            .map_err(|e| anyhow!("{e}"))?;
        }

        Ok(())
    }

    async fn count_local_posts(&self) -> Result<u64> {
        self.repo
            .count_local_notes()
            .await
            .map_err(|e| anyhow!("{e}"))
    }
}
