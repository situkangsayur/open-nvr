# Open-NVR

Open-source Network Video Recorder for IP cameras. Built with Rust (Axum) + Nuxt 3 + PostgreSQL + Keycloak.

## Features

- **Multi-protocol**: RTSP, ONVIF, MJPEG camera support
- **Network Discovery**: Auto-scan subnets, ARP table, ONVIF WS-Discovery, MAC OUI identification
- **Live View**: WebSocket-based MSE streaming, fullscreen mode, grid layouts
- **Recording**: Continuous/motion-based recording to MinIO (S3-compatible)
- **Motion Detection**: Pure Rust frame differencing with configurable zones
- **Playback**: Timeline view, multi-camera sync, speed control, MP4 export
- **PTZ Control**: Pan/tilt/zoom via ONVIF
- **Security**: Keycloak OIDC auth, RBAC (admin/operator/viewer), per-camera access, OWASP headers, audit trail
- **Alerts**: Detection events, network security monitoring, browser notifications
- **Dark/Light Mode**: Toggle theme with localStorage persistence
- **System Capacity**: Server spec analysis with camera capacity estimates
- **Message Broker**: NATS JetStream for event pub/sub
- **Credential Encryption**: AES-encrypted camera passwords

## Tech Stack

| Component | Technology |
|-----------|-----------|
| Backend | Rust (Axum + Tokio) |
| Frontend | Nuxt 3 + Vue 3 + TypeScript + Tailwind CSS |
| Database | PostgreSQL 16 |
| Auth | Keycloak 26 (OIDC + RBAC) |
| Storage | MinIO (S3-compatible) |
| Cache | Redis 7 |
| Broker | NATS 2 (JetStream) |
| Video | retina (RTSP), ffmpeg (snapshots/export) |

## Quick Start

### Prerequisites
- Docker & Docker Compose
- Rust 1.75+
- Node.js 20+
- ffmpeg

### Development

```bash
# Clone
git clone git@github.com:situkangsayur/open-nvr.git
cd open-nvr

# Start dependencies
cp .env.example .env
docker compose up -d

# Backend
cd backend
cargo run

# Frontend (new terminal)
cd frontend
npm install --legacy-peer-deps
npx nuxt dev
```

### Production Deployment

```bash
# On target server
./scripts/deploy.sh
```

See `scripts/deploy.sh` for full deployment automation.

## Default Credentials

| User | Password | Role |
|------|----------|------|
| admin | admin123 | Admin (full access) |
| operatoruser | operatoruser123 | Operator |
| vieweruser | vieweruser123 | Viewer |

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | /health | Health check |
| GET/POST | /api/cameras | Camera CRUD |
| POST | /api/cameras/{id}/test | Test connection |
| POST | /api/cameras/{id}/ping | Check liveness |
| GET | /api/cameras/{id}/snapshot | ffmpeg snapshot |
| POST | /api/discovery/scan | Network scanner |
| GET | /api/discovery/subnets | Auto-detect subnets |
| GET | /api/timeline | Recording timeline |
| GET | /api/recordings | Recording list |
| GET | /api/events | Detection events |
| POST | /api/cameras/{id}/ptz | PTZ control |
| GET/POST | /api/layouts | Grid layouts |
| GET/POST | /api/settings/retention | Retention policies |
| GET | /api/system/capacity | Server capacity analysis |
| GET/POST | /api/permissions/* | RBAC management |
| GET | /api/audit/logs | Audit trail |
| WS | /ws/stream/{id} | Live video stream |
| WS | /ws/events | Real-time events |

## Camera Support

| Brand | Protocol | Auto-Discover | Notes |
|-------|----------|--------------|-------|
| Hikvision | RTSP, ONVIF | Yes (MAC OUI) | |
| Dahua | RTSP, ONVIF | Yes (MAC OUI) | |
| Reolink | RTSP | Yes (MAC OUI) | |
| TP-Link Tapo | RTSP | Yes | |
| V360 Pro / XMEye | RTSP + ONVIF PTZ | Yes (MAC OUI + ONVIF) | ONVIF port 8899, PTZ via ContinuousMove. Audio not available via RTSP (P2P only). Best URL: /live/ch00_1 |
| Imou | RTSP (needs enabling) | Yes (MAC OUI) | |
| ESP32-CAM | MJPEG | Yes (HTTP) | |
| Generic ONVIF | ONVIF + RTSP | Yes (WS-Discovery) | |

## ONVIF Discovery Results

| Camera IP | Brand | PTZ | Audio (ONVIF) | Audio (RTSP) | Best Stream |
|-----------|-------|-----|---------------|--------------|-------------|
| V360/XiongMai | V360 | Digital PTZ | Source exists, not in stream | No | /live/ch00_1 (720p) |
| (LAN camera) | Unknown | Real PTZ motor | No | No | /live/ch00_1 |

### Known Limitations
- V360/XiongMai cameras do NOT send audio via RTSP despite having audio hardware
- Audio is only available via XMEye P2P proprietary protocol (V360 Pro app)
- PTZ on V360 is digital (software crop/pan), not physical motor
- ONVIF port is 8899, not standard 80

## License

MIT
