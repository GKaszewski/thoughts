# Architecture

Hexagonal (ports & adapters) architecture. Dependencies point inward — adapters implement domain ports, application orchestrates use cases, presentation handles HTTP.

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         ENTRY POINTS                                    │
│                                                                         │
│  ┌─────────────┐          ┌──────────┐                                  │
│  │  bootstrap   │          │  worker   │                                 │
│  │  (HTTP srv)  │          │ (bg jobs) │                                 │
│  └──────┬───────┘          └─────┬─────┘                                │
│         │ wires all deps         │ consumes events                      │
└─────────┼────────────────────────┼──────────────────────────────────────┘
          │                        │
          ▼                        ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                      INTERFACE LAYER                                    │
│                                                                         │
│  ┌──────────────────┐    ┌────────────┐                                 │
│  │  presentation     │    │  api-types  │                                │
│  │  (axum handlers,  │◄───│  (DTOs,     │                                │
│  │   extractors,     │    │   OpenAPI)  │                                │
│  │   AppState)       │    └────────────┘                                 │
│  └────────┬──────────┘                                                  │
│           │ calls use cases                                             │
└───────────┼─────────────────────────────────────────────────────────────┘
            │
            ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                     APPLICATION LAYER                                   │
│                                                                         │
│  ┌─────────────────────────────────────────────┐                        │
│  │  application                                 │                       │
│  │                                              │                       │
│  │  use_cases/                                  │                       │
│  │    thoughts, social, profile,                │                       │
│  │    federation_management                     │                       │
│  │                                              │                       │
│  │  services/                                   │                       │
│  │    FederationEventService                    │                       │
│  │    (processes domain events → broadcasts)    │                       │
│  └──────────────────┬──────────────────────────┘                        │
│                     │ depends only on domain ports                      │
└─────────────────────┼───────────────────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                       DOMAIN LAYER (core)                               │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │  domain                                                         │    │
│  │                                                                 │    │
│  │  models/         value_objects/       events/        errors/    │    │
│  │   Thought         UserId              DomainEvent    DomainErr │    │
│  │   User            ThoughtId           EventEnvelope            │    │
│  │   RemoteActor     Username                                     │    │
│  │   Follow,Like     Content                                      │    │
│  │   Boost,Block     Email                                        │    │
│  │   Notification    PasswordHash                                 │    │
│  │   Tag,FeedEntry                                                │    │
│  │                                                                 │    │
│  │  ports/ (trait interfaces)                                      │    │
│  │   UserRepository          FederationContentRepository           │    │
│  │   ThoughtRepository       FederationBroadcastPort               │    │
│  │   LikeRepository          FederationLookupPort                  │    │
│  │   BoostRepository         FederationFollowPort                  │    │
│  │   FollowRepository        FederationFollowRequestPort           │    │
│  │   BlockRepository         FederationFetchPort                   │    │
│  │   TagRepository           FederationBlockPort                   │    │
│  │   NotificationRepository  FederationSchedulerPort               │    │
│  │   FeedRepository          FederationActionPort (supertrait)     │    │
│  │   SearchPort              EventPublisher / EventConsumer        │    │
│  │   AuthService             MediaStore / OutboxWriter             │    │
│  │   PasswordHasher          RemoteActorConnectionRepository       │    │
│  │   ApiKeyRepository        EngagementRepository                  │    │
│  └─────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────┘
                      ▲
                      │ implements ports
                      │
┌─────────────────────────────────────────────────────────────────────────┐
│                      ADAPTER LAYER                                      │
│                                                                         │
│  ┌──────────────────┐  ┌─────────────────────┐  ┌──────────────────┐   │
│  │  postgres         │  │  activitypub         │  │  auth             │  │
│  │                   │  │                      │  │                   │  │
│  │  UserRepository   │  │  FederationActionPort│  │  AuthService      │  │
│  │  ThoughtRepo      │  │  FederationBroadcast │  │  ApiKeyService    │  │
│  │  LikeRepo         │  │  FederationContent.. │  └──────────────────┘  │
│  │  BoostRepo        │  │  FederationScheduler │                        │
│  │  FollowRepo       │  │                      │  ┌──────────────────┐  │
│  │  BlockRepo        │  │  (wraps k-ap lib)    │  │  storage          │  │
│  │  TagRepo          │  └─────────────────────┘  │  MediaStore       │  │
│  │  NotificationRepo │                            └──────────────────┘  │
│  │  ApiKeyRepo       │  ┌─────────────────────┐                        │
│  │  TopFriendRepo    │  │  postgres-federation │  ┌──────────────────┐  │
│  │  FeedRepo         │  │  (k-ap DB traits)    │  │  nats             │  │
│  │  EngagementRepo   │  └─────────────────────┘  │  EventPublisher   │  │
│  │  FederationContent│                            │  EventConsumer    │  │
│  │  OutboxWriter     │  ┌─────────────────────┐  └──────────────────┘  │
│  │  RemoteActorRepo  │  │  postgres-search     │                        │
│  │  RemoteActorConn  │  │  SearchPort          │  ┌──────────────────┐  │
│  └──────────────────┘  └─────────────────────┘  │  event-transport   │  │
│                                                   │  event-payload    │  │
│                                                   │  (serialization)  │  │
│                                                   └──────────────────┘  │
└─────────────────────────────────────────────────────────────────────────┘
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
