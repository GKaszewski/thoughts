const JWT_TTL_SECS: i64 = 86_400; // 24 hours (was 30 days)
const JWT_SECRET_MIN_BYTES: usize = 32; // 256 bits minimum for HS256

use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;

use activitypub::ThoughtsObjectHandler;
use activitypub_base::service::ActivityPubService;
use auth::ApiKeyServiceImpl;
use domain::{errors::DomainError, events::DomainEvent, ports::{EventPublisher, OutboxWriter}};
use event_transport::EventPublisherAdapter;
use nats::NatsTransport;
use postgres::activitypub::PgActivityPubRepository;
use postgres::engagement::PgEngagementRepository;
use postgres::outbox::PgOutboxWriter;
use postgres::remote_actor_connections::PgRemoteActorConnectionRepository;
use postgres_federation::{PostgresApUserRepository, PostgresFederationRepository};
use presentation::state::AppState;

use crate::config::Config;

/// Everything the binary needs to start serving.
pub struct Infrastructure {
    pub state: AppState,
    pub ap_service: Arc<ActivityPubService>,
}

struct NoOpEventPublisher;

#[async_trait]
impl EventPublisher for NoOpEventPublisher {
    async fn publish(&self, _e: &DomainEvent) -> Result<(), DomainError> {
        Ok(())
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
    let event_publisher: Arc<dyn EventPublisher> = match &cfg.nats_url {
        Some(url) => match async_nats::connect(url).await {
            Ok(client) => {
                tracing::info!("Connected to NATS at {url}");
                if let Err(e) = nats::ensure_stream(&client).await {
                    tracing::warn!("JetStream stream setup failed: {e} — events may be lost");
                }
                Arc::new(EventPublisherAdapter::new(NatsTransport::new(client)))
            }
            Err(e) => {
                tracing::warn!("NATS connect failed ({e}) — falling back to no-op publisher");
                Arc::new(NoOpEventPublisher)
            }
        },
        None => {
            tracing::info!("NATS_URL not set — using no-op event publisher");
            Arc::new(NoOpEventPublisher)
        }
    };

    // 3. ActivityPub federation
    let ap_service = Arc::new(
        ActivityPubService::new(
            Arc::new(PostgresFederationRepository::new(pool.clone())),
            Arc::new(PostgresApUserRepository::new(
                pool.clone(),
                cfg.base_url.clone(),
            )),
            Arc::new(ThoughtsObjectHandler::new(
                Arc::new(PgActivityPubRepository::new(pool.clone())),
                &cfg.base_url,
                Some(event_publisher.clone()),
                Arc::new(postgres::tag::PgTagRepository::new(pool.clone())),
            )),
            cfg.base_url.clone(),
            cfg.allow_registration,
            "thoughts".to_string(),
            cfg.debug,
            None,
        )
        .await
        .expect("Failed to build ActivityPubService"),
    );

    // 4. Application state
    let state = AppState {
        users: Arc::new(postgres::user::PgUserRepository::new(pool.clone())),
        thoughts: Arc::new(postgres::thought::PgThoughtRepository::new(pool.clone())),
        likes: Arc::new(postgres::like::PgLikeRepository::new(pool.clone())),
        boosts: Arc::new(postgres::boost::PgBoostRepository::new(pool.clone())),
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
        api_key_auth: Arc::new(ApiKeyServiceImpl::new(
            Arc::new(postgres::api_key::PgApiKeyRepository::new(pool.clone())),
        )),
        engagement: Arc::new(PgEngagementRepository::new(pool.clone())),
    };

    Infrastructure { state, ap_service }
}
