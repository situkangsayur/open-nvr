# Open-NVR

Open-source Network Video Recorder with AI-powered analytics.

## Project Overview

Open-NVR is a self-hosted NVR system that ingests RTSP camera streams, records video segments to object storage, and runs on-device AI inference (via ONNX Runtime) for motion detection, object detection, and event alerting. It provides a web-based dashboard for live viewing, playback, and configuration.

## Tech Stack

- **Backend:** Rust (Axum web framework, SQLx for database, tokio async runtime)
- **Frontend:** Nuxt 3 (Vue 3 + TypeScript)
- **Database:** PostgreSQL
- **Cache / Pub-Sub:** Redis
- **Object Storage:** MinIO (S3-compatible)
- **Auth:** Keycloak (OpenID Connect)
- **AI Inference:** ONNX Runtime (object detection, motion analysis)
- **Streaming:** RTSP ingest via GStreamer / FFmpeg bindings
- **Containerisation:** Docker Compose for local dev, Dockerfiles for production

## Architecture

```
┌──────────┐   RTSP    ┌──────────────┐   Segments   ┌───────┐
│  Cameras │──────────> │  Ingest Svc  │────────────> │ MinIO │
└──────────┘            └──────┬───────┘              └───────┘
                               │ frames
                               v
                        ┌──────────────┐
                        │  AI Pipeline │  (ONNX Runtime)
                        └──────┬───────┘
                               │ events
                               v
┌──────────┐   REST/WS  ┌──────────────┐   SQL    ┌────────────┐
│ Frontend │<──────────> │   API (Axum) │<───────> │ PostgreSQL │
│ (Nuxt 3) │            └──────┬───────┘          └────────────┘
└──────────┘                   │
                               v
                        ┌──────────────┐
                        │    Redis     │  (cache + pub/sub)
                        └──────────────┘
```

## Workspace Structure

```
open-nvr/
├── crates/
│   ├── core/           # Shared types, config, DB models
│   ├── api/            # Axum HTTP/WebSocket server
│   ├── ingest/         # RTSP stream capture & recording
│   ├── ai-pipeline/    # ONNX inference, detection logic
│   └── migration/      # SQLx migrations
├── frontend/           # Nuxt 3 application
├── ai-models/models/   # ONNX model files (git-ignored)
├── docker/             # Dockerfiles & compose config
├── Cargo.toml          # Workspace root
├── Makefile
├── .env.example
└── CLAUDE.md           # This file
```

## Development Commands

```bash
# First-time setup
cp .env.example .env           # then edit secrets
make docker-up                 # start Postgres, Redis, MinIO, Keycloak
make migrate                   # run DB migrations

# Backend
make dev                       # run backend with hot-reload (cargo-watch)
make build                     # release build
make test                      # run all tests
make fmt                       # format code
make clippy                    # lint

# Frontend
make frontend-dev              # Nuxt dev server
make frontend-build            # production build

# Infrastructure
make docker-up                 # start services
make docker-down               # stop services
```

## Key Conventions

- All Rust crates live under `crates/` and are members of the Cargo workspace.
- Database queries use SQLx compile-time checked queries where possible.
- API endpoints follow REST conventions; real-time updates use WebSockets.
- Camera credentials are encrypted at rest using the `CREDENTIAL_ENCRYPTION_KEY`.
- ONNX model files (`.onnx`) are git-ignored; download or place them in `ai-models/models/`.
- Environment configuration is loaded from `.env` via the `dotenvy` crate.
