use application::services::{FederationEventService, NotificationEventService};
use domain::{errors::DomainError, events::DomainEvent};
use std::sync::Arc;

pub struct NotificationHandler {
    pub service: Arc<NotificationEventService>,
}

impl NotificationHandler {
    pub async fn handle(&self, event: &DomainEvent) -> Result<(), DomainError> {
        self.service.process(event).await
    }
}

pub struct FederationHandler {
    pub service: Arc<FederationEventService>,
}

impl FederationHandler {
    pub async fn handle(&self, event: &DomainEvent) -> Result<(), DomainError> {
        self.service.process(event).await
    }
}
