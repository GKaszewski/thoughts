use std::sync::Arc;

use async_trait::async_trait;
use k_ap::ActivityPubService;

use domain::{
    errors::DomainError,
    models::remote_actor::RemoteActor as DomainRemoteActor,
    ports::{
        FederationFetchPort, FederationFollowPort, FederationFollowRequestPort,
        FederationLookupPort, FederationSchedulerPort, RemoteActorConnectionRepository,
    },
    value_objects::UserId,
};

const HTTP_FETCH_TIMEOUT_SECS: u64 = 30;
const BATCH_FETCH_SLEEP_MS: u64 = 100;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn content_to_html(text: &str) -> String {
    let escaped = text
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;");
    let paragraphs: Vec<&str> = escaped.split('\n').filter(|s| !s.is_empty()).collect();
    if paragraphs.is_empty() {
        format!("<p>{}</p>", escaped)
    } else {
        paragraphs
            .iter()
            .map(|p| format!("<p>{}</p>", p))
            .collect::<Vec<_>>()
            .join("")
    }
}

fn build_note_json(
    thought: &domain::models::thought::Thought,
    local_actor_ap_id: &str,
    local_actor_followers_url: &str,
    base_url: &str,
    in_reply_to_url: Option<&str>,
) -> serde_json::Value {
    let ap_id = format!("{}/thoughts/{}", base_url, thought.id);

    let (to, cc) = match thought.visibility {
        domain::models::thought::Visibility::Public => (
            vec![k_ap::AS_PUBLIC.to_string()],
            vec![local_actor_followers_url.to_string()],
        ),
        domain::models::thought::Visibility::Unlisted => (
            vec![local_actor_followers_url.to_string()],
            vec![k_ap::AS_PUBLIC.to_string()],
        ),
        domain::models::thought::Visibility::Followers => {
            (vec![local_actor_followers_url.to_string()], vec![])
        }
        domain::models::thought::Visibility::Direct => (vec![], vec![]),
    };

    let mut note = serde_json::json!({
        "type": "Note",
        "id": ap_id,
        "url": ap_id,
        "attributedTo": local_actor_ap_id,
        "content": content_to_html(thought.content.as_str()),
        "published": thought.created_at.to_rfc3339(),
        "to": to,
        "cc": cc,
        "sensitive": thought.sensitive,
    });
    if let Some(ref cw) = thought.content_warning {
        note["summary"] = serde_json::json!(cw);
    }
    if let Some(reply_url) = in_reply_to_url {
        note["inReplyTo"] = serde_json::json!(reply_url);
    }
    if let Some(updated_at) = thought.updated_at {
        note["updated"] = serde_json::json!(updated_at.to_rfc3339());
    }
    let hashtags = domain::hashtag::extract(thought.content.as_str());
    if !hashtags.is_empty() {
        let ap_tags: Vec<serde_json::Value> = hashtags
            .iter()
            .map(|h| {
                serde_json::json!({
                    "type": "Hashtag",
                    "name": h.ap_name,
                    "href": format!("{}/{}", base_url, h.url_slug),
                })
            })
            .collect();
        note["tag"] = serde_json::json!(ap_tags);
    }
    note
}

fn thought_to_ap_visibility(
    v: &domain::models::thought::Visibility,
) -> k_ap::ApVisibility {
    match v {
        domain::models::thought::Visibility::Public => k_ap::ApVisibility::Public,
        domain::models::thought::Visibility::Unlisted => k_ap::ApVisibility::Public,
        domain::models::thought::Visibility::Followers => k_ap::ApVisibility::FollowersOnly,
        domain::models::thought::Visibility::Direct => k_ap::ApVisibility::Private,
    }
}

fn k_ap_actor_to_domain(a: k_ap::RemoteActor) -> DomainRemoteActor {
    DomainRemoteActor {
        url: a.url,
        handle: a.handle,
        display_name: a.display_name,
        avatar_url: a.avatar_url,
        outbox_url: a.outbox_url,
        last_fetched_at: chrono::Utc::now(),
        bio: None,
        banner_url: None,
        also_known_as: None,
        followers_url: None,
        following_url: None,
        attachment: vec![],
    }
}

async fn resolve_actor_profiles_from_urls(
    urls: Vec<String>,
) -> Vec<domain::models::actor_connection_summary::ActorConnectionSummary> {
    use futures::future;

    async fn fetch_one(
        url: String,
    ) -> Option<domain::models::actor_connection_summary::ActorConnectionSummary> {
        let resp: serde_json::Value = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            reqwest::Client::new()
                .get(&url)
                .header("Accept", "application/activity+json")
                .send(),
        )
        .await
        .ok()?
        .ok()?
        .json()
        .await
        .ok()?;

        let ap_url = resp["id"].as_str()?.to_string();
        let preferred_username = resp["preferredUsername"].as_str().unwrap_or("").to_string();
        let domain_str = url::Url::parse(&ap_url)
            .ok()
            .and_then(|u| u.host_str().map(|s| s.to_string()))
            .unwrap_or_default();
        let handle = format!("{}@{}", preferred_username, domain_str);
        let display_name = resp["name"].as_str().map(|s| s.to_string());
        let avatar_url = resp["icon"]["url"].as_str().map(|s| s.to_string());

        Some(
            domain::models::actor_connection_summary::ActorConnectionSummary {
                url: ap_url,
                handle,
                display_name,
                avatar_url,
            },
        )
    }

    let futs: Vec<_> = urls.into_iter().map(fetch_one).collect();
    let results = future::join_all(futs).await;

    results
        .into_iter()
        .filter_map(|r| {
            if r.is_none() {
                tracing::warn!("failed to resolve actor profile (timeout or parse error)");
            }
            r
        })
        .collect()
}

async fn webfinger_resolve_actor_url(handle: &str) -> anyhow::Result<String> {
    let normalized = handle.trim_start_matches('@');
    let at = normalized
        .rfind('@')
        .ok_or_else(|| anyhow::anyhow!("handle must be user@domain"))?;
    let (user, domain_str) = (&normalized[..at], &normalized[at + 1..]);
    let wf_url = format!(
        "https://{}/.well-known/webfinger?resource=acct:{}@{}",
        domain_str, user, domain_str
    );
    let wf: serde_json::Value = reqwest::Client::new()
        .get(&wf_url)
        .header("Accept", "application/jrd+json, application/json")
        .send()
        .await?
        .json()
        .await?;
    let self_href = wf["links"]
        .as_array()
        .and_then(|links| {
            links.iter().find(|l| {
                l["rel"].as_str() == Some("self")
                    && l["type"].as_str() == Some("application/activity+json")
            })
        })
        .and_then(|l| l["href"].as_str())
        .ok_or_else(|| anyhow::anyhow!("no self link in WebFinger response"))?
        .to_owned();
    Ok(self_href)
}

// ── ApFederationAdapter ───────────────────────────────────────────────────────

/// Wraps `k_ap::ActivityPubService` together with the `RemoteActorConnectionRepository`
/// (which k-ap doesn't own), and implements all domain federation port traits.
#[derive(Clone)]
pub struct ApFederationAdapter {
    pub(crate) inner: Arc<ActivityPubService>,
    pub(crate) connections_repo: Arc<dyn RemoteActorConnectionRepository>,
}

impl ApFederationAdapter {
    pub fn new(
        inner: Arc<ActivityPubService>,
        connections_repo: Arc<dyn RemoteActorConnectionRepository>,
    ) -> Self {
        Self {
            inner,
            connections_repo,
        }
    }

    pub fn router<S>(&self) -> axum::Router<S>
    where
        S: Clone + Send + Sync + 'static,
    {
        self.inner.router()
    }

    fn base_url(&self) -> &str {
        self.inner.base_url()
    }

    fn actor_ap_id(&self, user_uuid: uuid::Uuid) -> String {
        format!("{}/users/{}", self.base_url(), user_uuid)
    }

    fn actor_followers_url(&self, user_uuid: uuid::Uuid) -> String {
        format!("{}/followers", self.actor_ap_id(user_uuid))
    }
}

// ── OutboundFederationPort ────────────────────────────────────────────────────

#[async_trait]
impl crate::port::OutboundFederationPort for ApFederationAdapter {
    async fn broadcast_create(
        &self,
        author_user_id: &UserId,
        thought: &domain::models::thought::Thought,
        _author_username: &str,
        in_reply_to_url: Option<&str>,
    ) -> Result<(), DomainError> {
        let user_uuid = author_user_id.as_uuid();
        let ap_id = self.actor_ap_id(user_uuid);
        let followers_url = self.actor_followers_url(user_uuid);
        let note = build_note_json(
            thought,
            &ap_id,
            &followers_url,
            self.base_url(),
            in_reply_to_url,
        );
        self.inner
            .broadcast_create_note(
                user_uuid,
                note,
                thought_to_ap_visibility(&thought.visibility),
                vec![],
            )
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))
    }

    async fn broadcast_delete(
        &self,
        author_user_id: &UserId,
        thought_ap_id: &str,
    ) -> Result<(), DomainError> {
        let ap_id =
            url::Url::parse(thought_ap_id).map_err(|e| DomainError::Internal(e.to_string()))?;
        self.inner
            .broadcast_delete_to_followers(author_user_id.as_uuid(), ap_id)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))
    }

    async fn broadcast_update(
        &self,
        author_user_id: &UserId,
        thought: &domain::models::thought::Thought,
        _author_username: &str,
        in_reply_to_url: Option<&str>,
    ) -> Result<(), DomainError> {
        let user_uuid = author_user_id.as_uuid();
        let ap_id = self.actor_ap_id(user_uuid);
        let followers_url = self.actor_followers_url(user_uuid);
        let note = build_note_json(
            thought,
            &ap_id,
            &followers_url,
            self.base_url(),
            in_reply_to_url,
        );
        self.inner
            .broadcast_update_note(
                user_uuid,
                note,
                thought_to_ap_visibility(&thought.visibility),
                vec![],
            )
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))
    }

    async fn broadcast_announce(
        &self,
        booster_user_id: &UserId,
        object_ap_id: &str,
    ) -> Result<(), DomainError> {
        let ap_id =
            url::Url::parse(object_ap_id).map_err(|e| DomainError::Internal(e.to_string()))?;
        self.inner
            .broadcast_announce_to_followers(booster_user_id.as_uuid(), ap_id)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))
    }

    async fn broadcast_undo_announce(
        &self,
        booster_user_id: &UserId,
        object_ap_id: &str,
    ) -> Result<(), DomainError> {
        let ap_id =
            url::Url::parse(object_ap_id).map_err(|e| DomainError::Internal(e.to_string()))?;
        self.inner
            .broadcast_undo_announce_to_followers(booster_user_id.as_uuid(), ap_id)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))
    }

    async fn broadcast_like(
        &self,
        liker_user_id: &UserId,
        object_ap_id: &str,
        author_inbox_url: &str,
    ) -> Result<(), DomainError> {
        let object =
            url::Url::parse(object_ap_id).map_err(|e| DomainError::Internal(e.to_string()))?;
        let inbox =
            url::Url::parse(author_inbox_url).map_err(|e| DomainError::Internal(e.to_string()))?;
        self.inner
            .broadcast_like_to_inbox(liker_user_id.as_uuid(), object, inbox)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))
    }

    async fn broadcast_undo_like(
        &self,
        liker_user_id: &UserId,
        object_ap_id: &str,
        author_inbox_url: &str,
    ) -> Result<(), DomainError> {
        let object =
            url::Url::parse(object_ap_id).map_err(|e| DomainError::Internal(e.to_string()))?;
        let inbox =
            url::Url::parse(author_inbox_url).map_err(|e| DomainError::Internal(e.to_string()))?;
        self.inner
            .broadcast_undo_like_to_inbox(liker_user_id.as_uuid(), object, inbox)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))
    }

    async fn broadcast_actor_update(&self, user_id: &UserId) -> Result<(), DomainError> {
        self.inner
            .broadcast_actor_update(user_id.as_uuid())
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))
    }
}

// ── FederationSchedulerPort ───────────────────────────────────────────────────

#[async_trait]
impl FederationSchedulerPort for ApFederationAdapter {
    async fn schedule_actor_posts_fetch(
        &self,
        actor_ap_url: &str,
        outbox_url: &str,
    ) -> Result<(), DomainError> {
        let service = self.inner.clone();
        let actor = actor_ap_url.to_string();
        let outbox = outbox_url.to_string();
        tokio::spawn(async move {
            if let Err(e) = service.import_remote_outbox(&outbox, &actor).await {
                tracing::warn!(actor = %actor, error = %e, "posts backfill failed");
            }
        });
        Ok(())
    }

    async fn schedule_connections_fetch(
        &self,
        actor_ap_url: &str,
        collection_url: &str,
        connection_type: &str,
        page: u32,
    ) -> Result<(), DomainError> {
        if page != 1 {
            return Ok(());
        }
        let actor = actor_ap_url.to_string();
        let collection = collection_url.to_string();
        let conn_type = connection_type.to_string();
        let connections_repo = self.connections_repo.clone();
        tokio::spawn(async move {
            let client = match reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(HTTP_FETCH_TIMEOUT_SECS))
                .build()
            {
                Ok(c) => c,
                Err(e) => {
                    tracing::warn!(error = %e, "connections fetch: failed to build client");
                    return;
                }
            };

            let mut all_urls: Vec<String> = Vec::new();
            let mut current_url: Option<String> = Some(collection.clone());
            const MAX_ACTORS: usize = 500;

            while let Some(url) = current_url.take() {
                let val: serde_json::Value = match client
                    .get(&url)
                    .header("Accept", "application/activity+json, application/ld+json")
                    .send()
                    .await
                {
                    Ok(r) => match r.json().await {
                        Ok(v) => v,
                        Err(e) => {
                            tracing::warn!(error = %e, url = %url, "connections: parse error");
                            break;
                        }
                    },
                    Err(e) => {
                        tracing::warn!(error = %e, url = %url, "connections: HTTP error");
                        break;
                    }
                };

                if val["type"].as_str() == Some("OrderedCollection") {
                    current_url = val["first"].as_str().map(|s| s.to_string());
                    continue;
                }

                let empty = vec![];
                let items = val["orderedItems"].as_array().unwrap_or(&empty);
                for item in items {
                    let actor_url = item.as_str().or_else(|| item["id"].as_str()).unwrap_or("");
                    if !actor_url.is_empty() {
                        all_urls.push(actor_url.to_string());
                    }
                }

                if all_urls.len() >= MAX_ACTORS {
                    break;
                }
                current_url = val["next"].as_str().map(|s| s.to_string());
                if current_url.is_some() {
                    tokio::time::sleep(std::time::Duration::from_millis(BATCH_FETCH_SLEEP_MS))
                        .await;
                }
            }

            if all_urls.is_empty() {
                tracing::debug!(
                    actor = %actor,
                    connection_type = %conn_type,
                    "connections: empty collection"
                );
                return;
            }

            const PAGE_SIZE: usize = 20;
            for (idx, chunk) in all_urls.chunks(PAGE_SIZE).enumerate() {
                let page_num = (idx + 1) as u32;
                let resolved = resolve_actor_profiles_from_urls(chunk.to_vec()).await;
                if let Err(e) = connections_repo
                    .upsert_connections(&actor, &conn_type, page_num, &resolved)
                    .await
                {
                    tracing::warn!(error = %e, "connections: upsert failed");
                }
            }

            tracing::debug!(
                actor = %actor,
                connection_type = %conn_type,
                count = all_urls.len(),
                "connections fetch complete"
            );
        });
        Ok(())
    }
}

// ── FederationLookupPort ──────────────────────────────────────────────────────

#[async_trait]
impl FederationLookupPort for ApFederationAdapter {
    async fn lookup_actor(&self, handle: &str) -> Result<DomainRemoteActor, DomainError> {
        let actor = self
            .inner
            .lookup_actor_by_handle(handle)
            .await
            .map_err(|e| DomainError::ExternalService(e.to_string()))?;

        Ok(DomainRemoteActor {
            url: actor.ap_url.to_string(),
            handle: actor.handle,
            display_name: actor.display_name,
            avatar_url: actor.avatar_url.as_ref().map(|u| u.to_string()),
            outbox_url: actor.outbox_url.as_ref().map(|u| u.to_string()),
            last_fetched_at: chrono::Utc::now(),
            bio: actor.bio,
            banner_url: actor.banner_url.as_ref().map(|u| u.to_string()),
            also_known_as: actor.also_known_as.into_iter().next(),
            followers_url: actor.followers_url.as_ref().map(|u| u.to_string()),
            following_url: actor.following_url.as_ref().map(|u| u.to_string()),
            attachment: actor
                .attachment
                .into_iter()
                .map(|f| (f.name, f.value))
                .collect(),
        })
    }

    async fn actor_json(&self, user_id: &UserId) -> Result<String, DomainError> {
        self.inner
            .actor_json(&user_id.as_uuid().to_string())
            .await
            .map_err(|e| DomainError::ExternalService(e.to_string()))
    }

    async fn followers_collection_json(
        &self,
        user_id: &UserId,
        page: Option<u32>,
    ) -> Result<String, DomainError> {
        self.inner
            .followers_collection_json(user_id.as_uuid(), page)
            .await
            .map_err(|e| DomainError::ExternalService(e.to_string()))
    }

    async fn following_collection_json(
        &self,
        user_id: &UserId,
        page: Option<u32>,
    ) -> Result<String, DomainError> {
        self.inner
            .following_collection_json(user_id.as_uuid(), page)
            .await
            .map_err(|e| DomainError::ExternalService(e.to_string()))
    }
}

// ── FederationFetchPort ───────────────────────────────────────────────────────

#[async_trait]
impl FederationFetchPort for ApFederationAdapter {
    async fn fetch_outbox_page(
        &self,
        outbox_url: &str,
        page: u32,
    ) -> Result<Vec<domain::models::remote_note::RemoteNote>, DomainError> {
        use chrono::DateTime;

        let client = reqwest::Client::new();
        let base: serde_json::Value = client
            .get(outbox_url)
            .header("Accept", "application/activity+json, application/ld+json")
            .send()
            .await
            .map_err(|e| DomainError::ExternalService(e.to_string()))?
            .json()
            .await
            .map_err(|e| DomainError::ExternalService(e.to_string()))?;

        let url = base["first"]
            .as_str()
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("{}?page={}", outbox_url, page));

        let resp: serde_json::Value = client
            .get(&url)
            .header("Accept", "application/activity+json, application/ld+json")
            .send()
            .await
            .map_err(|e| DomainError::ExternalService(e.to_string()))?
            .json()
            .await
            .map_err(|e| DomainError::ExternalService(e.to_string()))?;

        let empty = vec![];
        let items = resp["orderedItems"].as_array().unwrap_or(&empty);

        let notes = items
            .iter()
            .filter_map(|item| {
                let note = if item["type"].as_str() == Some("Create") {
                    &item["object"]
                } else if item["type"].as_str() == Some("Note") {
                    item
                } else {
                    return None;
                };

                let to = note["to"].as_array()?;
                let is_public = to
                    .iter()
                    .any(|t| t.as_str() == Some("https://www.w3.org/ns/activitystreams#Public"));
                if !is_public {
                    return None;
                }

                let published = DateTime::parse_from_rfc3339(note["published"].as_str()?)
                    .ok()?
                    .with_timezone(&chrono::Utc);

                let text = note["content"].as_str().unwrap_or("").to_string();
                let has_attachments = note["attachment"]
                    .as_array()
                    .map(|a| !a.is_empty())
                    .unwrap_or(false);

                let content = if has_attachments {
                    let notice =
                        "<p class=\"media-notice\">📎 Media attachment — not supported</p>";
                    if text.is_empty() {
                        notice.to_string()
                    } else {
                        format!("{text}{notice}")
                    }
                } else {
                    text
                };

                Some(domain::models::remote_note::RemoteNote {
                    ap_id: note["id"].as_str()?.to_string(),
                    content,
                    published,
                    sensitive: note["sensitive"].as_bool().unwrap_or(false),
                    content_warning: note["summary"].as_str().map(|s| s.to_string()),
                })
            })
            .collect();

        Ok(notes)
    }

    async fn fetch_actor_urls_from_collection(
        &self,
        collection_url: &str,
    ) -> Result<Vec<String>, DomainError> {
        let client = reqwest::Client::new();
        let base: serde_json::Value = client
            .get(collection_url)
            .header("Accept", "application/activity+json, application/ld+json")
            .send()
            .await
            .map_err(|e| DomainError::ExternalService(e.to_string()))?
            .json()
            .await
            .map_err(|e| DomainError::ExternalService(e.to_string()))?;

        let page = if base["orderedItems"].is_null() {
            if let Some(first_url) = base["first"].as_str() {
                client
                    .get(first_url)
                    .header("Accept", "application/activity+json, application/ld+json")
                    .send()
                    .await
                    .map_err(|e| DomainError::ExternalService(e.to_string()))?
                    .json()
                    .await
                    .map_err(|e| DomainError::ExternalService(e.to_string()))?
            } else {
                base
            }
        } else {
            base
        };

        let empty = vec![];
        let items = page["orderedItems"].as_array().unwrap_or(&empty);
        Ok(items
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect())
    }

    async fn resolve_actor_profiles(
        &self,
        urls: Vec<String>,
    ) -> Vec<domain::models::actor_connection_summary::ActorConnectionSummary> {
        resolve_actor_profiles_from_urls(urls).await
    }
}

// ── FederationFollowPort ──────────────────────────────────────────────────────

#[async_trait]
impl FederationFollowPort for ApFederationAdapter {
    async fn follow_remote(&self, local_user_id: &UserId, handle: &str) -> Result<(), DomainError> {
        self.inner
            .follow(local_user_id.as_uuid(), handle)
            .await
            .map_err(|e| DomainError::ExternalService(e.to_string()))
    }

    async fn unfollow_remote(
        &self,
        local_user_id: &UserId,
        handle: &str,
    ) -> Result<(), DomainError> {
        let actor_url = webfinger_resolve_actor_url(handle)
            .await
            .map_err(|e| DomainError::ExternalService(e.to_string()))?;
        self.inner
            .unfollow(local_user_id.as_uuid(), &actor_url)
            .await
            .map_err(|e| DomainError::ExternalService(e.to_string()))
    }

    async fn get_remote_following(
        &self,
        user_id: &UserId,
    ) -> Result<Vec<DomainRemoteActor>, DomainError> {
        self.inner
            .get_following(user_id.as_uuid())
            .await
            .map(|v| v.into_iter().map(k_ap_actor_to_domain).collect())
            .map_err(|e| DomainError::ExternalService(e.to_string()))
    }

    async fn broadcast_move(
        &self,
        user_id: &UserId,
        new_actor_url: url::Url,
    ) -> Result<(), DomainError> {
        self.inner
            .broadcast_move(user_id.as_uuid(), new_actor_url)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))
    }
}

// ── FederationFollowRequestPort ───────────────────────────────────────────────

#[async_trait]
impl FederationFollowRequestPort for ApFederationAdapter {
    async fn get_pending_followers(
        &self,
        user_id: &UserId,
    ) -> Result<Vec<DomainRemoteActor>, DomainError> {
        self.inner
            .get_pending_followers(user_id.as_uuid())
            .await
            .map(|v| v.into_iter().map(k_ap_actor_to_domain).collect())
            .map_err(|e| DomainError::ExternalService(e.to_string()))
    }

    async fn accept_follow_request(
        &self,
        user_id: &UserId,
        actor_url: &str,
    ) -> Result<(), DomainError> {
        self.inner
            .accept_follower(user_id.as_uuid(), actor_url)
            .await
            .map_err(|e| DomainError::ExternalService(e.to_string()))
    }

    async fn reject_follow_request(
        &self,
        user_id: &UserId,
        actor_url: &str,
    ) -> Result<(), DomainError> {
        self.inner
            .reject_follower(user_id.as_uuid(), actor_url)
            .await
            .map_err(|e| DomainError::ExternalService(e.to_string()))
    }

    async fn get_remote_followers(
        &self,
        user_id: &UserId,
    ) -> Result<Vec<DomainRemoteActor>, DomainError> {
        self.inner
            .get_accepted_followers(user_id.as_uuid())
            .await
            .map(|v| v.into_iter().map(k_ap_actor_to_domain).collect())
            .map_err(|e| DomainError::ExternalService(e.to_string()))
    }

    async fn remove_remote_follower(
        &self,
        user_id: &UserId,
        actor_url: &str,
    ) -> Result<(), DomainError> {
        self.inner
            .remove_follower(user_id.as_uuid(), actor_url)
            .await
            .map_err(|e| DomainError::ExternalService(e.to_string()))
    }

    async fn mark_follower_accepted(
        &self,
        user_id: &UserId,
        actor_url: &str,
    ) -> Result<(), DomainError> {
        self.inner
            .mark_follower_accepted(user_id.as_uuid(), actor_url)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))
    }

    async fn mark_follower_rejected(
        &self,
        user_id: &UserId,
        actor_url: &str,
    ) -> Result<(), DomainError> {
        self.inner
            .mark_follower_rejected(user_id.as_uuid(), actor_url)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))
    }
}

// FederationActionPort is a blanket supertrait; no explicit impl needed.
