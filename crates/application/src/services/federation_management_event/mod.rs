use domain::{errors::DomainError, events::DomainEvent, ports::FederationActionPort};
use std::sync::Arc;

pub struct FederationManagementEventService {
    pub federation: Arc<dyn FederationActionPort>,
}

impl FederationManagementEventService {
    pub async fn process(&self, event: &DomainEvent) -> Result<(), DomainError> {
        match event {
            DomainEvent::RemoteFollowAccepted {
                local_user_id,
                remote_actor_url,
            } => {
                tracing::info!(
                    local_user_id = %local_user_id,
                    actor = %remote_actor_url,
                    "federation-mgmt: accepting follow — sending Accept + backfill"
                );
                self.federation
                    .accept_follow_request(local_user_id, remote_actor_url)
                    .await
            }
            DomainEvent::RemoteFollowRejected {
                local_user_id,
                remote_actor_url,
            } => {
                tracing::info!(
                    local_user_id = %local_user_id,
                    actor = %remote_actor_url,
                    "federation-mgmt: rejecting follow — sending Reject"
                );
                self.federation
                    .reject_follow_request(local_user_id, remote_actor_url)
                    .await
            }
            DomainEvent::ActorMoved {
                user_id,
                new_actor_url,
            } => {
                tracing::info!(
                    user_id = %user_id,
                    target = %new_actor_url,
                    "federation-mgmt: broadcasting Move"
                );
                let url = url::Url::parse(new_actor_url)
                    .map_err(|e| DomainError::Internal(e.to_string()))?;
                self.federation
                    .broadcast_move(user_id, url)
                    .await
                    .map_err(|e| DomainError::Internal(e.to_string()))
            }
            _ => Ok(()),
        }
    }
}
