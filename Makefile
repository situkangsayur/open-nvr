.PHONY: dev build test docker-up docker-down migrate fmt clippy frontend-dev frontend-build

# Start backend in development mode with hot-reload
dev:
	cargo watch -x 'run --bin open-nvr'

# Build the backend in release mode
build:
	cargo build --release

# Run all Rust tests
test:
	cargo test --workspace

# Start all Docker services (Postgres, Redis, MinIO, Keycloak)
docker-up:
	docker compose up -d

# Stop all Docker services
docker-down:
	docker compose down

# Run database migrations
migrate:
	cargo run --bin open-nvr -- migrate

# Format all Rust code
fmt:
	cargo fmt --all

# Run clippy lints on all workspace crates
clippy:
	cargo clippy --workspace -- -D warnings

# Start Nuxt frontend dev server
frontend-dev:
	cd frontend && npm run dev

# Build Nuxt frontend for production
frontend-build:
	cd frontend && npm run build
