# Smart Access Control

**English** | [简体中文](README.zh-CN.md)

A **software-simulated** physical access control backend written in Rust
(Axum + Tokio + PostgreSQL), with a React dashboard ([`dashboard/`](dashboard/README.md)).
It is a learning project; it does not talk to real hardware or any vendor protocol.

- Design and request flows: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
- Security review and known risks: [SECURITY.md](SECURITY.md)

## Features

- **Access decisions**: a card swipe is checked against card, holder, door
  state, group permissions and schedules; every attempt is written to an
  append-only event log. Anything missing or failing is denied.
- **Admin API** (`/api/v1`): users, cards, doors, access groups, schedules,
  permissions, events and an audit log. Roles: `admin` (read/write) and
  `viewer` (read-only).
- **Auth**: Argon2id passwords, short-lived JWT access tokens, rotating
  refresh tokens with reuse detection, login throttling.
- **Simulated door controllers** with per-controller keys, heartbeats and
  offline detection, plus an in-process simulator for demos.
- **Live events** over WebSocket, request IDs, JSON logs and Prometheus metrics.

## Requirements

- Rust stable (edition 2024)
- Docker with Docker Compose
- Node.js 20.19+ or 22.12+ (dashboard)

## Quick start

Run from this directory:

```sh
cp .env.example .env          # then set JWT_SECRET to `openssl rand -hex 32`
docker compose up -d          # PostgreSQL; migrations run on app startup
cargo run                     # API on http://127.0.0.1:8080

curl http://127.0.0.1:8080/api/v1/health

cargo run -- create-admin ops admin           # first admin; password from stdin
cd dashboard && npm install && npm run dev    # http://localhost:5173
```

With `SIMULATOR_ENABLED=true` (the `.env.example` default), the dashboard's
**Monitor** page can start simulated controllers and swipe cards.

## Configuration

Settings come from environment variables (or `.env`). The essentials:

| Variable | Purpose |
| -------- | ------- |
| `DATABASE_URL` | **Required.** PostgreSQL connection |
| `JWT_SECRET` | **Required**, ≥ 32 bytes |
| `APP_HOST` / `APP_PORT` | Bind address (default `127.0.0.1:8080`) |
| `SIMULATOR_ENABLED` | Enable `/simulator/*` (dev/demo only) |
| `CORS_ALLOWED_ORIGINS` | Browser origins allowed to call the API |
| `LOG_FORMAT` | `pretty` or `json` |
| `METRICS_TOKEN` | Enables `/metrics` behind this bearer token |

See [`.env.example`](.env.example) and [`src/config.rs`](src/config.rs) for the full list.

## Deployment

```sh
cp .env.production.example .env.production   # fill in SITE_ADDRESS and secrets
docker compose -f docker-compose.prod.yml --env-file .env.production up -d --build
```

Runs Caddy (HTTPS) → a distroless app container serving API and dashboard →
PostgreSQL on an internal network.

## Testing

Database tests need the compose PostgreSQL and `TEST_DATABASE_URL` (set in
`.env.example`). Each test uses its own throwaway database.

```sh
cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
cd dashboard && npm run lint && npm run typecheck && npm test
```
