use activitypub::ActivityPubRepository;
use domain::ports::*;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub users: Arc<dyn UserRepository>,
    pub thoughts: Arc<dyn ThoughtRepository>,
    pub likes: Arc<dyn LikeRepository>,
    pub boosts: Arc<dyn BoostRepository>,
    pub follows: Arc<dyn FollowRepository>,
    pub blocks: Arc<dyn BlockRepository>,
    pub tags: Arc<dyn TagRepository>,
    pub api_keys: Arc<dyn ApiKeyRepository>,
    pub api_key_auth: Arc<dyn ApiKeyService>,
    pub top_friends: Arc<dyn TopFriendRepository>,
    pub notifications: Arc<dyn NotificationRepository>,
    pub remote_actors: Arc<dyn RemoteActorRepository>,
    pub feed: Arc<dyn FeedRepository>,
    pub search: Arc<dyn SearchPort>,
    pub auth: Arc<dyn AuthService>,
    pub hasher: Arc<dyn PasswordHasher>,
    pub events: Arc<dyn EventPublisher>,
    pub outbox: Arc<dyn OutboxWriter>,
    pub federation: Arc<dyn FederationActionPort>,
    pub ap_repo: Arc<dyn ActivityPubRepository>,
    pub remote_actor_connections: Arc<dyn RemoteActorConnectionRepository>,
    pub federation_scheduler: Arc<dyn FederationSchedulerPort>,
    pub engagement: Arc<dyn EngagementRepository>,
}
