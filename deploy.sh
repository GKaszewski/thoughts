#!/usr/bin/env bash
set -euo pipefail

REGISTRY="registry.gabrielkaszewski.dev"
BACKEND_IMAGE="$REGISTRY/thoughts:latest"
FRONTEND_IMAGE="$REGISTRY/thoughts-frontend:latest"

# Public API URL seen by the browser.
# Override with: NEXT_PUBLIC_API_URL=https://api.example.com ./deploy.sh
API_URL="${NEXT_PUBLIC_API_URL:-https://api.thoughts.gabrielkaszewski.dev}"

# Internal API URL used by Next.js SSR (can be a Docker-internal address in prod).
# Override with: NEXT_PUBLIC_SERVER_SIDE_API_URL=http://api:8000 ./deploy.sh
SSR_API_URL="${NEXT_PUBLIC_SERVER_SIDE_API_URL:-$API_URL}"

echo "==> building backend image: $BACKEND_IMAGE"
docker buildx build --platform linux/amd64 \
  -t "$BACKEND_IMAGE" --push .

echo "==> building frontend image: $FRONTEND_IMAGE"
docker buildx build --platform linux/amd64 \
  --build-arg "NEXT_PUBLIC_API_URL=$API_URL" \
  --build-arg "NEXT_PUBLIC_SERVER_SIDE_API_URL=$SSR_API_URL" \
  -t "$FRONTEND_IMAGE" --push \
  ./thoughts-frontend

echo "==> pushed $BACKEND_IMAGE"
echo "==> pushed $FRONTEND_IMAGE"
