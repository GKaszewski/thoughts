FROM node:22-slim AS builder
WORKDIR /app

ARG NEXT_PUBLIC_API_URL
ENV NEXT_PUBLIC_API_URL=$NEXT_PUBLIC_API_URL

ARG NEXT_PUBLIC_SERVER_SIDE_API_URL
ENV NEXT_PUBLIC_SERVER_SIDE_API_URL=$NEXT_PUBLIC_SERVER_SIDE_API_URL

# Install dependencies with Bun for speed
COPY --chown=node:node package.json bun.lock ./
RUN npm install -g bun
RUN bun install --frozen-lockfile

# Copy the rest of the app and build with Node's Next.js runtime
COPY --chown=node:node . .

ENV NODE_ENV=production
RUN bun run build

FROM node:22-slim AS release

RUN apt-get update && apt-get install -y --no-install-recommends curl && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/public ./public
COPY --from=builder /app/.next/standalone ./
COPY --from=builder /app/.next/static ./.next/static

EXPOSE 3000
ENV HOSTNAME="0.0.0.0"
CMD ["node", "server.js"]

