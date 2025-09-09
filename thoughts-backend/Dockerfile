FROM rust:1.89-slim AS builder

RUN apt-get update && apt-get install -y libssl-dev pkg-config && rm -rf /var/lib/apt/lists/*

RUN cargo install cargo-chef --locked
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY api/Cargo.toml ./api/
COPY app/Cargo.toml ./app/
COPY common/Cargo.toml ./common/
COPY doc/Cargo.toml ./doc/
COPY migration/Cargo.toml ./migration/
COPY models/Cargo.toml ./models/
COPY utils/Cargo.toml ./utils/

RUN mkdir -p src && echo "fn main() {}" > src/main.rs

RUN cargo chef prepare --recipe-path recipe.json

RUN cargo chef cook --release --recipe-path recipe.json

COPY . .

RUN cargo build --release --bin thoughts-backend

FROM debian:13-slim AS runtime

RUN apt-get update && apt-get install -y openssl && rm -rf /var/lib/apt/lists/*

RUN groupadd --system --gid 1001 appgroup && \
    useradd --system --uid 1001 --gid appgroup appuser

WORKDIR /app

COPY --from=builder /app/target/release/thoughts-backend .

RUN chown -R appuser:appgroup /app

USER appuser

EXPOSE 8000

CMD ["./thoughts-backend"]