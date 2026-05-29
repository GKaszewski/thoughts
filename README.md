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
- JWT authentication (Bearer token) with API key support for third-party clients
- OpenAPI documentation at `/docs` (Swagger UI) and `/scalar` (Scalar)
- Full-text search over thoughts and users via PostgreSQL trigram indexes
- **Profile fields** — up to 4 custom key/value fields (Website, Pronouns, etc.), federated as AP `PropertyValue` attachment
- **Custom CSS** — per-user stylesheet applied to their profile page
- **Visibility levels** — public, followers-only, unlisted, and direct posts
- **Content warnings** — optional CW label and sensitive flag on posts
- **Feed controls** — sort by newest, oldest, most liked, most boosted, or most discussed; filter to originals only, replies only, local only, or hide sensitive
- **Popular tags** — trending hashtag discovery
- Top friends — pin up to 8 users as highlighted contacts
- Account migration — set `alsoKnownAs` for Fediverse actor moves
- Home feed, public feed, and per-user thought timelines
- Rate limiting and registration control

## Federation

Thoughts implements the [ActivityPub](https://www.w3.org/TR/activitypub/) protocol, making it compatible with Mastodon, Misskey, Pleroma, and other Fediverse software.

### Fediverse endpoints

| Endpoint | Description |
|---|---|
| `GET /.well-known/webfinger` | WebFinger discovery (`?resource=acct:user@host`) |
| `GET /.well-known/nodeinfo` | NodeInfo pointer |
| `GET /nodeinfo/2.0` | NodeInfo 2.0 — software metadata |
| `GET /users/{username}` | Actor profile (content-negotiated: JSON-LD or REST) |
| `GET /users/{username}/outbox` | Paginated outbox of `Note` activities |
| `POST /users/{username}/inbox` | Per-actor inbox |
| `POST /inbox` | Shared inbox for bulk delivery |

### Federation flow

1. A remote user follows `@you@yourinstance.com` → Mastodon sends a `Follow` activity to `/users/you/inbox`
2. Thoughts accepts and delivers an `Accept` back to the remote actor's inbox
3. When you post, Thoughts fans out a `Create(Note)` activity to all remote followers via the NATS worker
4. Remote posts from people you follow are fetched, cached, and shown in your home feed

### Without NATS

Federation still works without NATS — activities are processed in-process synchronously. The worker is required for async fan-out delivery to remote servers at scale. See [Environment Variables](#environment-variables).

### Instance moderation

- **Domain blocks** — block an entire instance; no activities are delivered to or accepted from blocked domains
- **Actor blocks** — block individual remote actors; a `Block` activity is delivered and they are filtered from all feeds

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
  storage              — object storage adapter (local filesystem + S3/MinIO) implementing the MediaStore port
  postgres             — PostgreSQL repositories for all domain entities
  postgres-search      — PostgreSQL trigram full-text search
  postgres-federation  — PostgreSQL-backed federation repository
  k-ap (external)      — generic AP protocol layer (ActivityPubService, actor management, inbox/outbox routing, follower tracking, WebFinger, NodeInfo, HTTP signatures)
  activitypub          — project-specific AP wiring (ThoughtsObjectHandler, inbox/outbox)
  nats                 — NATS transport implementing Transport + MessageSource ports
  event-payload        — shared event serialization DTOs
  event-transport      — Transport trait + EventPublisherAdapter / MessageSource + EventConsumerAdapter
```

The `domain` and `application` crates have zero concrete adapter dependencies. All I/O goes through `&dyn Port` traits, keeping business logic fully testable with in-memory fakes.

## Media Storage

Users can upload avatar and banner images via `PUT /users/me/avatar` and `PUT /users/me/banner` (multipart/form-data). Uploaded images are served at `GET /media/*path` (public, no auth required). Set `STORAGE_BACKEND` to configure the backend.

## Prerequisites

- Rust stable (1.80+)
- PostgreSQL 15+
- NATS with JetStream (optional — see [Without NATS](#without-nats))
- Docker & Docker Compose (for the easiest local setup)

### Private cargo registry

The `k-ap` crate (ActivityPub protocol library) is hosted on a private Gitea registry configured in `.cargo/config.toml`. To build the project you need read access to `git.gabrielkaszewski.dev`. If you're contributing and don't have access, open an issue and I'll sort it out.

## Environment Variables

Copy `.env.example` to `.env` and fill in your values.

### Required

| Variable | Description |
|---|---|
| `DATABASE_URL` | PostgreSQL connection string |
| `JWT_SECRET` | Secret used to sign JWT tokens — use a long random string in production |
| `BASE_URL` | Public URL of the API server — used for ActivityPub actor URLs and canonical links |

### Optional

| Variable | Default | Description |
|---|---|---|
| `HOST` | `0.0.0.0` | Interface to bind |
| `PORT` | `8000` | Port to listen on |
| `NATS_URL` | — | NATS connection string. If unset, a no-op publisher is used and events are not delivered to the worker |
| `CORS_ORIGINS` | `*` | Comma-separated allowed origins for CORS, e.g. `https://app.example.com` |
| `RATE_LIMIT` | disabled | Max requests per minute per IP |
| `ALLOW_REGISTRATION` | `true` | Set to `false` to close sign-ups |
| `RUST_ENV` | `development` | Set to `production` to disable ActivityPub debug logging |
| `RUST_LOG` | `info` | Log level filter (`error`, `warn`, `info`, `debug`, `trace`) |
| `STORAGE_BACKEND` | `local` | Storage backend: `local` or `s3` |
| `STORAGE_PATH` | — | Local filesystem path for media (required when `STORAGE_BACKEND=local`) |
| `STORAGE_PREFIX` | — | Optional key prefix for all stored objects |
| `S3_ENDPOINT` | — | S3/MinIO endpoint URL (required when `STORAGE_BACKEND=s3`) |
| `S3_ACCESS_KEY_ID` | — | S3 access key (required when `STORAGE_BACKEND=s3`) |
| `S3_SECRET_ACCESS_KEY` | — | S3 secret key (required when `STORAGE_BACKEND=s3`) |
| `S3_BUCKET` | — | S3 bucket name (required when `STORAGE_BACKEND=s3`) |
| `S3_REGION` | `us-east-1` | S3 region |
| `UPLOAD_MAX_BYTES` | `5242880` | Max upload size in bytes (default 5 MiB) |
| `UPLOAD_ALLOWED_TYPES` | `image/jpeg,image/png,image/gif,image/webp,image/avif` | Comma-separated allowed MIME types |

### Frontend environment

Copy `thoughts-frontend/.env.example` to `thoughts-frontend/.env.local` and adjust:

| Variable | Description |
|---|---|
| `NEXT_PUBLIC_API_URL` | API URL for client-side (browser) requests, e.g. `http://localhost:8000` |
| `NEXT_PUBLIC_SERVER_SIDE_API_URL` | API URL for SSR requests — same as above locally, or `http://api:8000` inside Docker |
| `NEXT_PUBLIC_FEDIVERSE_DOMAIN` | (Optional) Domain shown on profile fediverse handles, e.g. `yourinstance.example.com` |

## Run

### Local development (recommended)

Start only the infrastructure containers (Postgres + NATS), then run the Rust backend and Next.js frontend natively for fast iteration:

```bash
# 1. Start Postgres + NATS
make dev-infra

# 2. Copy and fill in env files
cp .env.example .env
cp thoughts-frontend/.env.example thoughts-frontend/.env.local

# 3. API server (runs migrations automatically on startup)
cargo run -p bootstrap

# 4. Event worker (separate terminal, optional)
cargo run -p worker

# 5. Frontend (separate terminal)
cd thoughts-frontend && bun install && bun dev
```

### Bare metal

```bash
# API server (runs migrations automatically on startup)
cargo run -p bootstrap

# Event worker — federation fan-out and notifications (separate terminal)
cargo run -p worker
```

Both processes share the same PostgreSQL database. The worker is optional but required for ActivityPub delivery to remote servers.

## Test

```bash
# Unit tests only — no database required
make test-unit

# Integration tests — requires DATABASE_URL pointing to a running PostgreSQL
make test-integration

# Everything (unit + integration)
make test

# Full check suite: fmt + clippy + tests
make check
```

`make test-unit` runs domain, application, api-types, and activitypub tests using in-memory fakes — the fastest feedback loop for business logic. `make test-integration` runs the adapter crates against a live PostgreSQL.

## API

All REST endpoints are under the root path. Authentication uses `Authorization: Bearer <token>` obtained from `POST /auth/login`.

Interactive API documentation is available at runtime:

- **Swagger UI** — `http://localhost:8000/docs`
- **Scalar** — `http://localhost:8000/scalar`

## Frontend

The Next.js frontend lives in `thoughts-frontend/`. See [Frontend environment](#frontend-environment) for required env vars, or follow the [local development](#local-development-recommended) steps above.

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
  -e STORAGE_BACKEND=local \
  -e STORAGE_PATH=/data/media \
  -v media_vol:/data/media \
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

### Full Docker stack

`compose.yml` spins up the full stack: PostgreSQL, NATS (with JetStream and monitoring on port 8222), the API server, the event worker, and the frontend.

```bash
make up          # or: docker compose up --build
```

Services:

| Service | Port | Description |
|---|---|---|
| `postgres` | 5432 | PostgreSQL 16 |
| `nats` | 4222 / 8222 | NATS with JetStream; 8222 is the monitoring endpoint |
| `api` | 8000 | Thoughts API server |
| `worker` | — | Event worker (no exposed port) |
| `frontend` | 3000 | Next.js frontend |

## Contributing

Contributions are welcome. A few guidelines:

- **Run tests before opening a PR.** At minimum: `make test-unit` (no database needed). For adapter changes: `make test-integration` with a live database. `make check` runs the full suite (fmt + clippy + tests).
- **Keep the hexagonal boundary.** `domain` and `application` must not import any adapter crate. Use `&dyn Port` traits for all I/O.
- **No ORM.** The project uses raw `sqlx`. Keep it that way.
- **ActivityPub changes** — test against a live Mastodon instance if possible, or use the AP debug logs (`RUST_ENV=development`).
- **Small, focused PRs** are easier to review than large ones.

For significant changes, open an issue first to discuss the approach.

## License

MIT License. See [LICENSE](LICENSE).
