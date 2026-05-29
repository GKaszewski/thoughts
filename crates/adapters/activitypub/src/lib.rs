pub mod handler;
pub mod note;
pub mod port;
pub mod service;
pub mod urls;

pub use handler::ThoughtsObjectHandler;
pub use note::ThoughtNote;
pub use port::{
    AcceptNoteInput, ActivityPubRepository, ActorApUrls, OutboundFederationPort, OutboxEntry,
};
pub use service::ApFederationAdapter;
pub use urls::ThoughtsUrls;

use domain::ports::RemoteActorConnectionRepository;
use k_ap::ActivityPubService;
use std::sync::Arc;

pub struct ApServiceConfig {
    pub base_url: String,
    pub activity_repo: Arc<dyn k_ap::ActivityRepository>,
    pub follow_repo: Arc<dyn k_ap::FollowRepository>,
    pub actor_repo: Arc<dyn k_ap::ActorRepository>,
    pub blocklist_repo: Arc<dyn k_ap::BlocklistRepository>,
    pub user_repo: Arc<dyn k_ap::ApUserRepository>,
    pub ap_handler: Arc<ThoughtsObjectHandler>,
    pub connections_repo: Arc<dyn RemoteActorConnectionRepository>,
    pub event_publisher: Option<Arc<dyn k_ap::data::EventPublisher>>,
    pub allow_registration: bool,
    pub debug: bool,
}

pub async fn build_ap_service(
    cfg: ApServiceConfig,
) -> (Arc<ActivityPubService>, Arc<ApFederationAdapter>) {
    let mut builder = ActivityPubService::builder(cfg.base_url)
        .activity_repo(cfg.activity_repo)
        .follow_repo(cfg.follow_repo)
        .actor_repo(cfg.actor_repo)
        .blocklist_repo(cfg.blocklist_repo)
        .user_repo(cfg.user_repo)
        .content_reader(cfg.ap_handler.clone())
        .object_handler(cfg.ap_handler)
        .allow_registration(cfg.allow_registration)
        .software_name("thoughts")
        .debug(cfg.debug);
    if let Some(publisher) = cfg.event_publisher {
        builder = builder.event_publisher(publisher);
    }
    let raw = Arc::new(
        builder
            .build()
            .await
            .expect("Failed to build ActivityPubService"),
    );
    let adapter = Arc::new(ApFederationAdapter::new(raw.clone(), cfg.connections_repo));
    (raw, adapter)
}
