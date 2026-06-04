# Architecture

Hexagonal (ports & adapters) architecture. Dependencies point inward — adapters implement domain ports, application orchestrates use cases, presentation handles HTTP.

## Crate dependency graph

```mermaid
graph TD
    subgraph Entry Points
        bootstrap["bootstrap<br/><small>HTTP server, DI wiring</small>"]
        worker["worker<br/><small>background job consumer</small>"]
    end

    subgraph Interface Layer
        presentation["presentation<br/><small>axum handlers, extractors, AppState</small>"]
        api_types["api-types<br/><small>DTOs, OpenAPI</small>"]
    end

    subgraph Application Layer
        application["application<br/><small>use cases, FederationEventService</small>"]
    end

    subgraph Domain Layer
        domain["domain<br/><small>models, value objects, events, port traits</small>"]
    end

    subgraph Adapters
        postgres["postgres<br/><small>UserRepo, ThoughtRepo, LikeRepo,<br/>BoostRepo, FollowRepo, BlockRepo,<br/>TagRepo, FeedRepo, FederationContentRepo, ...</small>"]
        activitypub["activitypub<br/><small>FederationActionPort,<br/>FederationBroadcastPort,<br/>FederationSchedulerPort<br/>(wraps k-ap)</small>"]
        postgres_fed["postgres-federation<br/><small>k-ap DB traits</small>"]
        postgres_search["postgres-search<br/><small>SearchPort</small>"]
        auth["auth<br/><small>AuthService, ApiKeyService</small>"]
        nats["nats<br/><small>EventPublisher, EventConsumer</small>"]
        storage["storage<br/><small>MediaStore</small>"]
        event_transport["event-transport<br/><small>event delivery</small>"]
        event_payload["event-payload<br/><small>event serialization</small>"]
    end

    bootstrap --> presentation
    bootstrap --> application
    bootstrap --> postgres
    bootstrap --> postgres_fed
    bootstrap --> postgres_search
    bootstrap --> activitypub
    bootstrap --> auth
    bootstrap --> nats
    bootstrap --> storage
    bootstrap --> event_transport
    bootstrap --> event_payload

    worker --> application
    worker --> activitypub
    worker --> postgres
    worker --> postgres_fed
    worker --> nats
    worker --> event_transport
    worker --> event_payload

    presentation --> application
    presentation --> api_types
    presentation --> domain

    application --> domain

    postgres --> domain
    activitypub --> domain
    postgres_fed -.-> domain
    postgres_search --> domain
    postgres_search --> postgres
    auth --> domain
    nats --> domain
    storage --> domain
    event_transport --> domain
    event_payload --> domain
```

## Domain ports

```mermaid
classDiagram
    class domain {
        <<core>>
    }

    namespace Data Ports {
        class UserRepository {
            <<trait>>
            find_by_id()
            find_by_username()
            save()
            update_profile()
        }
        class ThoughtRepository {
            <<trait>>
            save()
            find_by_id()
            delete()
            update_content()
        }
        class LikeRepository { <<trait>> }
        class BoostRepository { <<trait>> }
        class FollowRepository { <<trait>> }
        class BlockRepository { <<trait>> }
        class TagRepository { <<trait>> }
        class FeedRepository { <<trait>> }
        class NotificationRepository { <<trait>> }
        class EngagementRepository { <<trait>> }
        class SearchPort { <<trait>> }
    }

    namespace Federation Ports {
        class FederationContentRepository {
            <<trait>>
            outbox_entries_for_actor()
            find_remote_actor_id()
            intern_remote_actor()
            accept_note()
            retract_note()
        }
        class FederationBroadcastPort {
            <<trait>>
            broadcast_create()
            broadcast_delete()
            broadcast_update()
            broadcast_announce()
            broadcast_like()
        }
        class FederationActionPort {
            <<supertrait>>
        }
        class FederationLookupPort { <<trait>> }
        class FederationFollowPort { <<trait>> }
        class FederationFollowRequestPort { <<trait>> }
        class FederationFetchPort { <<trait>> }
        class FederationBlockPort { <<trait>> }
        class FederationSchedulerPort { <<trait>> }
    }

    namespace Infra Ports {
        class EventPublisher { <<trait>> }
        class EventConsumer { <<trait>> }
        class AuthService { <<trait>> }
        class PasswordHasher { <<trait>> }
        class MediaStore { <<trait>> }
    }

    FederationActionPort --|> FederationLookupPort
    FederationActionPort --|> FederationFollowPort
    FederationActionPort --|> FederationFollowRequestPort
    FederationActionPort --|> FederationFetchPort
    FederationActionPort --|> FederationBlockPort
```

## Dependency rule

```
bootstrap/worker ──► presentation ──► application ──► domain ◄── adapters
```

- **domain** — zero framework deps, pure business logic, defines all port traits
- **application** — orchestrates use cases, depends only on domain
- **presentation** — HTTP handlers (axum), depends on domain + application
- **adapters** — implement domain ports, depend inward on domain only
- **bootstrap/worker** — composition roots, wire adapters into ports
