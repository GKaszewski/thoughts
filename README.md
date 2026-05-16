# Thoughts

A self-hosted microblogging server with full ActivityPub federation. Write short posts, follow people on Mastodon and other Fediverse servers, and receive their posts in your feed. Built in Rust with a Next.js frontend.

## Features

- Short-form posts (thoughts) with replies, boosts, and likes
- Full ActivityPub federation — follow/unfollow remote actors, accept/reject followers, federated content broadcast as `Note` objects, paginated outbox, NodeInfo discovery, WebFinger, shared inbox, actor profile sync
- **Remote actor discovery** — search by `@user@instance` handle, view full remote profiles (bio, banner, profile fields, posts, followers, following tabs), follow from within the UI
- **Worker-backed remote caches** — remote posts and follower/following lists are fetched by the NATS worker and cached locally; profiles populate on first visit and refresh in the background
- Content negotiation at `GET /users/{username}` — serves ActivityPub actor JSON or REST profile based on `Accept` header
- Federation moderation — per-instance domain blocking, per-user actor blocking with `Block` activity delivery, delivery filter excludes blocked actors and blocked-domain inboxes
- Async event fan-out via NATS JetStream — notifications and AP delivery run in a separate worker process; pull consumer with 1-hour TTL caching
- JWT authentication (Bearer token)
- OpenAPI documentation at `/docs` (Swagger UI) and `/scalar` (Scalar)
- Full-text search over thoughts and users via PostgreSQL trigram indexes
- Top friends — pin up to 5 users as highlighted contacts
- API keys for third-party client access
- Home feed, public feed, and per-user thought timelines

## Architecture

Hexagonal (Ports & Adapters) with Domain-Driven Design:

```
domain              — pure types and port trait definitions, no external deps
application         — use cases and event processing services (business logic)
api-types           — shared REST API request/response DTOs
presentation        — Axum HTTP router, OpenAPI spec, composition root for the API process
bootstrap           — binary: thoughts (API server)
worker              — binary: thoughts-worker (event consumer — notifications, AP fan-out)
adapters/
  auth                 — JWT issuance and validation, Argon2 password hashing
  postgres             — PostgreSQL repositories for all domain entities
  postgres-search      — PostgreSQL trigram full-text search
  postgres-federation  — PostgreSQL-backed federation repository
  activitypub-base     — core ActivityPub protocol types, ActivityPubService, federation middleware
  activitypub          — project-specific AP wiring (ThoughtsObjectHandler, inbox/outbox)
  nats                 — NATS transport implementing Transport + MessageSource ports
  event-payload        — shared event serialization DTOs
  event-transport      — Transport trait + EventPublisherAdapter / MessageSource + EventConsumerAdapter
```

## Prerequisites

- Rust stable (1.80+)
- PostgreSQL 15+
- NATS (optional — federation and notifications still work without it, events queue in-process)

## Environment Variables

Copy `.env.example` to `.env` and fill in your values:

```env
DATABASE_URL=postgres://postgres:password@localhost:5432/thoughts
JWT_SECRET=change-me
BASE_URL=http://localhost:3000
NATS_URL=nats://localhost:4222   # optional
```

See `.env.example` for all available options.

## Run

```bash
# API server (runs migrations automatically on startup)
cargo run -p bootstrap

# Event worker — federation fan-out and notifications (separate terminal)
cargo run -p worker
```

Both processes share the same PostgreSQL database. The worker is optional but required for ActivityPub delivery to remote servers.

## Test

```bash
# Unit tests — no database required
cargo test -p application

# Full workspace (requires DATABASE_URL pointing to a running PostgreSQL)
cargo test --workspace
```

The `application` crate contains unit tests for all event services and use cases backed by in-memory fakes from `domain`'s `test-helpers` feature. These are the fastest feedback loop for business logic.

## API

All REST endpoints are under the root path. Authentication uses `Authorization: Bearer <token>` obtained from `POST /auth/login`.

Interactive API documentation is available at runtime:

- **Swagger UI** — `http://localhost:8000/docs`
- **Scalar** — `http://localhost:8000/scalar`

## Frontend

The Next.js frontend lives in `thoughts-frontend/`. It requires two environment variables:

```env
NEXT_PUBLIC_API_URL=http://localhost:8000        # client-side requests
NEXT_PUBLIC_SERVER_SIDE_API_URL=http://localhost:8000  # SSR requests
```

```bash
cd thoughts-frontend
bun install
bun run dev   # http://localhost:3000
```

## Docker

The backend image contains both `thoughts` (API server) and `thoughts-worker` (event processor). Run them as separate containers:

```bash
docker build -t thoughts .

# API server
docker run -p 8000:8000 \
  -e DATABASE_URL=postgres://postgres:password@db:5432/thoughts \
  -e JWT_SECRET=change-me \
  -e BASE_URL=https://yourdomain.example.com \
  -e NATS_URL=nats://nats:4222 \
  thoughts

# Event worker (same image, different entrypoint)
docker run \
  -e DATABASE_URL=postgres://postgres:password@db:5432/thoughts \
  -e BASE_URL=https://yourdomain.example.com \
  -e NATS_URL=nats://nats:4222 \
  --entrypoint ./thoughts-worker \
  thoughts

# Frontend
docker build -t thoughts-frontend \
  --build-arg NEXT_PUBLIC_API_URL=https://api.yourdomain.example.com \
  --build-arg NEXT_PUBLIC_SERVER_SIDE_API_URL=http://thoughts:8000 \
  thoughts-frontend/
docker run -p 3000:3000 thoughts-frontend
```

See `compose.yml` for a full local development stack.

## License

MIT License. See [LICENSE](LICENSE).
