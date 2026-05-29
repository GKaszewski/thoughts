const JWT_TTL_SECS: i64 = 86_400; // 24 hours (was 30 days)
const JWT_SECRET_MIN_BYTES: usize = 32; // 256 bits minimum for HS256

use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;

use application::use_cases::profile::UploadConfig;
use storage::{build_store, ObjectStorageAdapter, StorageConfig};

use activitypub::{build_ap_service, ApFederationAdapter, ApServiceConfig, ThoughtsObjectHandler};
use auth::ApiKeyServiceImpl;
use domain::{
    errors::DomainError,
    events::DomainEvent,
    ports::{EventPublisher, OutboxWriter},
};
use event_transport::{EventPublisherAdapter, Transport};
use k_ap::FederationEvent;
use nats::NatsTransport;
use postgres::activitypub::PgActivityPubRepository;
use postgres::engagement::PgEngagementRepository;
use postgres::outbox::PgOutboxWriter;
use postgres::remote_actor_connections::PgRemoteActorConnectionRepository;
use postgres_federation::{PgApUserRepository, PgFederationRepository};
use presentation::state::AppState;

use crate::config::Config;

/// Everything the binary needs to start serving.
pub struct Infrastructure {
    pub state: AppState,
    pub ap_service: Arc<ApFederationAdapter>,
}

struct NoOpEventPublisher;

#[async_trait]
impl EventPublisher for NoOpEventPublisher {
    async fn publish(&self, _e: &DomainEvent) -> Result<(), DomainError> {
        Ok(())
    }
}

struct KapPublisher(NatsTransport);

#[async_trait]
impl k_ap::data::EventPublisher for KapPublisher {
    async fn publish(&self, event: FederationEvent) -> anyhow::Result<()> {
        let (subject, payload) = match event {
            FederationEvent::DeliveryRequested {
                inbox,
                activity,
                signing_actor_id,
            } => (
                "federation.delivery.requested",
                serde_json::to_vec(&event_payload::EventPayload::FederationDeliveryRequested {
                    inbox: inbox.to_string(),
                    activity,
                    signing_actor_id: signing_actor_id.to_string(),
                })?,
            ),
            FederationEvent::BackfillRequested {
                owner_user_id,
                follower_inbox_url,
            } => (
                "federation.backfill.requested",
                serde_json::to_vec(&event_payload::EventPayload::FederationBackfillRequested {
                    owner_user_id: owner_user_id.to_string(),
                    follower_inbox_url,
                })?,
            ),
            FederationEvent::DeliveryFailed { inbox, error, .. } => {
                tracing::warn!(%inbox, %error, "AP delivery failed permanently");
                return Ok(());
            }
        };
        self.0
            .publish_bytes(subject, &payload)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }
}

pub async fn build(cfg: &Config) -> Infrastructure {
    // 1. Database connection + migrations
    let pool = PgPool::connect(&cfg.database_url)
        .await
        .expect("Failed to connect to database");
    sqlx::migrate!("../adapters/postgres/migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");
    tracing::info!("Database connected and migrations applied");

    // 2. Event publisher — real NATS or no-op fallback
    let nats_client: Option<async_nats::Client> = match &cfg.nats_url {
        Some(url) => match async_nats::connect(url).await {
            Ok(client) => {
                tracing::info!("Connected to NATS at {url}");
                if let Err(e) = nats::ensure_stream(&client).await {
                    tracing::warn!("JetStream stream setup failed: {e} — events may be lost");
                }
                Some(client)
            }
            Err(e) => {
                tracing::warn!("NATS connect failed ({e}) — falling back to no-op publisher");
                None
            }
        },
        None => {
            tracing::info!("NATS_URL not set — using no-op event publisher");
            None
        }
    };
    let event_publisher: Arc<dyn EventPublisher> = match &nats_client {
        Some(client) => Arc::new(EventPublisherAdapter::new(NatsTransport::new(
            client.clone(),
        ))),
        None => Arc::new(NoOpEventPublisher),
    };
    let kap_publisher: Option<Arc<dyn k_ap::data::EventPublisher>> = nats_client
        .as_ref()
        .map(|c| Arc::new(KapPublisher(NatsTransport::new(c.clone()))) as _);

    // 3. ActivityPub federation
    let connections_repo = Arc::new(PgRemoteActorConnectionRepository::new(pool.clone()));
    let fed_repo = Arc::new(PgFederationRepository::new(pool.clone()));
    let likes: Arc<dyn domain::ports::LikeRepository> =
        Arc::new(postgres::like::PgLikeRepository::new(pool.clone()));
    let boosts: Arc<dyn domain::ports::BoostRepository> =
        Arc::new(postgres::boost::PgBoostRepository::new(pool.clone()));
    let ap_handler = Arc::new(ThoughtsObjectHandler::new(
        Arc::new(PgActivityPubRepository::new(pool.clone())),
        &cfg.base_url,
        Some(event_publisher.clone()),
        Arc::new(postgres::tag::PgTagRepository::new(pool.clone())),
        likes.clone(),
        boosts.clone(),
    ));
    let (_raw, ap_service) = build_ap_service(ApServiceConfig {
        base_url: cfg.base_url.clone(),
        activity_repo: fed_repo.clone(),
        follow_repo: fed_repo.clone(),
        actor_repo: fed_repo.clone(),
        blocklist_repo: fed_repo.clone(),
        user_repo: Arc::new(PgApUserRepository::new(pool.clone(), cfg.base_url.clone())),
        ap_handler,
        connections_repo,
        event_publisher: kap_publisher,
        allow_registration: cfg.allow_registration,
        debug: cfg.debug,
    })
    .await;

    // 4. Storage adapter
    let storage_cfg = StorageConfig {
        backend: cfg.storage_backend.clone(),
        local_path: cfg.storage_path.clone(),
        s3_endpoint: cfg.s3_endpoint.clone(),
        s3_access_key_id: cfg.s3_access_key_id.clone(),
        s3_secret_access_key: cfg.s3_secret_access_key.clone(),
        s3_bucket: cfg.s3_bucket.clone(),
        s3_region: cfg.s3_region.clone(),
    };
    let object_store = build_store(&storage_cfg).expect("Failed to build object store");
    let media_adapter: Arc<dyn domain::ports::MediaStore> = Arc::new(
        ObjectStorageAdapter::new(object_store, cfg.storage_prefix.clone())
            .expect("Failed to create storage adapter"),
    );
    let upload_config = UploadConfig {
        max_bytes: cfg.upload_max_bytes,
        allowed_content_types: cfg.upload_allowed_types.clone(),
    };

    // 5. Application state
    let state = AppState {
        users: Arc::new(postgres::user::PgUserRepository::new(pool.clone())),
        thoughts: Arc::new(postgres::thought::PgThoughtRepository::new(pool.clone())),
        likes: likes.clone(),
        boosts: boosts.clone(),
        follows: Arc::new(postgres::follow::PgFollowRepository::new(pool.clone())),
        blocks: Arc::new(postgres::block::PgBlockRepository::new(pool.clone())),
        tags: Arc::new(postgres::tag::PgTagRepository::new(pool.clone())),
        api_keys: Arc::new(postgres::api_key::PgApiKeyRepository::new(pool.clone())),
        top_friends: Arc::new(postgres::top_friend::PgTopFriendRepository::new(
            pool.clone(),
        )),
        notifications: Arc::new(postgres::notification::PgNotificationRepository::new(
            pool.clone(),
        )),
        remote_actors: Arc::new(postgres::remote_actor::PgRemoteActorRepository::new(
            pool.clone(),
        )),
        feed: Arc::new(postgres::feed::PgFeedRepository::new(pool.clone())),
        search: Arc::new(postgres_search::PgSearchRepository::new(pool.clone())),
        auth: Arc::new({
            if cfg.jwt_secret.len() < JWT_SECRET_MIN_BYTES {
                panic!(
                    "JWT_SECRET is {} bytes — minimum is {} bytes for HS256 security",
                    cfg.jwt_secret.len(),
                    JWT_SECRET_MIN_BYTES,
                );
            }
            auth::JwtAuthService::new(cfg.jwt_secret.clone(), JWT_TTL_SECS)
        }),
        hasher: Arc::new(auth::Argon2PasswordHasher),
        events: event_publisher,
        outbox: Arc::new(PgOutboxWriter::new(pool.clone())) as Arc<dyn OutboxWriter>,
        federation: ap_service.clone() as Arc<dyn domain::ports::FederationActionPort>,
        ap_repo: Arc::new(PgActivityPubRepository::new(pool.clone())),
        remote_actor_connections: Arc::new(PgRemoteActorConnectionRepository::new(pool.clone())),
        federation_scheduler: ap_service.clone() as Arc<dyn domain::ports::FederationSchedulerPort>,
        api_key_auth: Arc::new(ApiKeyServiceImpl::new(Arc::new(
            postgres::api_key::PgApiKeyRepository::new(pool.clone()),
        ))),
        engagement: Arc::new(PgEngagementRepository::new(pool.clone())),
        media: media_adapter,
        upload_config,
        base_url: cfg.base_url.clone(),
    };

    Infrastructure { state, ap_service }
}
