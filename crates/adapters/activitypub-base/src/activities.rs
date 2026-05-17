use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::activity::{
        AcceptType, CreateType, DeleteType, FollowType, RejectType, UndoType, UpdateType,
    },
    protocol::verification::verify_domains_match,
    traits::Activity,
};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(rename = "Announce")]
pub struct AnnounceType;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename = "Like")]
pub struct LikeType;

impl Default for LikeType {
    fn default() -> Self {
        Self
    }
}

use crate::actors::DbActor;
use crate::data::FederationData;
use crate::error::Error;
use crate::repository::{FollowerStatus, FollowingStatus};

// --- Follow ---

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FollowActivity {
    pub(crate) id: Url,
    #[serde(rename = "type", default)]
    pub(crate) kind: FollowType,
    pub(crate) actor: ObjectId<DbActor>,
    pub(crate) object: ObjectId<DbActor>,
}

#[async_trait::async_trait]
impl Activity for FollowActivity {
    type DataType = FederationData;
    type Error = Error;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    async fn verify(&self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let target_url = self.object.inner();
        let target_domain = match (target_url.host_str(), target_url.port()) {
            (Some(host), Some(port)) => format!("{}:{}", host, port),
            (Some(host), None) => host.to_string(),
            _ => {
                return Err(Error::bad_request(anyhow::anyhow!(
                    "invalid follow target URL"
                )));
            }
        };
        if target_domain != data.domain {
            return Err(Error::bad_request(anyhow::anyhow!(
                "follow target is not a local actor"
            )));
        }
        Ok(())
    }

    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let domain = self.actor().host_str().unwrap_or("");
        if data.federation_repo.is_domain_blocked(domain).await? {
            tracing::info!(actor = %self.actor(), "ignoring activity from blocked domain");
            return Ok(());
        }
        let _follower = self.actor.dereference(data).await?;
        let local_actor = self.object.dereference(data).await?;

        if data
            .federation_repo
            .is_actor_blocked(local_actor.user_id, self.actor.inner().as_str())
            .await?
        {
            tracing::info!(actor = %self.actor.inner(), "ignoring follow from blocked actor");
            return Ok(());
        }

        data.federation_repo
            .add_follower(
                local_actor.user_id,
                self.actor.inner().as_str(),
                FollowerStatus::Pending,
                self.id.as_str(),
            )
            .await?;

        tracing::info!(
            follower = %self.actor.inner(),
            local_user = %local_actor.user_id,
            "follow request pending approval"
        );
        Ok(())
    }
}

// --- Accept ---

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceptActivity {
    pub(crate) id: Url,
    #[serde(rename = "type", default)]
    pub(crate) kind: AcceptType,
    pub(crate) actor: ObjectId<DbActor>,
    pub(crate) object: FollowActivity,
}

#[async_trait::async_trait]
impl Activity for AcceptActivity {
    type DataType = FederationData;
    type Error = Error;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    async fn verify(&self, _data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        if self.actor.inner() != self.object.object.inner() {
            return Err(Error::bad_request(anyhow::anyhow!(
                "Accept actor does not match Follow target"
            )));
        }
        Ok(())
    }

    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let domain = self.actor().host_str().unwrap_or("");
        if data.federation_repo.is_domain_blocked(domain).await? {
            tracing::info!(actor = %self.actor(), "ignoring activity from blocked domain");
            return Ok(());
        }
        let local_user_id = crate::urls::extract_user_id_from_url(self.object.actor.inner())
            .ok_or_else(|| Error::bad_request(anyhow::anyhow!("invalid actor URL in Follow")))?;
        data.federation_repo
            .update_following_status(
                local_user_id,
                self.actor.inner().as_str(),
                FollowingStatus::Accepted,
            )
            .await?;

        tracing::info!(remote_actor = %self.actor.inner(), "follow accepted by remote");
        Ok(())
    }
}

// --- Reject ---

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RejectActivity {
    pub(crate) id: Url,
    #[serde(rename = "type", default)]
    pub(crate) kind: RejectType,
    pub(crate) actor: ObjectId<DbActor>,
    pub(crate) object: FollowActivity,
}

#[async_trait::async_trait]
impl Activity for RejectActivity {
    type DataType = FederationData;
    type Error = Error;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    async fn verify(&self, _data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        if self.actor.inner() != self.object.object.inner() {
            return Err(Error::bad_request(anyhow::anyhow!(
                "Reject actor does not match Follow target"
            )));
        }
        Ok(())
    }

    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let domain = self.actor().host_str().unwrap_or("");
        if data.federation_repo.is_domain_blocked(domain).await? {
            tracing::info!(actor = %self.actor(), "ignoring activity from blocked domain");
            return Ok(());
        }
        if let Some(user_id) = crate::urls::extract_user_id_from_url(self.object.actor.inner()) {
            data.federation_repo
                .remove_following(user_id, self.actor.inner().as_str())
                .await?;
        }
        tracing::info!(actor = %self.actor.inner(), "follow rejected");
        Ok(())
    }
}

// --- Undo ---

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoActivity {
    pub(crate) id: Url,
    #[serde(rename = "type", default)]
    pub(crate) kind: UndoType,
    pub(crate) actor: ObjectId<DbActor>,
    pub(crate) object: serde_json::Value,
}

#[async_trait::async_trait]
impl Activity for UndoActivity {
    type DataType = FederationData;
    type Error = Error;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    async fn verify(&self, _data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        // The actor undoing must be the same as the actor in the wrapped activity.
        if let Some(inner_actor) = self.object.get("actor").and_then(|v| v.as_str()) {
            if inner_actor != self.actor.inner().as_str() {
                return Err(Error::bad_request(anyhow::anyhow!(
                    "Undo actor does not match inner activity actor"
                )));
            }
        }
        Ok(())
    }

    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let domain = self.actor().host_str().unwrap_or("");
        if data.federation_repo.is_domain_blocked(domain).await? {
            tracing::info!(actor = %self.actor(), "ignoring Undo from blocked domain");
            return Ok(());
        }

        let obj_type = self
            .object
            .get("type")
            .and_then(|t| t.as_str())
            .unwrap_or("");

        match obj_type {
            "Follow" => {
                if let Some(obj_url) = self.object.get("object").and_then(|o| o.as_str())
                    && let Ok(url) = Url::parse(obj_url)
                    && let Some(user_id) = crate::urls::extract_user_id_from_url(&url)
                {
                    data.federation_repo
                        .remove_follower(user_id, self.actor.inner().as_str())
                        .await?;
                }
                data.object_handler
                    .on_actor_removed(self.actor.inner())
                    .await
                    .map_err(|e| Error::from(anyhow::anyhow!(e)))?;
                tracing::info!(actor = %self.actor.inner(), "unfollowed");
            }
            "Add" => {
                let ap_id_str = self
                    .object
                    .get("object")
                    .and_then(|o| o.get("id"))
                    .and_then(|id| id.as_str())
                    .or_else(|| self.object.get("id").and_then(|id| id.as_str()));

                if let Some(ap_id_str) = ap_id_str
                    && let Ok(ap_id) = Url::parse(ap_id_str)
                {
                    data.object_handler
                        .on_delete(&ap_id, self.actor.inner())
                        .await
                        .map_err(|e| Error::from(anyhow::anyhow!(e)))?;
                    tracing::info!(ap_id = %ap_id_str, "undo Add (watchlist remove)");
                }
            }
            "Like" => {
                if let Some(obj_url_str) = self.object.get("object").and_then(|o| o.as_str())
                    && let Ok(obj_url) = Url::parse(obj_url_str)
                    && obj_url.host_str().unwrap_or("") == data.domain
                {
                    data.object_handler
                        .on_unlike(&obj_url, self.actor.inner())
                        .await
                        .unwrap_or_else(|e| {
                            tracing::warn!(error = %e, "failed to process unlike");
                        });
                }
                tracing::info!(actor = %self.actor.inner(), "received Undo(Like)");
            }
            other => {
                tracing::debug!(kind = %other, "ignoring Undo of unknown activity type");
            }
        }

        Ok(())
    }
}

// --- Create ---

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateActivity {
    pub(crate) id: Url,
    #[serde(rename = "type", default)]
    pub(crate) kind: CreateType,
    pub(crate) actor: ObjectId<DbActor>,
    pub(crate) object: serde_json::Value,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub(crate) to: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub(crate) cc: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub(crate) bto: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub(crate) bcc: Vec<String>,
}

#[async_trait::async_trait]
impl Activity for CreateActivity {
    type DataType = FederationData;
    type Error = Error;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    async fn verify(&self, _data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        if let Some(attributed_to) = self.object.get("attributedTo").and_then(|v| v.as_str())
            && let Ok(attributed_url) = Url::parse(attributed_to)
            && &attributed_url != self.actor.inner()
        {
            return Err(Error::bad_request(anyhow::anyhow!(
                "Create actor does not match object attributedTo"
            )));
        }
        Ok(())
    }

    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let domain = self.actor().host_str().unwrap_or("");
        if data.federation_repo.is_domain_blocked(domain).await? {
            tracing::info!(actor = %self.actor(), "ignoring activity from blocked domain");
            return Ok(());
        }
        // Use the Note's own id, not the Create activity id (which ends in /activity).
        // Delete activities reference the Note id, so they must match.
        let ap_id = self
            .object
            .get("id")
            .and_then(|v| v.as_str())
            .and_then(|s| Url::parse(s).ok())
            .unwrap_or_else(|| self.id.clone());
        let actor_url = self.actor.inner().clone();
        data.object_handler
            .on_create(&ap_id, &actor_url, self.object)
            .await
            .map_err(|e| Error::from(anyhow::anyhow!(e)))?;
        tracing::info!(actor = %actor_url, "received create activity");
        Ok(())
    }
}

// --- Delete ---

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteActivity {
    pub(crate) id: Url,
    #[serde(rename = "type", default)]
    pub(crate) kind: DeleteType,
    pub(crate) actor: ObjectId<DbActor>,
    pub(crate) object: serde_json::Value,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub(crate) to: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub(crate) cc: Vec<String>,
}

#[async_trait::async_trait]
impl Activity for DeleteActivity {
    type DataType = FederationData;
    type Error = Error;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    async fn verify(&self, _data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let actor_domain = self.actor.inner().host_str().unwrap_or("");
        let object_domain = match &self.object {
            serde_json::Value::String(s) => Url::parse(s)
                .ok()
                .and_then(|u| u.host_str().map(|h| h.to_string()))
                .unwrap_or_default(),
            serde_json::Value::Object(o) => o
                .get("id")
                .and_then(|v| v.as_str())
                .and_then(|s| Url::parse(s).ok())
                .and_then(|u| u.host_str().map(|h| h.to_string()))
                .unwrap_or_default(),
            _ => String::new(),
        };
        if !object_domain.is_empty() && actor_domain != object_domain {
            return Err(Error::bad_request(anyhow::anyhow!(
                "Delete actor domain does not match object domain"
            )));
        }
        Ok(())
    }

    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let domain = self.actor().host_str().unwrap_or("");
        if data.federation_repo.is_domain_blocked(domain).await? {
            tracing::info!(actor = %self.actor(), "ignoring activity from blocked domain");
            return Ok(());
        }
        let actor_url = self.actor.inner().clone();

        // Extract object URL — handles plain string and Tombstone {"id":"...","type":"Tombstone"}
        let object_url_str = match &self.object {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Object(o) => o
                .get("id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_default(),
            _ => String::new(),
        };
        let Ok(object_url) = Url::parse(&object_url_str) else {
            tracing::warn!(actor = %actor_url, "Delete activity has unparseable object, ignoring");
            return Ok(());
        };

        // Actor self-deletion: Mastodon sends Delete(actor_url) when an account is deleted.
        if object_url == *self.actor.inner() {
            data.object_handler
                .on_actor_removed(&actor_url)
                .await
                .map_err(|e| Error::from(anyhow::anyhow!(e)))?;
            tracing::info!(actor = %actor_url, "received Delete(actor) — remote account deleted");
            return Ok(());
        }

        // Normal note deletion.
        data.object_handler
            .on_delete(&object_url, &actor_url)
            .await
            .map_err(|e| Error::from(anyhow::anyhow!(e)))?;
        tracing::info!(object = %object_url, "received Delete(note)");
        Ok(())
    }
}

// --- Update ---

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateActivity {
    pub(crate) id: Url,
    #[serde(rename = "type", default)]
    pub(crate) kind: UpdateType,
    pub(crate) actor: ObjectId<DbActor>,
    pub(crate) object: serde_json::Value,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub(crate) to: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub(crate) cc: Vec<String>,
}

#[async_trait::async_trait]
impl Activity for UpdateActivity {
    type DataType = FederationData;
    type Error = Error;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    async fn verify(&self, _data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        if let Some(attributed_to) = self.object.get("attributedTo").and_then(|v| v.as_str())
            && let Ok(attributed_url) = Url::parse(attributed_to)
            && &attributed_url != self.actor.inner()
        {
            return Err(Error::bad_request(anyhow::anyhow!(
                "Update actor does not match object attributedTo"
            )));
        }
        Ok(())
    }

    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let domain = self.actor().host_str().unwrap_or("");
        if data.federation_repo.is_domain_blocked(domain).await? {
            tracing::info!(actor = %self.actor(), "ignoring activity from blocked domain");
            return Ok(());
        }
        let ap_id = self
            .object
            .get("id")
            .and_then(|v| v.as_str())
            .and_then(|s| Url::parse(s).ok())
            .unwrap_or_else(|| self.id.clone());
        let actor_url = self.actor.inner().clone();
        data.object_handler
            .on_update(&ap_id, &actor_url, self.object)
            .await
            .map_err(|e| Error::from(anyhow::anyhow!(e)))?;
        tracing::info!(actor = %actor_url, "received update activity");
        Ok(())
    }
}

// --- Announce ---

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnounceActivity {
    pub(crate) id: Url,
    #[serde(rename = "type", default)]
    pub(crate) kind: AnnounceType,
    pub(crate) actor: ObjectId<DbActor>,
    pub(crate) object: Url,
    pub(crate) published: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub(crate) to: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub(crate) cc: Vec<String>,
}

#[async_trait::async_trait]
impl Activity for AnnounceActivity {
    type DataType = FederationData;
    type Error = Error;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    async fn verify(&self, _data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        verify_domains_match(&self.id, self.actor.inner())?;
        Ok(())
    }

    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let domain = self.actor().host_str().unwrap_or("");
        if data.federation_repo.is_domain_blocked(domain).await? {
            tracing::info!(actor = %self.actor(), "ignoring activity from blocked domain");
            return Ok(());
        }
        let object_domain = self.object.host_str().unwrap_or("");
        if object_domain != data.domain {
            tracing::debug!(
                actor = %self.actor.inner(),
                object = %self.object,
                "received Announce of non-local object — skipped (cross-server boost not supported)"
            );
            return Ok(());
        }
        data.federation_repo
            .add_announce(
                self.id.as_str(),
                self.object.as_str(),
                self.actor.inner().as_str(),
                self.published.unwrap_or_else(chrono::Utc::now),
            )
            .await?;
        data.object_handler
            .on_announce_received(&self.object, self.actor.inner())
            .await
            .unwrap_or_else(|e| {
                tracing::warn!(error = %e, "failed to process announce notification");
            });
        tracing::info!(actor = %self.actor.inner(), object = %self.object, "received announce");
        Ok(())
    }
}

// --- Like ---

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LikeActivity {
    pub id: Url,
    #[serde(rename = "type")]
    pub kind: LikeType,
    pub actor: ObjectId<DbActor>,
    pub object: Url,
}

#[async_trait::async_trait]
impl Activity for LikeActivity {
    type DataType = FederationData;
    type Error = crate::error::Error;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    async fn verify(&self, _data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        verify_domains_match(&self.id, self.actor.inner())?;
        Ok(())
    }

    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let domain = self.actor().host_str().unwrap_or("");
        if data.federation_repo.is_domain_blocked(domain).await? {
            tracing::info!(actor = %self.actor(), "ignoring Like from blocked domain");
            return Ok(());
        }

        // Only process if the liked object is on our instance.
        if self.object.host_str().unwrap_or("") != data.domain {
            return Ok(());
        }

        data.object_handler
            .on_like(&self.object, self.actor.inner())
            .await
            .map_err(|e| crate::error::Error::from(anyhow::anyhow!(e)))?;

        tracing::info!(actor = %self.actor.inner(), object = %self.object, "received like");
        Ok(())
    }
}

// --- Add ---

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(rename = "Add")]
pub struct AddType;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddActivity {
    pub(crate) id: Url,
    #[serde(rename = "type", default)]
    pub(crate) kind: AddType,
    pub(crate) actor: ObjectId<DbActor>,
    pub(crate) object: serde_json::Value,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub(crate) to: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub(crate) cc: Vec<String>,
}

#[async_trait::async_trait]
impl Activity for AddActivity {
    type DataType = FederationData;
    type Error = Error;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    async fn verify(&self, _data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        if let Some(attributed_to) = self.object.get("attributedTo").and_then(|v| v.as_str())
            && let Ok(attributed_url) = Url::parse(attributed_to)
            && &attributed_url != self.actor.inner()
        {
            return Err(Error::bad_request(anyhow::anyhow!(
                "Add actor does not match object attributedTo"
            )));
        }
        Ok(())
    }

    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let domain = self.actor().host_str().unwrap_or("");
        if data.federation_repo.is_domain_blocked(domain).await? {
            tracing::info!(actor = %self.actor(), "ignoring Add from blocked domain");
            return Ok(());
        }
        let ap_id = self.id.clone();
        let actor_url = self.actor.inner().clone();
        data.object_handler
            .on_create(&ap_id, &actor_url, self.object)
            .await
            .map_err(|e| Error::from(anyhow::anyhow!(e)))?;
        tracing::info!(actor = %actor_url, "received Add activity");
        Ok(())
    }
}

// --- Block ---

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(rename = "Block")]
pub struct BlockType;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockActivity {
    pub(crate) id: Url,
    #[serde(rename = "type", default)]
    pub(crate) kind: BlockType,
    pub(crate) actor: ObjectId<DbActor>,
    pub(crate) object: Url,
}

#[async_trait::async_trait]
impl Activity for BlockActivity {
    type DataType = FederationData;
    type Error = Error;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    async fn verify(&self, _data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        verify_domains_match(&self.id, self.actor.inner())?;
        Ok(())
    }

    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let domain = self.actor().host_str().unwrap_or("");
        if data.federation_repo.is_domain_blocked(domain).await? {
            tracing::info!(actor = %self.actor(), "ignoring activity from blocked domain");
            return Ok(());
        }
        if let Some(local_user_id) = crate::urls::extract_user_id_from_url(&self.object) {
            let _ = data
                .federation_repo
                .remove_following(local_user_id, self.actor.inner().as_str())
                .await;
            let _ = data
                .federation_repo
                .remove_follower(local_user_id, self.actor.inner().as_str())
                .await;
        }
        tracing::info!(actor = %self.actor.inner(), "received block — removed following and follower");
        Ok(())
    }
}

// --- Move (account migration) ---

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(rename = "Move")]
pub struct MoveType;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveActivity {
    pub(crate) id: Url,
    #[serde(rename = "type", default)]
    pub(crate) kind: MoveType,
    pub(crate) actor: ObjectId<DbActor>,
    pub(crate) object: Url,
    pub(crate) target: Url,
}

#[async_trait::async_trait]
impl Activity for MoveActivity {
    type DataType = FederationData;
    type Error = Error;

    fn id(&self) -> &Url {
        &self.id
    }
    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    async fn verify(&self, _data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        if &self.object != self.actor.inner() {
            return Err(Error::bad_request(anyhow::anyhow!(
                "Move object must be the actor itself"
            )));
        }
        Ok(())
    }

    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let domain = self.actor().host_str().unwrap_or("");
        if data.federation_repo.is_domain_blocked(domain).await? {
            return Ok(());
        }
        tracing::info!(
            actor = %self.actor.inner(),
            target = %self.target,
            "received Move (account migration) — target noted"
        );
        Ok(())
    }
}

// --- Inbox dispatch enum ---

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
#[enum_delegate::implement(Activity)]
pub enum InboxActivities {
    #[serde(rename = "Follow")]
    Follow(FollowActivity),
    #[serde(rename = "Accept")]
    Accept(AcceptActivity),
    #[serde(rename = "Reject")]
    Reject(RejectActivity),
    #[serde(rename = "Undo")]
    Undo(UndoActivity),
    #[serde(rename = "Create")]
    Create(CreateActivity),
    #[serde(rename = "Delete")]
    Delete(DeleteActivity),
    #[serde(rename = "Update")]
    Update(UpdateActivity),
    #[serde(rename = "Announce")]
    Announce(AnnounceActivity),
    #[serde(rename = "Add")]
    Add(AddActivity),
    #[serde(rename = "Block")]
    Block(BlockActivity),
    #[serde(rename = "Like")]
    Like(LikeActivity),
    #[serde(rename = "Move")]
    Move(MoveActivity),
}
