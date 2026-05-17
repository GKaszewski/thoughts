use std::sync::Arc;

use domain::ports::FederationFetchPort;

use activitypub_federation::{
    activity_sending::SendActivityTask, fetch::object_id::ObjectId, protocol::context::WithContext,
    traits::Actor,
};
use axum::{Router, routing::get, routing::post};
use url::Url;

use crate::{
    activities::{
        AcceptActivity, CreateActivity, FollowActivity, RejectActivity, UndoActivity,
        UpdateActivity,
    },
    actors::{DbActor, get_local_actor},
    content::ApObjectHandler,
    data::FederationData,
    federation::ApFederationConfig,
    inbox::inbox_handler,
    nodeinfo::{nodeinfo_handler, nodeinfo_well_known_handler},
    outbox::outbox_handler,
    repository::{
        BlockedDomain, FederationRepository, FollowerStatus, FollowingStatus, RemoteActor,
    },
    urls::activity_url,
    user::ApUserRepository,
    webfinger::webfinger_handler,
};

const DELIVERY_MAX_ATTEMPTS: u32 = 3;
const DELIVERY_INITIAL_DELAY_SECS: u64 = 1;
const HTTP_FETCH_TIMEOUT_SECS: u64 = 30;
const BATCH_FETCH_SLEEP_MS: u64 = 100;

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

fn thought_note_json(
    thought: &domain::models::thought::Thought,
    local_actor: &crate::actors::DbActor,
    base_url: &str,
    in_reply_to_url: Option<&str>,
) -> anyhow::Result<(url::Url, serde_json::Value)> {
    let ap_id = url::Url::parse(&format!("{}/thoughts/{}", base_url, thought.id))?;

    // Build to/cc based on visibility per AP spec.
    let (to, cc) = match thought.visibility {
        domain::models::thought::Visibility::Public => (
            vec![crate::urls::AS_PUBLIC.to_string()],
            vec![local_actor.followers_url.to_string()],
        ),
        domain::models::thought::Visibility::Unlisted => (
            vec![local_actor.followers_url.to_string()],
            vec![crate::urls::AS_PUBLIC.to_string()],
        ),
        domain::models::thought::Visibility::Followers => {
            (vec![local_actor.followers_url.to_string()], vec![])
        }
        domain::models::thought::Visibility::Direct => (vec![], vec![]),
    };

    let mut note = serde_json::json!({
        "type": "Note",
        "id": ap_id.to_string(),
        "url": ap_id.to_string(),
        "attributedTo": local_actor.ap_id.to_string(),
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
    Ok((ap_id, note))
}

fn collect_inboxes(followers: &[crate::repository::Follower]) -> Vec<Url> {
    let mut seen = std::collections::HashSet::new();
    let mut inboxes = Vec::new();
    for f in followers {
        let inbox_str = f
            .actor
            .shared_inbox_url
            .as_deref()
            .unwrap_or(&f.actor.inbox_url);
        if seen.insert(inbox_str.to_string())
            && let Ok(url) = Url::parse(inbox_str)
        {
            inboxes.push(url);
        }
    }
    inboxes
}

pub(crate) async fn send_with_retry(
    sends: Vec<SendActivityTask>,
    data: &activitypub_federation::config::Data<FederationData>,
) -> Vec<anyhow::Error> {
    let mut failures = vec![];
    for send in sends {
        let mut delay = std::time::Duration::from_secs(DELIVERY_INITIAL_DELAY_SECS);
        for attempt in 1..=DELIVERY_MAX_ATTEMPTS {
            match send.clone().sign_and_send(data).await {
                Ok(()) => break,
                Err(e) if attempt < DELIVERY_MAX_ATTEMPTS => {
                    tracing::warn!(attempt, error = %e, "delivery failed, retrying");
                    tokio::time::sleep(delay).await;
                    delay *= 2;
                }
                Err(e) => {
                    tracing::error!(attempt, error = %e, "delivery failed permanently");
                    failures.push(anyhow::anyhow!(e));
                }
            }
        }
    }
    failures
}

#[derive(Clone)]
pub struct ActivityPubService {
    federation_config: ApFederationConfig,
    base_url: String,
    connections_repo: Arc<dyn domain::ports::RemoteActorConnectionRepository>,
}

impl ActivityPubService {
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        repo: Arc<dyn FederationRepository>,
        user_repo: Arc<dyn ApUserRepository>,
        object_handler: Arc<dyn ApObjectHandler>,
        base_url: String,
        allow_registration: bool,
        software_name: String,
        debug: bool,
        event_publisher: Option<Arc<dyn domain::ports::EventPublisher>>,
        connections_repo: Arc<dyn domain::ports::RemoteActorConnectionRepository>,
    ) -> anyhow::Result<Self> {
        let data = FederationData::new(
            repo,
            user_repo,
            object_handler,
            base_url.clone(),
            allow_registration,
            software_name,
            event_publisher,
        );
        let federation_config = ApFederationConfig::new(data, debug).await?;
        Ok(Self {
            federation_config,
            base_url,
            connections_repo,
        })
    }

    pub fn federation_config(&self) -> &ApFederationConfig {
        &self.federation_config
    }

    pub fn request_data(&self) -> activitypub_federation::config::Data<FederationData> {
        self.federation_config.to_request_data()
    }

    /// Returns `(local_actor, deduplicated_inboxes)` for all accepted followers,
    /// excluding blocked actors and blocked domains.
    /// Returns `None` if there are no eligible followers.
    async fn accepted_follower_inboxes(
        &self,
        data: &activitypub_federation::config::Data<FederationData>,
        local_user_id: uuid::Uuid,
    ) -> anyhow::Result<Option<(crate::actors::DbActor, Vec<Url>)>> {
        let local_actor = get_local_actor(local_user_id, data)
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?;

        let followers = data.federation_repo.get_followers(local_user_id).await?;
        let blocked = data
            .federation_repo
            .get_blocked_actors(local_user_id)
            .await
            .unwrap_or_default();
        let blocked_set: std::collections::HashSet<String> = blocked.into_iter().collect();
        let blocked_domains = data
            .federation_repo
            .get_blocked_domains()
            .await
            .unwrap_or_default();
        let blocked_domain_set: std::collections::HashSet<String> =
            blocked_domains.into_iter().map(|d| d.domain).collect();

        let accepted: Vec<_> = followers
            .into_iter()
            .filter(|f| f.status == FollowerStatus::Accepted)
            .filter(|f| !blocked_set.contains(&f.actor.url))
            .filter(|f| {
                let domain = url::Url::parse(&f.actor.inbox_url)
                    .ok()
                    .and_then(|u| u.host_str().map(|s| s.to_string()))
                    .unwrap_or_default();
                !blocked_domain_set.contains(&domain)
            })
            .collect();

        if accepted.is_empty() {
            return Ok(None);
        }

        Ok(Some((local_actor, collect_inboxes(&accepted))))
    }

    pub async fn actor_json(&self, user_id_str: &str) -> anyhow::Result<String> {
        use activitypub_federation::traits::Object;
        let uuid = uuid::Uuid::parse_str(user_id_str)?;
        let data = self.federation_config.to_request_data();
        let actor = get_local_actor(uuid, &data)
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        let person = actor
            .into_json(&data)
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        Ok(serde_json::to_string(&WithContext::new_default(person))?)
    }

    /// Returns the ActivityPub router compatible with any outer state `S`.
    /// Handlers only use `Data<FederationData>` injected by the middleware layer,
    /// so the router is independent of the application state type.
    pub fn router<S>(&self) -> Router<S>
    where
        S: Clone + Send + Sync + 'static,
    {
        Router::new()
            .route("/.well-known/nodeinfo", get(nodeinfo_well_known_handler))
            .route("/nodeinfo/2.0", get(nodeinfo_handler))
            .route("/.well-known/webfinger", get(webfinger_handler))
            .route("/inbox", post(inbox_handler))
            .route("/users/{id}/inbox", post(inbox_handler))
            .route("/users/{id}/outbox", get(outbox_handler))
            .layer(self.federation_config.middleware())
    }

    /// Fan out an Announce activity to all accepted followers.
    pub async fn broadcast_announce_to_followers(
        &self,
        local_user_id: uuid::Uuid,
        object_ap_id: url::Url,
    ) -> anyhow::Result<()> {
        // Deterministic ID so Undo(Announce) can reference this same activity.
        let announce_id = url::Url::parse(&format!(
            "{}/activities/announce/{}",
            self.base_url,
            uuid::Uuid::new_v5(
                &uuid::Uuid::NAMESPACE_URL,
                format!("{}/{}", local_user_id, object_ap_id).as_bytes(),
            )
        ))
        .map_err(|e| anyhow::anyhow!("{e}"))?;

        let data = self.federation_config.to_request_data();
        let Some((local_actor, inboxes)) =
            self.accepted_follower_inboxes(&data, local_user_id).await?
        else {
            return Ok(());
        };

        let announce = crate::activities::AnnounceActivity {
            id: announce_id,
            kind: Default::default(),
            actor: activitypub_federation::fetch::object_id::ObjectId::from(
                local_actor.ap_id.clone(),
            ),
            object: object_ap_id,
            published: Some(chrono::Utc::now()),
            to: vec![crate::urls::AS_PUBLIC.to_string()],
            cc: vec![local_actor.followers_url.to_string()],
        };

        let sends = activitypub_federation::activity_sending::SendActivityTask::prepare(
            &activitypub_federation::protocol::context::WithContext::new_default(announce),
            &local_actor,
            inboxes,
            &data,
        )
        .await?;
        let failures = send_with_retry(sends, &data).await;
        if !failures.is_empty() {
            tracing::warn!(count = failures.len(), "some Announce deliveries failed");
        }
        Ok(())
    }

    /// Fan out an Undo(Announce) activity to all accepted followers.
    pub async fn broadcast_undo_announce_to_followers(
        &self,
        local_user_id: uuid::Uuid,
        object_ap_id: url::Url,
    ) -> anyhow::Result<()> {
        // Reconstruct the same deterministic announce ID used when the boost was sent.
        let announce_id = url::Url::parse(&format!(
            "{}/activities/announce/{}",
            self.base_url,
            uuid::Uuid::new_v5(
                &uuid::Uuid::NAMESPACE_URL,
                format!("{}/{}", local_user_id, object_ap_id).as_bytes(),
            )
        ))
        .map_err(|e| anyhow::anyhow!("{e}"))?;

        let undo_id =
            crate::urls::activity_url(&self.base_url).map_err(|e| anyhow::anyhow!("{e}"))?;

        let data = self.federation_config.to_request_data();
        let Some((local_actor, inboxes)) =
            self.accepted_follower_inboxes(&data, local_user_id).await?
        else {
            return Ok(());
        };

        let undo = crate::activities::UndoActivity {
            id: undo_id,
            kind: Default::default(),
            actor: activitypub_federation::fetch::object_id::ObjectId::from(
                local_actor.ap_id.clone(),
            ),
            object: serde_json::json!({
                "type": "Announce",
                "id": announce_id.to_string(),
                "actor": local_actor.ap_id.to_string(),
                "object": object_ap_id.to_string(),
            }),
        };

        let sends = activitypub_federation::activity_sending::SendActivityTask::prepare(
            &activitypub_federation::protocol::context::WithContext::new_default(undo),
            &local_actor,
            inboxes,
            &data,
        )
        .await?;
        let failures = send_with_retry(sends, &data).await;
        if !failures.is_empty() {
            tracing::warn!(
                count = failures.len(),
                "some Undo(Announce) deliveries failed"
            );
        }
        Ok(())
    }

    /// Send a Like activity to a single inbox.
    pub async fn broadcast_like_to_inbox(
        &self,
        liker_user_id: uuid::Uuid,
        object_ap_id: url::Url,
        author_inbox_url: url::Url,
    ) -> anyhow::Result<()> {
        let data = self.federation_config.to_request_data();
        let local_actor = get_local_actor(liker_user_id, &data)
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?;

        // Deterministic ID so Undo(Like) can reference the same activity.
        let like_id = url::Url::parse(&format!(
            "{}/activities/like/{}",
            self.base_url,
            uuid::Uuid::new_v5(
                &uuid::Uuid::NAMESPACE_URL,
                format!("{}/{}", liker_user_id, object_ap_id).as_bytes(),
            )
        ))?;

        let like = crate::activities::LikeActivity {
            id: like_id,
            kind: Default::default(),
            actor: ObjectId::from(local_actor.ap_id.clone()),
            object: object_ap_id,
        };

        let sends = SendActivityTask::prepare(
            &WithContext::new_default(like),
            &local_actor,
            vec![author_inbox_url],
            &data,
        )
        .await?;
        let failures = send_with_retry(sends, &data).await;
        if !failures.is_empty() {
            tracing::warn!(
                count = failures.len(),
                "some Like deliveries failed permanently"
            );
        }
        Ok(())
    }

    /// Send an Undo(Like) activity to a single inbox.
    pub async fn broadcast_undo_like_to_inbox(
        &self,
        liker_user_id: uuid::Uuid,
        object_ap_id: url::Url,
        author_inbox_url: url::Url,
    ) -> anyhow::Result<()> {
        let data = self.federation_config.to_request_data();
        let local_actor = get_local_actor(liker_user_id, &data)
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?;

        // Reconstruct the same deterministic like ID.
        let like_id = url::Url::parse(&format!(
            "{}/activities/like/{}",
            self.base_url,
            uuid::Uuid::new_v5(
                &uuid::Uuid::NAMESPACE_URL,
                format!("{}/{}", liker_user_id, object_ap_id).as_bytes(),
            )
        ))?;

        let undo_id = activity_url(&self.base_url).map_err(|e| anyhow::anyhow!("{e}"))?;

        let undo = crate::activities::UndoActivity {
            id: undo_id,
            kind: Default::default(),
            actor: ObjectId::from(local_actor.ap_id.clone()),
            object: serde_json::json!({
                "type": "Like",
                "id": like_id.to_string(),
                "actor": local_actor.ap_id.to_string(),
                "object": object_ap_id.to_string(),
            }),
        };

        let sends = SendActivityTask::prepare(
            &WithContext::new_default(undo),
            &local_actor,
            vec![author_inbox_url],
            &data,
        )
        .await?;
        let failures = send_with_retry(sends, &data).await;
        if !failures.is_empty() {
            tracing::warn!(
                count = failures.len(),
                "some Undo(Like) deliveries failed permanently"
            );
        }
        Ok(())
    }

    /// Resolve a `@user@domain` handle to a `DbActor` over HTTPS directly.
    /// The library's `webfinger_resolve_actor` tries HTTP first in debug mode, which breaks
    /// on servers that don't redirect HTTP → HTTPS.
    async fn webfinger_https(
        handle: &str,
        data: &activitypub_federation::config::Data<FederationData>,
    ) -> anyhow::Result<DbActor> {
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
        let self_url = url::Url::parse(&self_href)?;
        let actor: DbActor = ObjectId::from(self_url)
            .dereference(data)
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        Ok(actor)
    }

    pub async fn follow(&self, local_user_id: uuid::Uuid, handle: &str) -> anyhow::Result<()> {
        let data = self.federation_config.to_request_data();

        let normalized = handle.trim_start_matches('@');
        let parts: Vec<&str> = normalized.splitn(2, '@').collect();
        if parts.len() == 2 && parts[1] == data.domain {
            return self.follow_local(local_user_id, parts[0], &data).await;
        }

        let remote_actor: DbActor = Self::webfinger_https(handle, &data).await?;

        let local_actor = get_local_actor(local_user_id, &data)
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?;

        let follow_id = activity_url(&self.base_url).map_err(|e| anyhow::anyhow!("{e}"))?;
        let follow_id_str = follow_id.to_string();
        let follow = FollowActivity {
            id: follow_id,
            kind: Default::default(),
            actor: ObjectId::from(local_actor.ap_id.clone()),
            object: ObjectId::from(remote_actor.ap_id.clone()),
        };
        let follow_with_ctx = WithContext::new_default(follow);

        let sends = SendActivityTask::prepare(
            &follow_with_ctx,
            &local_actor,
            vec![remote_actor.inbox()],
            &data,
        )
        .await?;
        let failures = send_with_retry(sends, &data).await;
        if !failures.is_empty() {
            tracing::warn!(
                count = failures.len(),
                "some activity deliveries failed permanently"
            );
        }

        let domain = remote_actor.ap_id.host_str().unwrap_or("");
        let full_handle = format!("{}@{}", remote_actor.username, domain);
        let remote = RemoteActor {
            url: remote_actor.ap_id.to_string(),
            handle: full_handle,
            inbox_url: remote_actor.inbox_url.to_string(),
            shared_inbox_url: remote_actor
                .shared_inbox_url
                .as_ref()
                .map(|u| u.to_string()),
            display_name: Some(remote_actor.username.clone()),
            avatar_url: remote_actor.avatar_url.as_ref().map(|u| u.to_string()),
            outbox_url: Some(remote_actor.outbox_url.to_string()),
        };
        data.federation_repo
            .add_following(local_user_id, remote, &follow_id_str)
            .await?;

        Ok(())
    }

    pub async fn unfollow(
        &self,
        local_user_id: uuid::Uuid,
        actor_url_str: &str,
    ) -> anyhow::Result<()> {
        let data = self.federation_config.to_request_data();

        if actor_url_str.starts_with(&self.base_url) {
            return self
                .unfollow_local(local_user_id, actor_url_str, &data)
                .await;
        }

        let remote = data
            .federation_repo
            .get_remote_actor(actor_url_str)
            .await?
            .ok_or_else(|| anyhow::anyhow!("remote actor not found: {}", actor_url_str))?;

        let local_actor = get_local_actor(local_user_id, &data)
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?;

        let remote_ap_id = Url::parse(actor_url_str)?;
        let inbox = Url::parse(&remote.inbox_url)?;

        let follow_activity_id_str = data
            .federation_repo
            .get_follow_activity_id(local_user_id, actor_url_str)
            .await?;
        let follow_id = match follow_activity_id_str {
            Some(id) => Url::parse(&id)?,
            None => activity_url(&self.base_url).map_err(|e| anyhow::anyhow!("{e}"))?,
        };
        let follow = FollowActivity {
            id: follow_id,
            kind: Default::default(),
            actor: ObjectId::from(local_actor.ap_id.clone()),
            object: ObjectId::from(remote_ap_id),
        };

        let undo_id = activity_url(&self.base_url).map_err(|e| anyhow::anyhow!("{e}"))?;
        let undo = UndoActivity {
            id: undo_id,
            kind: Default::default(),
            actor: ObjectId::from(local_actor.ap_id.clone()),
            object: serde_json::to_value(&follow).map_err(|e| anyhow::anyhow!("{e}"))?,
        };

        let sends = SendActivityTask::prepare(
            &WithContext::new_default(undo),
            &local_actor,
            vec![inbox],
            &data,
        )
        .await?;
        let failures = send_with_retry(sends, &data).await;
        if !failures.is_empty() {
            tracing::warn!(
                count = failures.len(),
                "some activity deliveries failed permanently"
            );
        }

        data.federation_repo
            .remove_following(local_user_id, actor_url_str)
            .await?;

        data.object_handler
            .on_actor_removed(&Url::parse(actor_url_str)?)
            .await?;

        Ok(())
    }

    pub async fn accept_follower(
        &self,
        local_user_id: uuid::Uuid,
        remote_actor_url: &str,
    ) -> anyhow::Result<()> {
        let data = self.federation_config.to_request_data();
        let local_actor = get_local_actor(local_user_id, &data)
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?;

        let remote_actor = data
            .federation_repo
            .get_remote_actor(remote_actor_url)
            .await?
            .ok_or_else(|| anyhow::anyhow!("remote actor not found"))?;

        let follow_id_str = data
            .federation_repo
            .get_follower_follow_activity_id(local_user_id, remote_actor_url)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!("follow activity id not found for {}", remote_actor_url)
            })?;
        let follow_id = Url::parse(&follow_id_str)?;
        let follow = FollowActivity {
            id: follow_id,
            kind: Default::default(),
            actor: ObjectId::from(Url::parse(remote_actor_url)?),
            object: ObjectId::from(local_actor.ap_id.clone()),
        };
        let accept = AcceptActivity {
            id: activity_url(&self.base_url).map_err(|e| anyhow::anyhow!("{e}"))?,
            kind: Default::default(),
            actor: ObjectId::from(local_actor.ap_id.clone()),
            object: follow,
        };

        data.federation_repo
            .update_follower_status(local_user_id, remote_actor_url, FollowerStatus::Accepted)
            .await?;

        let inbox = Url::parse(&remote_actor.inbox_url)?;
        let sends = SendActivityTask::prepare(
            &WithContext::new_default(accept),
            &local_actor,
            vec![inbox.clone()],
            &data,
        )
        .await?;
        let failures = send_with_retry(sends, &data).await;
        if !failures.is_empty() {
            tracing::warn!(
                "failed to deliver Accept activity, but follower is marked accepted locally"
            );
        }

        let target_inbox = remote_actor
            .shared_inbox_url
            .clone()
            .unwrap_or_else(|| remote_actor.inbox_url.clone());
        self.spawn_backfill(local_user_id, target_inbox);

        Ok(())
    }

    pub async fn reject_follower(
        &self,
        local_user_id: uuid::Uuid,
        remote_actor_url: &str,
    ) -> anyhow::Result<()> {
        let data = self.federation_config.to_request_data();
        let local_actor = get_local_actor(local_user_id, &data)
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?;

        let remote_actor = data
            .federation_repo
            .get_remote_actor(remote_actor_url)
            .await?
            .ok_or_else(|| anyhow::anyhow!("remote actor not found"))?;

        let follow_id = activity_url(&self.base_url).map_err(|e| anyhow::anyhow!("{e}"))?;
        let follow = FollowActivity {
            id: follow_id,
            kind: Default::default(),
            actor: ObjectId::from(Url::parse(remote_actor_url)?),
            object: ObjectId::from(local_actor.ap_id.clone()),
        };
        let reject = RejectActivity {
            id: activity_url(&self.base_url).map_err(|e| anyhow::anyhow!("{e}"))?,
            kind: Default::default(),
            actor: ObjectId::from(local_actor.ap_id.clone()),
            object: follow,
        };

        let inbox = Url::parse(&remote_actor.inbox_url)?;
        let sends = SendActivityTask::prepare(
            &WithContext::new_default(reject),
            &local_actor,
            vec![inbox],
            &data,
        )
        .await?;
        let failures = send_with_retry(sends, &data).await;
        if !failures.is_empty() {
            tracing::warn!(
                count = failures.len(),
                "some activity deliveries failed permanently"
            );
        }

        data.federation_repo
            .remove_follower(local_user_id, remote_actor_url)
            .await?;

        Ok(())
    }

    pub async fn get_pending_followers(
        &self,
        local_user_id: uuid::Uuid,
    ) -> anyhow::Result<Vec<RemoteActor>> {
        let data = self.federation_config.to_request_data();
        data.federation_repo
            .get_pending_followers(local_user_id)
            .await
    }

    pub async fn get_accepted_followers(
        &self,
        local_user_id: uuid::Uuid,
    ) -> anyhow::Result<Vec<RemoteActor>> {
        let data = self.federation_config.to_request_data();
        let followers = data.federation_repo.get_followers(local_user_id).await?;
        Ok(followers
            .into_iter()
            .filter(|f| f.status == FollowerStatus::Accepted)
            .map(|f| f.actor)
            .collect())
    }

    pub async fn count_accepted_followers(
        &self,
        local_user_id: uuid::Uuid,
    ) -> anyhow::Result<usize> {
        let data = self.federation_config.to_request_data();
        let followers = data.federation_repo.get_followers(local_user_id).await?;
        Ok(followers
            .into_iter()
            .filter(|f| f.status == FollowerStatus::Accepted)
            .count())
    }

    pub async fn get_following(
        &self,
        local_user_id: uuid::Uuid,
    ) -> anyhow::Result<Vec<RemoteActor>> {
        let data = self.federation_config.to_request_data();
        data.federation_repo.get_following(local_user_id).await
    }

    pub async fn count_following(&self, local_user_id: uuid::Uuid) -> anyhow::Result<usize> {
        let data = self.federation_config.to_request_data();
        data.federation_repo.count_following(local_user_id).await
    }

    pub async fn remove_follower(
        &self,
        local_user_id: uuid::Uuid,
        actor_url: &str,
    ) -> anyhow::Result<()> {
        let data = self.federation_config.to_request_data();
        data.federation_repo
            .remove_follower(local_user_id, actor_url)
            .await
    }

    /// Broadcast a Delete activity to all accepted followers for a removed review.
    pub async fn broadcast_delete_to_followers(
        &self,
        local_user_id: uuid::Uuid,
        ap_id: Url,
    ) -> anyhow::Result<()> {
        let data = self.federation_config.to_request_data();
        let Some((local_actor, inboxes)) =
            self.accepted_follower_inboxes(&data, local_user_id).await?
        else {
            return Ok(());
        };

        let delete_id =
            crate::urls::activity_url(&self.base_url).map_err(|e| anyhow::anyhow!("{e}"))?;
        let delete = crate::activities::DeleteActivity {
            id: delete_id,
            kind: Default::default(),
            actor: ObjectId::from(local_actor.ap_id.clone()),
            object: serde_json::json!(ap_id.to_string()),
            to: vec![crate::urls::AS_PUBLIC.to_string()],
            cc: vec![local_actor.followers_url.to_string()],
        };
        let delete_with_ctx = WithContext::new_default(delete);
        let sends =
            SendActivityTask::prepare(&delete_with_ctx, &local_actor, inboxes, &data).await?;
        let failures = send_with_retry(sends, &data).await;
        if !failures.is_empty() {
            tracing::warn!(
                count = failures.len(),
                "some delete activity deliveries failed"
            );
        }
        Ok(())
    }

    /// Broadcast an Add(WatchlistObject) activity to all accepted followers.
    pub async fn broadcast_add_to_followers(
        &self,
        local_user_id: uuid::Uuid,
        ap_id: Url,
        object: serde_json::Value,
    ) -> anyhow::Result<()> {
        let data = self.federation_config.to_request_data();
        let Some((local_actor, inboxes)) =
            self.accepted_follower_inboxes(&data, local_user_id).await?
        else {
            return Ok(());
        };

        let add = crate::activities::AddActivity {
            id: ap_id,
            kind: Default::default(),
            actor: ObjectId::from(local_actor.ap_id.clone()),
            object,
            to: vec![crate::urls::AS_PUBLIC.to_string()],
            cc: vec![local_actor.followers_url.to_string()],
        };
        let add_with_ctx = WithContext::new_default(add);
        let sends = SendActivityTask::prepare(&add_with_ctx, &local_actor, inboxes, &data).await?;
        let failures = send_with_retry(sends, &data).await;
        if !failures.is_empty() {
            tracing::warn!(count = failures.len(), "some Add deliveries failed");
        }
        Ok(())
    }

    /// Broadcast an Undo(Add) activity to all accepted followers.
    pub async fn broadcast_undo_add_to_followers(
        &self,
        local_user_id: uuid::Uuid,
        watchlist_entry_ap_id: Url,
    ) -> anyhow::Result<()> {
        let data = self.federation_config.to_request_data();
        let Some((local_actor, inboxes)) =
            self.accepted_follower_inboxes(&data, local_user_id).await?
        else {
            return Ok(());
        };

        let undo_id =
            crate::urls::activity_url(&self.base_url).map_err(|e| anyhow::anyhow!("{e}"))?;
        let undo = crate::activities::UndoActivity {
            id: undo_id,
            kind: Default::default(),
            actor: ObjectId::from(local_actor.ap_id.clone()),
            object: serde_json::json!({
                "type": "Add",
                "id": watchlist_entry_ap_id.as_str(),
                "object": { "id": watchlist_entry_ap_id.as_str() }
            }),
        };
        let undo_with_ctx = WithContext::new_default(undo);
        let sends = SendActivityTask::prepare(&undo_with_ctx, &local_actor, inboxes, &data).await?;
        let failures = send_with_retry(sends, &data).await;
        if !failures.is_empty() {
            tracing::warn!(count = failures.len(), "some Undo(Add) deliveries failed");
        }
        Ok(())
    }

    pub async fn broadcast_actor_update(&self, user_id: uuid::Uuid) -> anyhow::Result<()> {
        use activitypub_federation::traits::Object;

        let data = self.federation_config.to_request_data();
        let local_actor = get_local_actor(user_id, &data)
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?;

        let person = local_actor
            .clone()
            .into_json(&data)
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        // Wrap with @context so Mastodon's JSON-LD processor can resolve field names.
        let person_json = serde_json::to_value(WithContext::new_default(person))?;

        let update_id = Url::parse(&format!(
            "{}/activities/update/{}",
            self.base_url,
            uuid::Uuid::new_v4()
        ))?;

        let update = UpdateActivity {
            id: update_id,
            kind: Default::default(),
            actor: ObjectId::from(local_actor.ap_id.clone()),
            object: person_json,
            to: vec![crate::urls::AS_PUBLIC.to_string()],
            cc: vec![local_actor.followers_url.to_string()],
        };

        let followers = data.federation_repo.get_followers(user_id).await?;
        let accepted: Vec<_> = followers
            .into_iter()
            .filter(|f| f.status == FollowerStatus::Accepted)
            .collect();

        if accepted.is_empty() {
            tracing::info!(user_id = %user_id, "no accepted followers, skipping actor update broadcast");
            return Ok(());
        }

        let inboxes = collect_inboxes(&accepted);
        tracing::info!(
            user_id = %user_id,
            follower_count = accepted.len(),
            inbox_count = inboxes.len(),
            inboxes = ?inboxes,
            "broadcasting actor update"
        );

        let sends = SendActivityTask::prepare(
            &WithContext::new_default(update),
            &local_actor,
            inboxes,
            &data,
        )
        .await?;

        let failures = send_with_retry(sends, &data).await;
        if !failures.is_empty() {
            return Err(anyhow::anyhow!(
                "actor update delivery failed for {} inbox(es): {}",
                failures.len(),
                failures
                    .iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join("; ")
            ));
        }
        tracing::info!(user_id = %user_id, "actor update broadcast complete");
        Ok(())
    }

    pub async fn block_actor(
        &self,
        local_user_id: uuid::Uuid,
        actor_url: &str,
    ) -> anyhow::Result<()> {
        let data = self.federation_config.to_request_data();

        data.federation_repo
            .add_blocked_actor(local_user_id, actor_url)
            .await?;
        let _ = data
            .federation_repo
            .remove_follower(local_user_id, actor_url)
            .await;
        let _ = data
            .federation_repo
            .remove_following(local_user_id, actor_url)
            .await;

        let local_actor = get_local_actor(local_user_id, &data)
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?;

        if let Ok(Some(remote_actor)) = data.federation_repo.get_remote_actor(actor_url).await {
            let block_id =
                crate::urls::activity_url(&self.base_url).map_err(|e| anyhow::anyhow!("{e}"))?;
            let block = crate::activities::BlockActivity {
                id: block_id,
                kind: Default::default(),
                actor: ObjectId::from(local_actor.ap_id.clone()),
                object: Url::parse(actor_url)?,
            };
            let inbox = Url::parse(&remote_actor.inbox_url)?;
            let sends = SendActivityTask::prepare(
                &WithContext::new_default(block),
                &local_actor,
                vec![inbox],
                &data,
            )
            .await?;
            let failures = send_with_retry(sends, &data).await;
            if !failures.is_empty() {
                tracing::warn!(actor = %actor_url, "failed to deliver Block activity");
            }
        }

        Ok(())
    }

    pub async fn unblock_actor(
        &self,
        local_user_id: uuid::Uuid,
        actor_url: &str,
    ) -> anyhow::Result<()> {
        let data = self.federation_config.to_request_data();
        data.federation_repo
            .remove_blocked_actor(local_user_id, actor_url)
            .await
    }

    pub async fn get_blocked_actors(
        &self,
        local_user_id: uuid::Uuid,
    ) -> anyhow::Result<Vec<RemoteActor>> {
        let data = self.federation_config.to_request_data();
        let actor_urls = data
            .federation_repo
            .get_blocked_actors(local_user_id)
            .await?;
        let mut actors = Vec::new();
        for url in actor_urls {
            let actor = match data.federation_repo.get_remote_actor(&url).await {
                Ok(Some(a)) => a,
                _ => RemoteActor {
                    url: url.clone(),
                    handle: url.clone(),
                    inbox_url: url.clone(),
                    shared_inbox_url: None,
                    display_name: None,
                    avatar_url: None,
                    outbox_url: None,
                },
            };
            actors.push(actor);
        }
        Ok(actors)
    }

    pub async fn add_blocked_domain(
        &self,
        domain: &str,
        reason: Option<&str>,
    ) -> anyhow::Result<()> {
        let data = self.federation_config.to_request_data();
        data.federation_repo
            .add_blocked_domain(domain, reason)
            .await
    }

    pub async fn remove_blocked_domain(&self, domain: &str) -> anyhow::Result<()> {
        let data = self.federation_config.to_request_data();
        data.federation_repo.remove_blocked_domain(domain).await
    }

    pub async fn get_blocked_domains(&self) -> anyhow::Result<Vec<BlockedDomain>> {
        let data = self.federation_config.to_request_data();
        data.federation_repo.get_blocked_domains().await
    }

    async fn follow_local(
        &self,
        local_user_id: uuid::Uuid,
        target_username: &str,
        data: &activitypub_federation::config::Data<FederationData>,
    ) -> anyhow::Result<()> {
        let target = data
            .user_repo
            .find_by_username(target_username)
            .await?
            .ok_or_else(|| anyhow::anyhow!("user not found: {}", target_username))?;

        if target.id == local_user_id {
            return Err(anyhow::anyhow!("cannot follow yourself"));
        }

        let follower_actor_url = crate::urls::actor_url(&self.base_url, local_user_id).to_string();
        let target_actor_url = crate::urls::actor_url(&self.base_url, target.id);
        let target_inbox_url = format!("{}/inbox", target_actor_url);
        let follow_id = activity_url(&self.base_url)
            .map_err(|e| anyhow::anyhow!("{e}"))?
            .to_string();

        data.federation_repo
            .add_follower(
                target.id,
                &follower_actor_url,
                FollowerStatus::Accepted,
                &follow_id,
            )
            .await?;

        let target_as_remote = RemoteActor {
            url: target_actor_url.to_string(),
            handle: format!("{}@{}", target.username, data.domain),
            inbox_url: target_inbox_url,
            shared_inbox_url: None,
            display_name: Some(target.username),
            avatar_url: None,
            outbox_url: None,
        };
        data.federation_repo
            .add_following(local_user_id, target_as_remote, &follow_id)
            .await?;

        data.federation_repo
            .update_following_status(
                local_user_id,
                target_actor_url.as_ref(),
                FollowingStatus::Accepted,
            )
            .await?;

        tracing::info!(follower = %local_user_id, followee = %target.id, "local follow");
        Ok(())
    }

    async fn unfollow_local(
        &self,
        local_user_id: uuid::Uuid,
        target_actor_url: &str,
        data: &activitypub_federation::config::Data<FederationData>,
    ) -> anyhow::Result<()> {
        let target_url = Url::parse(target_actor_url)?;
        let target_user_id = crate::urls::extract_user_id_from_url(&target_url)
            .ok_or_else(|| anyhow::anyhow!("invalid local actor URL: {}", target_actor_url))?;

        let local_actor_url = crate::urls::actor_url(&self.base_url, local_user_id).to_string();

        data.federation_repo
            .remove_follower(target_user_id, &local_actor_url)
            .await?;
        data.federation_repo
            .remove_following(local_user_id, target_actor_url)
            .await?;

        tracing::info!(follower = %local_user_id, followee = %target_user_id, "local unfollow");
        Ok(())
    }

    pub async fn backfill_outbox(&self, outbox_url: &str, actor_url: &str) -> anyhow::Result<()> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(HTTP_FETCH_TIMEOUT_SECS))
            .build()?;
        let data = self.federation_config.to_request_data();
        let actor = url::Url::parse(actor_url)?;

        let root: serde_json::Value = client
            .get(outbox_url)
            .header("Accept", "application/activity+json")
            .send()
            .await?
            .json()
            .await?;

        let first = match root.get("first").and_then(|v| v.as_str()) {
            Some(url) => url.to_string(),
            None => {
                tracing::debug!(outbox = %outbox_url, "outbox has no first page, nothing to backfill");
                return Ok(());
            }
        };

        let mut current_url = first;
        let mut visited = std::collections::HashSet::new();

        loop {
            if !visited.insert(current_url.clone()) {
                tracing::warn!(url = %current_url, "backfill: loop detected, stopping");
                break;
            }

            let page: serde_json::Value = match client
                .get(&current_url)
                .header("Accept", "application/activity+json")
                .send()
                .await
            {
                Ok(resp) => match resp.json().await {
                    Ok(v) => v,
                    Err(e) => {
                        tracing::error!(error = %e, url = %current_url, "backfill: failed to parse page JSON");
                        break;
                    }
                },
                Err(e) => {
                    tracing::error!(error = %e, url = %current_url, "backfill: HTTP request failed");
                    break;
                }
            };

            if let Some(items) = page.get("orderedItems").and_then(|v| v.as_array()) {
                for item in items {
                    let activity_type = item.get("type").and_then(|v| v.as_str()).unwrap_or("");
                    if activity_type != "Create" && activity_type != "Add" {
                        continue;
                    }
                    let object = match item.get("object") {
                        Some(o) if o.is_object() => o.clone(),
                        _ => continue,
                    };
                    let ap_id = match object
                        .get("id")
                        .and_then(|v| v.as_str())
                        .and_then(|s| url::Url::parse(s).ok())
                    {
                        Some(u) => u,
                        None => continue,
                    };
                    if let Err(e) = data.object_handler.on_create(&ap_id, &actor, object).await {
                        tracing::warn!(ap_id = %ap_id, error = %e, "backfill: failed to process item, skipping");
                    }
                }
            }

            match page.get("next").and_then(|v| v.as_str()) {
                Some(next) => current_url = next.to_string(),
                None => break,
            }
        }

        tracing::info!(outbox = %outbox_url, pages = visited.len(), "backfill complete");
        Ok(())
    }

    fn adapter_actor_to_domain(
        a: crate::repository::RemoteActor,
    ) -> domain::models::remote_actor::RemoteActor {
        domain::models::remote_actor::RemoteActor {
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

    fn spawn_backfill(&self, owner_user_id: uuid::Uuid, follower_inbox_url: String) {
        let config = self.federation_config.clone();
        let base_url = self.base_url.clone();
        tokio::spawn(async move {
            if let Err(e) = ActivityPubService::run_backfill(
                config,
                base_url,
                owner_user_id,
                follower_inbox_url,
            )
            .await
            {
                tracing::warn!(error = %e, "backfill: task failed");
            }
        });
    }

    async fn run_backfill(
        config: ApFederationConfig,
        base_url: String,
        owner_user_id: uuid::Uuid,
        follower_inbox_url: String,
    ) -> anyhow::Result<()> {
        const BATCH_SIZE: usize = 20;

        let data = config.to_request_data();
        let local_actor = get_local_actor(owner_user_id, &data)
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        let inbox = Url::parse(&follower_inbox_url)?;

        let mut objects = data
            .object_handler
            .get_local_objects_for_user(owner_user_id)
            .await?;
        objects.reverse(); // oldest first → chronological feed

        let total = objects.len();
        let mut success_count = 0usize;
        let mut failure_count = 0usize;

        for chunk in objects.chunks(BATCH_SIZE) {
            for (ap_id, object_json) in chunk {
                // Use a stable Create activity ID derived from the object's ap_id
                let create_id = Url::parse(&format!(
                    "{}/activities/create/{}",
                    base_url,
                    uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_URL, ap_id.as_str().as_bytes())
                ))?;

                let create = CreateActivity {
                    id: create_id,
                    kind: Default::default(),
                    actor: ObjectId::from(local_actor.ap_id.clone()),
                    object: object_json.clone(),
                    to: vec![],
                    cc: vec![],
                    bto: vec![],
                    bcc: vec![],
                };

                let sends = SendActivityTask::prepare(
                    &WithContext::new_default(create),
                    &local_actor,
                    vec![inbox.clone()],
                    &data,
                )
                .await?;
                let failures = send_with_retry(sends, &data).await;
                if failures.is_empty() {
                    success_count += 1;
                } else {
                    failure_count += 1;
                }
            }
            tokio::time::sleep(std::time::Duration::from_millis(BATCH_FETCH_SLEEP_MS)).await;
        }

        tracing::info!(
            user_id = %owner_user_id,
            follower = %follower_inbox_url,
            sent = success_count,
            failed = failure_count,
            total = total,
            "backfill complete"
        );
        Ok(())
    }
}

#[async_trait::async_trait]
impl crate::ap_ports::OutboundFederationPort for ActivityPubService {
    // Actor identity (ap_id, followers_url) comes from federation config via get_local_actor.
    // author_username is provided by the caller but not needed here.
    async fn broadcast_create(
        &self,
        author_user_id: &domain::value_objects::UserId,
        thought: &domain::models::thought::Thought,
        _author_username: &str,
        in_reply_to_url: Option<&str>,
    ) -> Result<(), domain::errors::DomainError> {
        let user_uuid = author_user_id.as_uuid();
        let data = self.federation_config.to_request_data();
        let Some((local_actor, inboxes)) =
            self.accepted_follower_inboxes(&data, user_uuid)
                .await
                .map_err(|e| domain::errors::DomainError::Internal(e.to_string()))?
        else {
            return Ok(());
        };

        let (ap_id, note) =
            thought_note_json(thought, &local_actor, &self.base_url, in_reply_to_url)
                .map_err(|e| domain::errors::DomainError::Internal(e.to_string()))?;

        let create = crate::activities::CreateActivity {
            id: ap_id,
            kind: Default::default(),
            actor: activitypub_federation::fetch::object_id::ObjectId::from(
                local_actor.ap_id.clone(),
            ),
            object: note,
            to: vec![crate::urls::AS_PUBLIC.to_string()],
            cc: vec![local_actor.followers_url.to_string()],
            bto: vec![],
            bcc: vec![],
        };
        let sends = activitypub_federation::activity_sending::SendActivityTask::prepare(
            &activitypub_federation::protocol::context::WithContext::new_default(create),
            &local_actor,
            inboxes,
            &data,
        )
        .await
        .map_err(|e| domain::errors::DomainError::Internal(e.to_string()))?;
        let failures = send_with_retry(sends, &data).await;
        if !failures.is_empty() {
            tracing::warn!(count = failures.len(), "some Create deliveries failed");
        }
        Ok(())
    }

    async fn broadcast_delete(
        &self,
        author_user_id: &domain::value_objects::UserId,
        thought_ap_id: &str,
    ) -> Result<(), domain::errors::DomainError> {
        let user_uuid = author_user_id.as_uuid();
        let ap_id = url::Url::parse(thought_ap_id)
            .map_err(|e| domain::errors::DomainError::Internal(e.to_string()))?;
        self.broadcast_delete_to_followers(user_uuid, ap_id)
            .await
            .map_err(|e| domain::errors::DomainError::Internal(e.to_string()))
    }

    // Actor identity (ap_id, followers_url) comes from federation config via get_local_actor.
    // author_username is provided by the caller but not needed here.
    async fn broadcast_update(
        &self,
        author_user_id: &domain::value_objects::UserId,
        thought: &domain::models::thought::Thought,
        _author_username: &str,
        in_reply_to_url: Option<&str>,
    ) -> Result<(), domain::errors::DomainError> {
        let user_uuid = author_user_id.as_uuid();
        let data = self.federation_config.to_request_data();
        let Some((local_actor, inboxes)) =
            self.accepted_follower_inboxes(&data, user_uuid)
                .await
                .map_err(|e| domain::errors::DomainError::Internal(e.to_string()))?
        else {
            return Ok(());
        };

        let (_ap_id, note) =
            thought_note_json(thought, &local_actor, &self.base_url, in_reply_to_url)
                .map_err(|e| domain::errors::DomainError::Internal(e.to_string()))?;

        let update_id = url::Url::parse(&format!(
            "{}/activities/update/{}",
            self.base_url,
            uuid::Uuid::new_v4()
        ))
        .map_err(|e| domain::errors::DomainError::Internal(e.to_string()))?;
        let update = crate::activities::UpdateActivity {
            id: update_id,
            kind: Default::default(),
            actor: activitypub_federation::fetch::object_id::ObjectId::from(
                local_actor.ap_id.clone(),
            ),
            object: note,
            to: vec![crate::urls::AS_PUBLIC.to_string()],
            cc: vec![local_actor.followers_url.to_string()],
        };
        let sends = activitypub_federation::activity_sending::SendActivityTask::prepare(
            &activitypub_federation::protocol::context::WithContext::new_default(update),
            &local_actor,
            inboxes,
            &data,
        )
        .await
        .map_err(|e| domain::errors::DomainError::Internal(e.to_string()))?;
        let failures = send_with_retry(sends, &data).await;
        if !failures.is_empty() {
            tracing::warn!(count = failures.len(), "some Update deliveries failed");
        }
        Ok(())
    }

    async fn broadcast_announce(
        &self,
        booster_user_id: &domain::value_objects::UserId,
        object_ap_id: &str,
    ) -> Result<(), domain::errors::DomainError> {
        let user_uuid = booster_user_id.as_uuid();
        let ap_id = url::Url::parse(object_ap_id)
            .map_err(|e| domain::errors::DomainError::Internal(e.to_string()))?;
        self.broadcast_announce_to_followers(user_uuid, ap_id)
            .await
            .map_err(|e| domain::errors::DomainError::Internal(e.to_string()))
    }

    async fn broadcast_undo_announce(
        &self,
        booster_user_id: &domain::value_objects::UserId,
        object_ap_id: &str,
    ) -> Result<(), domain::errors::DomainError> {
        let user_uuid = booster_user_id.as_uuid();
        let ap_id = url::Url::parse(object_ap_id)
            .map_err(|e| domain::errors::DomainError::Internal(e.to_string()))?;
        self.broadcast_undo_announce_to_followers(user_uuid, ap_id)
            .await
            .map_err(|e| domain::errors::DomainError::Internal(e.to_string()))
    }

    async fn broadcast_like(
        &self,
        liker_user_id: &domain::value_objects::UserId,
        object_ap_id: &str,
        author_inbox_url: &str,
    ) -> Result<(), domain::errors::DomainError> {
        let object = url::Url::parse(object_ap_id)
            .map_err(|e| domain::errors::DomainError::Internal(e.to_string()))?;
        let inbox = url::Url::parse(author_inbox_url)
            .map_err(|e| domain::errors::DomainError::Internal(e.to_string()))?;
        self.broadcast_like_to_inbox(liker_user_id.as_uuid(), object, inbox)
            .await
            .map_err(|e| domain::errors::DomainError::Internal(e.to_string()))
    }

    async fn broadcast_undo_like(
        &self,
        liker_user_id: &domain::value_objects::UserId,
        object_ap_id: &str,
        author_inbox_url: &str,
    ) -> Result<(), domain::errors::DomainError> {
        let object = url::Url::parse(object_ap_id)
            .map_err(|e| domain::errors::DomainError::Internal(e.to_string()))?;
        let inbox = url::Url::parse(author_inbox_url)
            .map_err(|e| domain::errors::DomainError::Internal(e.to_string()))?;
        self.broadcast_undo_like_to_inbox(liker_user_id.as_uuid(), object, inbox)
            .await
            .map_err(|e| domain::errors::DomainError::Internal(e.to_string()))
    }

    async fn broadcast_actor_update(
        &self,
        user_id: &domain::value_objects::UserId,
    ) -> Result<(), domain::errors::DomainError> {
        self.broadcast_actor_update(user_id.as_uuid())
            .await
            .map_err(|e| domain::errors::DomainError::Internal(e.to_string()))
    }
}

#[async_trait::async_trait]
impl domain::ports::FederationSchedulerPort for ActivityPubService {
    async fn schedule_actor_posts_fetch(
        &self,
        actor_ap_url: &str,
        outbox_url: &str,
    ) -> Result<(), domain::errors::DomainError> {
        let service = self.clone();
        let actor = actor_ap_url.to_string();
        let outbox = outbox_url.to_string();
        tokio::spawn(async move {
            if let Err(e) = service.backfill_outbox(&outbox, &actor).await {
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
    ) -> Result<(), domain::errors::DomainError> {
        // Only trigger a full fetch on page 1 to avoid redundant network traffic.
        if page != 1 {
            return Ok(());
        }
        let service = self.clone();
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

            // Walk the AP collection, following first/next links.
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

                // OrderedCollection root — follow its `first` page.
                if val["type"].as_str() == Some("OrderedCollection") {
                    current_url = val["first"].as_str().map(|s| s.to_string());
                    continue;
                }

                // Collect actor URLs from orderedItems (string or {id: ...}).
                let empty = vec![];
                let items = val["orderedItems"].as_array().unwrap_or(&empty);
                for item in items {
                    let actor_url = item
                        .as_str()
                        .or_else(|| item["id"].as_str())
                        .unwrap_or("");
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
                tracing::debug!(actor = %actor, connection_type = %conn_type, "connections: empty collection");
                return;
            }

            // Resolve profiles and cache in pages of PAGE_SIZE.
            const PAGE_SIZE: usize = 20;
            for (idx, chunk) in all_urls.chunks(PAGE_SIZE).enumerate() {
                let page_num = (idx + 1) as u32;
                let chunk_urls: Vec<String> = chunk.to_vec();
                let resolved = service.resolve_actor_profiles(chunk_urls).await;
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

#[async_trait::async_trait]
impl domain::ports::FederationLookupPort for ActivityPubService {
    async fn lookup_actor(
        &self,
        handle: &str,
    ) -> Result<domain::models::remote_actor::RemoteActor, domain::errors::DomainError> {
        use activitypub_federation::fetch::object_id::ObjectId;

        let normalized = handle.trim_start_matches('@');
        let at = normalized.rfind('@').ok_or_else(|| {
            domain::errors::DomainError::InvalidInput("handle must be user@domain".into())
        })?;
        let (user, domain_str) = (&normalized[..at], &normalized[at + 1..]);

        // Fetch WebFinger over HTTPS directly — the library's webfinger_resolve_actor
        // tries HTTP first in debug mode, which fails on servers without HTTP→HTTPS redirect.
        let wf_url = format!(
            "https://{}/.well-known/webfinger?resource=acct:{}@{}",
            domain_str, user, domain_str
        );
        let wf: serde_json::Value = reqwest::Client::new()
            .get(&wf_url)
            .header("Accept", "application/jrd+json, application/json")
            .send()
            .await
            .map_err(|e| domain::errors::DomainError::ExternalService(e.to_string()))?
            .json()
            .await
            .map_err(|e| domain::errors::DomainError::ExternalService(e.to_string()))?;

        let self_href = wf["links"]
            .as_array()
            .and_then(|links| {
                links.iter().find(|l| {
                    l["rel"].as_str() == Some("self")
                        && l["type"].as_str() == Some("application/activity+json")
                })
            })
            .and_then(|l| l["href"].as_str())
            .ok_or(domain::errors::DomainError::NotFound)?;

        let self_url = url::Url::parse(self_href)
            .map_err(|e| domain::errors::DomainError::ExternalService(e.to_string()))?;

        let data = self.federation_config.to_request_data();
        let actor: crate::actors::DbActor = ObjectId::from(self_url)
            .dereference(&data)
            .await
            .map_err(|e: crate::error::Error| {
                domain::errors::DomainError::ExternalService(e.to_string())
            })?;

        let domain_str = actor.ap_id.host_str().unwrap_or("");
        let full_handle = format!("{}@{}", actor.username, domain_str);

        Ok(domain::models::remote_actor::RemoteActor {
            url: actor.ap_id.to_string(),
            handle: full_handle,
            display_name: Some(actor.username.clone()),
            avatar_url: actor.avatar_url.as_ref().map(|u| u.to_string()),
            last_fetched_at: actor.last_refreshed_at,
            bio: actor.bio.clone(),
            banner_url: actor.banner_url.as_ref().map(|u| u.to_string()),
            also_known_as: actor.also_known_as.clone(),
            outbox_url: Some(actor.outbox_url.to_string()),
            followers_url: Some(actor.followers_url.to_string()),
            following_url: Some(actor.following_url.to_string()),
            attachment: actor
                .attachment
                .iter()
                .map(|f| (f.name.clone(), f.value.clone()))
                .collect(),
        })
    }

    async fn actor_json(
        &self,
        user_id: &domain::value_objects::UserId,
    ) -> Result<String, domain::errors::DomainError> {
        ActivityPubService::actor_json(self, &user_id.as_uuid().to_string())
            .await
            .map_err(|e| domain::errors::DomainError::ExternalService(e.to_string()))
    }

    async fn followers_collection_json(
        &self,
        user_id: &domain::value_objects::UserId,
        page: Option<u32>,
    ) -> Result<String, domain::errors::DomainError> {
        let data = self.federation_config.to_request_data();
        let uuid = user_id.as_uuid();
        let collection_id = format!("{}/users/{}/followers", self.base_url, uuid);
        let total = data
            .federation_repo
            .count_followers(uuid)
            .await
            .map_err(|e| domain::errors::DomainError::ExternalService(e.to_string()))?;
        let obj = if let Some(p) = page {
            let p = p.max(1);
            let offset = (p.saturating_sub(1) as usize) * crate::urls::AP_PAGE_SIZE;
            let followers = data
                .federation_repo
                .get_followers_page(uuid, offset as u32, crate::urls::AP_PAGE_SIZE)
                .await
                .map_err(|e| domain::errors::DomainError::ExternalService(e.to_string()))?;
            let has_next = offset + followers.len() < total;
            let items: Vec<String> = followers.into_iter().map(|f| f.actor.url).collect();
            let mut obj = serde_json::json!({
                "@context": crate::urls::AP_CONTEXT,
                "type": "OrderedCollectionPage",
                "id": format!("{}?page={}", collection_id, p),
                "partOf": collection_id,
                "totalItems": total,
                "orderedItems": items,
            });
            if has_next {
                obj["next"] = serde_json::json!(format!("{}?page={}", collection_id, p + 1));
            }
            obj
        } else {
            serde_json::json!({
                "@context": crate::urls::AP_CONTEXT,
                "type": "OrderedCollection",
                "id": collection_id,
                "totalItems": total,
                "first": format!("{}?page=1", collection_id),
            })
        };
        serde_json::to_string(&obj)
            .map_err(|e| domain::errors::DomainError::ExternalService(e.to_string()))
    }

    async fn following_collection_json(
        &self,
        user_id: &domain::value_objects::UserId,
        page: Option<u32>,
    ) -> Result<String, domain::errors::DomainError> {
        let data = self.federation_config.to_request_data();
        let uuid = user_id.as_uuid();
        let collection_id = format!("{}/users/{}/following", self.base_url, uuid);
        let total = data
            .federation_repo
            .count_following(uuid)
            .await
            .map_err(|e| domain::errors::DomainError::ExternalService(e.to_string()))?;
        let obj = if let Some(p) = page {
            let p = p.max(1);
            let offset = (p.saturating_sub(1) as usize) * crate::urls::AP_PAGE_SIZE;
            let following = data
                .federation_repo
                .get_following_page(uuid, offset as u32, crate::urls::AP_PAGE_SIZE)
                .await
                .map_err(|e| domain::errors::DomainError::ExternalService(e.to_string()))?;
            let has_next = offset + following.len() < total;
            let items: Vec<String> = following.into_iter().map(|a| a.url).collect();
            let mut obj = serde_json::json!({
                "@context": crate::urls::AP_CONTEXT,
                "type": "OrderedCollectionPage",
                "id": format!("{}?page={}", collection_id, p),
                "partOf": collection_id,
                "totalItems": total,
                "orderedItems": items,
            });
            if has_next {
                obj["next"] = serde_json::json!(format!("{}?page={}", collection_id, p + 1));
            }
            obj
        } else {
            serde_json::json!({
                "@context": crate::urls::AP_CONTEXT,
                "type": "OrderedCollection",
                "id": collection_id,
                "totalItems": total,
                "first": format!("{}?page=1", collection_id),
            })
        };
        serde_json::to_string(&obj)
            .map_err(|e| domain::errors::DomainError::ExternalService(e.to_string()))
    }
}

#[async_trait::async_trait]
impl domain::ports::FederationFetchPort for ActivityPubService {
    async fn fetch_outbox_page(
        &self,
        outbox_url: &str,
        page: u32,
    ) -> Result<Vec<domain::models::remote_note::RemoteNote>, domain::errors::DomainError> {
        use chrono::DateTime;

        // Fetch the base outbox to find the real first-page URL.
        // Mastodon uses ?page=true; other servers may use ?page=1 or a different param.
        let client = reqwest::Client::new();
        let base: serde_json::Value = client
            .get(outbox_url)
            .header("Accept", "application/activity+json, application/ld+json")
            .send()
            .await
            .map_err(|e| domain::errors::DomainError::ExternalService(e.to_string()))?
            .json()
            .await
            .map_err(|e| domain::errors::DomainError::ExternalService(e.to_string()))?;

        // Prefer the `first` link from the OrderedCollection; fall back to ?page=1.
        let url = base["first"]
            .as_str()
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("{}?page={}", outbox_url, page));

        let resp: serde_json::Value = client
            .get(&url)
            .header("Accept", "application/activity+json, application/ld+json")
            .send()
            .await
            .map_err(|e| domain::errors::DomainError::ExternalService(e.to_string()))?
            .json()
            .await
            .map_err(|e| domain::errors::DomainError::ExternalService(e.to_string()))?;

        let empty = vec![];
        let items = resp["orderedItems"].as_array().unwrap_or(&empty);

        let notes = items
            .iter()
            .filter_map(|item| {
                // Items are Create activities wrapping a Note, or Notes directly
                let note = if item["type"].as_str() == Some("Create") {
                    &item["object"]
                } else if item["type"].as_str() == Some("Note") {
                    item
                } else {
                    return None;
                };

                // Only public notes
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
    ) -> Result<Vec<String>, domain::errors::DomainError> {
        let client = reqwest::Client::new();
        let base: serde_json::Value = client
            .get(collection_url)
            .header("Accept", "application/activity+json, application/ld+json")
            .send()
            .await
            .map_err(|e| domain::errors::DomainError::ExternalService(e.to_string()))?
            .json()
            .await
            .map_err(|e| domain::errors::DomainError::ExternalService(e.to_string()))?;

        // Base collections typically have no orderedItems — follow the `first` page link.
        let page = if base["orderedItems"].is_null() {
            if let Some(first_url) = base["first"].as_str() {
                client
                    .get(first_url)
                    .header("Accept", "application/activity+json, application/ld+json")
                    .send()
                    .await
                    .map_err(|e| domain::errors::DomainError::ExternalService(e.to_string()))?
                    .json()
                    .await
                    .map_err(|e| domain::errors::DomainError::ExternalService(e.to_string()))?
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
}

#[async_trait::async_trait]
impl domain::ports::FederationFollowPort for ActivityPubService {
    async fn follow_remote(
        &self,
        local_user_id: &domain::value_objects::UserId,
        handle: &str,
    ) -> Result<(), domain::errors::DomainError> {
        self.follow(local_user_id.as_uuid(), handle)
            .await
            .map_err(|e| domain::errors::DomainError::ExternalService(e.to_string()))
    }

    async fn unfollow_remote(
        &self,
        local_user_id: &domain::value_objects::UserId,
        handle: &str,
    ) -> Result<(), domain::errors::DomainError> {
        let data = self.federation_config.to_request_data();
        let remote_actor: DbActor = Self::webfinger_https(handle, &data)
            .await
            .map_err(|e| domain::errors::DomainError::ExternalService(e.to_string()))?;
        let actor_url = remote_actor.ap_id.to_string();
        self.unfollow(local_user_id.as_uuid(), &actor_url)
            .await
            .map_err(|e| domain::errors::DomainError::ExternalService(e.to_string()))
    }

    async fn get_remote_following(
        &self,
        user_id: &domain::value_objects::UserId,
    ) -> Result<Vec<domain::models::remote_actor::RemoteActor>, domain::errors::DomainError> {
        self.get_following(user_id.as_uuid())
            .await
            .map(|v| v.into_iter().map(Self::adapter_actor_to_domain).collect())
            .map_err(|e| domain::errors::DomainError::ExternalService(e.to_string()))
    }
}

#[async_trait::async_trait]
impl domain::ports::FederationFollowRequestPort for ActivityPubService {
    async fn get_pending_followers(
        &self,
        user_id: &domain::value_objects::UserId,
    ) -> Result<Vec<domain::models::remote_actor::RemoteActor>, domain::errors::DomainError> {
        self.get_pending_followers(user_id.as_uuid())
            .await
            .map(|v| v.into_iter().map(Self::adapter_actor_to_domain).collect())
            .map_err(|e| domain::errors::DomainError::ExternalService(e.to_string()))
    }

    async fn accept_follow_request(
        &self,
        user_id: &domain::value_objects::UserId,
        actor_url: &str,
    ) -> Result<(), domain::errors::DomainError> {
        self.accept_follower(user_id.as_uuid(), actor_url)
            .await
            .map_err(|e| domain::errors::DomainError::ExternalService(e.to_string()))
    }

    async fn reject_follow_request(
        &self,
        user_id: &domain::value_objects::UserId,
        actor_url: &str,
    ) -> Result<(), domain::errors::DomainError> {
        self.reject_follower(user_id.as_uuid(), actor_url)
            .await
            .map_err(|e| domain::errors::DomainError::ExternalService(e.to_string()))
    }

    async fn get_remote_followers(
        &self,
        user_id: &domain::value_objects::UserId,
    ) -> Result<Vec<domain::models::remote_actor::RemoteActor>, domain::errors::DomainError> {
        self.get_accepted_followers(user_id.as_uuid())
            .await
            .map(|v| v.into_iter().map(Self::adapter_actor_to_domain).collect())
            .map_err(|e| domain::errors::DomainError::ExternalService(e.to_string()))
    }

    async fn remove_remote_follower(
        &self,
        user_id: &domain::value_objects::UserId,
        actor_url: &str,
    ) -> Result<(), domain::errors::DomainError> {
        self.remove_follower(user_id.as_uuid(), actor_url)
            .await
            .map_err(|e| domain::errors::DomainError::ExternalService(e.to_string()))
    }
}

#[cfg(test)]
#[path = "tests/service.rs"]
mod tests;
