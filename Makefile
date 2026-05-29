.DEFAULT_GOAL := check

# Run the full local check suite — same order as CI would.
check: fmt-check clippy test
	@echo "✅ All checks passed"

# Apply rustfmt to all files.
fmt:
	cargo fmt

# Check formatting without modifying files (CI-safe).
fmt-check:
	cargo fmt --check

# Run Clippy and treat warnings as errors.
clippy:
	cargo clippy -- -D warnings

# Run the full test suite (requires DATABASE_URL).
test:
	cargo test

# Unit tests only — no database required.
test-unit:
	cargo test -p domain -p application -p api-types -p activitypub

# Integration tests only — requires DATABASE_URL.
test-integration:
	cargo test -p postgres -p postgres-federation -p postgres-search -p presentation

# Apply fmt + clippy auto-fixes in one shot.
fix:
	cargo fmt
	cargo clippy --fix --allow-dirty --allow-staged

# Start infra (Postgres + NATS) for local development.
dev-infra:
	docker compose up postgres nats -d

# Stop infra.
dev-infra-down:
	docker compose down

# Full Docker stack.
up:
	docker compose up --build

.PHONY: check fmt fmt-check clippy test test-unit test-integration fix dev-infra dev-infra-down up
