use postgres::failed_event::PgFailedEventStore;
use postgres::remote_actor_connections::PgRemoteActorConnectionRepository;
use sqlx::PgPool;
use std::sync::Arc;

use activitypub::ThoughtsObjectHandler;
use activitypub::{
    build_ap_service, ActivityPubRepository, ApServiceConfig, OutboundFederationPort,
};
use application::services::{
    FederationEventService, FederationManagementEventService, NotificationEventService,
};
use domain::ports::EventPublisher;
use postgres::activitypub::PgActivityPubRepository;
use postgres_federation::{PgApUserRepository, PgFederationRepository};

use crate::handlers::{FederationHandler, FederationManagementHandler, NotificationHandler};

pub struct WorkerHandlers {
    pub notification: NotificationHandler,
    pub federation: FederationHandler,
    pub federation_management: FederationManagementHandler,
}

pub struct WorkerInfra {
    pub pool: PgPool,
    pub message_source: nats::NatsMessageSource,
    pub handlers: WorkerHandlers,
    pub dlq_store: Arc<PgFailedEventStore>,
    pub event_publisher: Arc<dyn EventPublisher>,
    pub raw_ap_service: Arc<k_ap::ActivityPubService>,
}

pub async fn build(database_url: &str, base_url: &str, nats_url: &str) -> WorkerInfra {
    let pool = PgPool::connect(database_url)
        .await
        .expect("DB connect failed");

    // Repos
    let thoughts = Arc::new(postgres::thought::PgThoughtRepository::new(pool.clone()));
    let users = Arc::new(postgres::user::PgUserRepository::new(pool.clone()));
    let notifications = Arc::new(postgres::notification::PgNotificationRepository::new(
        pool.clone(),
    ));

    // ActivityPub service (for federation fan-out)
    let connections_repo_worker = Arc::new(PgRemoteActorConnectionRepository::new(pool.clone()));
    let fed_repo_worker = Arc::new(PgFederationRepository::new(pool.clone()));
    let ap_handler_worker = Arc::new(ThoughtsObjectHandler::new(
        Arc::new(PgActivityPubRepository::new(pool.clone())),
        base_url,
        None,
        Arc::new(postgres::tag::PgTagRepository::new(pool.clone())),
        Arc::new(postgres::like::PgLikeRepository::new(pool.clone())),
        Arc::new(postgres::boost::PgBoostRepository::new(pool.clone())),
    ));
    let (raw_ap_service, ap_service) = build_ap_service(ApServiceConfig {
        base_url: base_url.to_string(),
        activity_repo: fed_repo_worker.clone(),
        follow_repo: fed_repo_worker.clone(),
        actor_repo: fed_repo_worker.clone(),
        blocklist_repo: fed_repo_worker.clone(),
        user_repo: Arc::new(PgApUserRepository::new(pool.clone(), base_url.to_string())),
        ap_handler: ap_handler_worker,
        connections_repo: connections_repo_worker,
        event_publisher: None,
        allow_registration: false,
        debug: false,
    })
    .await;
    let ap_outbound = ap_service.clone() as Arc<dyn OutboundFederationPort>;
    let ap_repo_worker =
        Arc::new(PgActivityPubRepository::new(pool.clone())) as Arc<dyn ActivityPubRepository>;

    // Application services
    let notification_svc = Arc::new(NotificationEventService {
        thoughts: thoughts.clone(),
        notifications,
    });
    let federation_svc = Arc::new(FederationEventService {
        thoughts,
        users,
        ap: ap_outbound,
        base_url: base_url.to_string(),
        ap_repo: ap_repo_worker,
    });
    let federation_management_svc = Arc::new(FederationManagementEventService {
        federation: ap_service.clone() as Arc<dyn domain::ports::FederationActionPort>,
    });

    // Thin handlers
    let handlers = WorkerHandlers {
        notification: NotificationHandler {
            service: notification_svc,
        },
        federation: FederationHandler {
            service: federation_svc,
        },
        federation_management: FederationManagementHandler {
            service: federation_management_svc,
        },
    };

    // DLQ store
    let dlq_store = Arc::new(PgFailedEventStore::new(pool.clone()));

    // NATS consumer + publisher
    let nats_client = async_nats::connect(nats_url)
        .await
        .expect("NATS connect failed");
    nats::ensure_stream(&nats_client)
        .await
        .expect("JetStream stream setup failed");
    let message_source = nats::NatsMessageSource::new(nats_client.clone());
    let event_publisher: Arc<dyn EventPublisher> = Arc::new(
        event_transport::EventPublisherAdapter::new(nats::NatsTransport::new(nats_client)),
    );

    WorkerInfra {
        pool,
        message_source,
        handlers,
        dlq_store,
        event_publisher,
        raw_ap_service,
    }
}
