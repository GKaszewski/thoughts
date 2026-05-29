# ----- build -----
FROM rust:slim-bookworm AS builder

WORKDIR /build

# Cache dependency compilation separately from source
COPY .cargo/ .cargo/
COPY Cargo.toml Cargo.lock ./
COPY crates/adapters/activitypub/Cargo.toml       crates/adapters/activitypub/Cargo.toml
COPY crates/adapters/auth/Cargo.toml              crates/adapters/auth/Cargo.toml
COPY crates/adapters/storage/Cargo.toml           crates/adapters/storage/Cargo.toml
COPY crates/adapters/event-payload/Cargo.toml     crates/adapters/event-payload/Cargo.toml
COPY crates/adapters/event-transport/Cargo.toml   crates/adapters/event-transport/Cargo.toml
COPY crates/adapters/nats/Cargo.toml              crates/adapters/nats/Cargo.toml
COPY crates/adapters/postgres/Cargo.toml          crates/adapters/postgres/Cargo.toml
COPY crates/adapters/postgres-federation/Cargo.toml crates/adapters/postgres-federation/Cargo.toml
COPY crates/adapters/postgres-search/Cargo.toml   crates/adapters/postgres-search/Cargo.toml
COPY crates/api-types/Cargo.toml                  crates/api-types/Cargo.toml
COPY crates/application/Cargo.toml                crates/application/Cargo.toml
COPY crates/bootstrap/Cargo.toml                  crates/bootstrap/Cargo.toml
COPY crates/domain/Cargo.toml                     crates/domain/Cargo.toml
COPY crates/presentation/Cargo.toml               crates/presentation/Cargo.toml
COPY crates/worker/Cargo.toml                     crates/worker/Cargo.toml

# Stub every crate so cargo can resolve and fetch deps without real source
RUN find crates -name "Cargo.toml" | sed 's|/Cargo.toml||' | \
    xargs -I{} sh -c 'mkdir -p {}/src && echo "fn main(){}" > {}/src/main.rs && echo "" > {}/src/lib.rs'

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
 && rm -rf /var/lib/apt/lists/*

RUN cargo fetch

# Now copy real source and build
COPY crates ./crates

RUN cargo build --release -p bootstrap -p worker --features storage/s3

# ----- runtime -----
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    wget \
 && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /build/target/release/thoughts        ./thoughts
COPY --from=builder /build/target/release/thoughts-worker ./thoughts-worker

EXPOSE 8000

ENV RUST_LOG=info

CMD ["./thoughts"]
