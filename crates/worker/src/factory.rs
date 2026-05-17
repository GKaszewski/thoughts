use postgres::failed_event::PgFailedEventStore;
use postgres::remote_actor_connections::PgRemoteActorConnectionRepository;
use sqlx::PgPool;
use std::sync::Arc;

use activitypub::{ApFederationAdapter, ThoughtsObjectHandler};
use activitypub::{ActivityPubRepository, OutboundFederationPort};
use k_ap::ActivityPubService;
use application::services::{FederationEventService, NotificationEventService};
use domain::ports::EventPublisher;
use postgres::activitypub::PgActivityPubRepository;
use postgres_federation::{PostgresApUserRepository, PostgresFederationRepository};

use crate::handlers::{FederationHandler, NotificationHandler};

pub struct WorkerHandlers {
    pub notification: NotificationHandler,
    pub federation: FederationHandler,
}

pub struct WorkerInfra {
    pub pool: PgPool,
    pub consumer: event_transport::EventConsumerAdapter<nats::NatsMessageSource>,
    pub handlers: WorkerHandlers,
    pub dlq_store: Arc<PgFailedEventStore>,
    pub event_publisher: Arc<dyn EventPublisher>,
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
    let connections_repo_worker =
        Arc::new(PgRemoteActorConnectionRepository::new(pool.clone()));
    let raw_ap_service = Arc::new(
        ActivityPubService::builder(
            Arc::new(PostgresFederationRepository::new(pool.clone())),
            Arc::new(PostgresApUserRepository::new(
                pool.clone(),
                base_url.to_string(),
            )),
            Arc::new(ThoughtsObjectHandler::new(
                Arc::new(PgActivityPubRepository::new(pool.clone())),
                base_url,
                None,
                Arc::new(postgres::tag::PgTagRepository::new(pool.clone())),
            )),
            base_url,
        )
        .software_name("thoughts")
        .build()
        .await
        .expect("ActivityPubService build failed"),
    );
    let ap_service = Arc::new(ApFederationAdapter::new(raw_ap_service, connections_repo_worker));
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

    // Thin handlers
    let handlers = WorkerHandlers {
        notification: NotificationHandler {
            service: notification_svc,
        },
        federation: FederationHandler {
            service: federation_svc,
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
    let consumer = event_transport::EventConsumerAdapter::new(nats::NatsMessageSource::new(
        nats_client.clone(),
    ));
    let event_publisher: Arc<dyn EventPublisher> = Arc::new(
        event_transport::EventPublisherAdapter::new(nats::NatsTransport::new(nats_client)),
    );

    WorkerInfra {
        pool,
        consumer,
        handlers,
        dlq_store,
        event_publisher,
    }
}
