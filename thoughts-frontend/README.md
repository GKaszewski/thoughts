# Thoughts — Frontend

Next.js 15 (App Router) frontend for the [Thoughts](../) self-hosted microblogging server.

## Features

- Post thoughts, reply, boost, and like
- Home feed, public feed, per-user timelines
- Browse and follow remote Fediverse actors by `@user@instance` handle
- Full remote actor profiles — bio, banner, profile fields, posts tab, followers/following tabs
- Full-text search for local users and thoughts; remote actor lookup via WebFinger
- Notifications, API key management, profile editing
- Dark/light theme

## Setup

```bash
bun install
```

Copy `.env.local.example` to `.env.local` (or set the variables directly):

```env
NEXT_PUBLIC_API_URL=http://localhost:8000
NEXT_PUBLIC_SERVER_SIDE_API_URL=http://localhost:8000
```

`NEXT_PUBLIC_API_URL` is used by client-side fetches (runs in the browser).
`NEXT_PUBLIC_SERVER_SIDE_API_URL` is used by server-side fetches (runs in Next.js SSR — can point to an internal service URL in Docker).

## Run

```bash
bun run dev     # development — http://localhost:3000
bun run build   # production build
bun run start   # serve production build
```

## Docker

```bash
docker build \
  --build-arg NEXT_PUBLIC_API_URL=https://api.yourdomain.example.com \
  --build-arg NEXT_PUBLIC_SERVER_SIDE_API_URL=http://thoughts:8000 \
  -t thoughts-frontend .
docker run -p 3000:3000 thoughts-frontend
```
