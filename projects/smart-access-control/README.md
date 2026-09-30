# Smart Access Control

**English** | [简体中文](README.zh-CN.md)

A **software-simulated** physical access control backend written in Rust
(Axum + Tokio). It is a learning project inspired by enterprise access control
platforms. It does not talk to real hardware and is not compatible with any
proprietary access control protocol.

> Status: **complete (Phases 1–8)**. The backend and the React dashboard
> ([`dashboard/`](dashboard/README.md)) form a software-only access control
> simulation with a security review ([SECURITY.md](SECURITY.md)),
> observability and a container deployment.
>
> **How it is built and why:** [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
> (layers, request flows, data model, design decisions).

## How an access decision works

1. `AccessDecisionService` takes a card number and a door ID. The decision
   time always comes from the server clock, never from the device.
2. `PgAccessDataSource` loads the card, holder, door, group memberships,
   permissions and schedules in one read-only `REPEATABLE READ` transaction,
   so the decision sees a single consistent state.
3. The pure function `domain::decide` checks, in order: card exists, card
   active, not expired, holder active, door exists / enabled / online,
   a permission applies, and its schedule allows the current local time.
   The first failed check is the denial reason. Anything missing is denied.
4. The attempt is appended to `access_events`, granted or denied. If it
   cannot be recorded, the service returns an error and the caller must
   treat that as denied. A database trigger rejects any `UPDATE`, `DELETE`
   or `TRUNCATE` on the log.

Denial reasons: `unknown_card`, `card_revoked`, `card_suspended`,
`card_expired`, `user_suspended`, `user_archived`, `unknown_door`,
`door_disabled`, `door_offline`, `permission_denied`, `outside_schedule`.

## Requirements

- Rust stable (edition 2024, tested with 1.98)
- Docker with Docker Compose (for PostgreSQL)
- Node.js 20.19+ or 22.12+ for the dashboard (tested with 24)
- Optional: `cargo install cargo-audit` for the dependency audit

## Quick start

All commands run from this directory (`projects/smart-access-control/`).

```sh
# 1. Configure: copy the example and change the password.
cp .env.example .env

# 2. Start PostgreSQL. The app needs it and applies migrations on startup.
docker compose up -d
docker compose ps            # wait for "healthy"

# 3. Run the API.
cargo run

# 4. Check it.
curl http://127.0.0.1:8080/api/v1/health
# {"status":"ok","version":"0.1.0"}
curl http://127.0.0.1:8080/api/v1/health/ready
# {"status":"ready"}
```

`cp .env.example .env` is not enough on its own: replace `JWT_SECRET` with
the output of `openssl rand -hex 32`. The server refuses to start with a
missing, short, or example secret.

Then create an administrator and start the dashboard:

```sh
# 5. An account to sign in with (the password is read from stdin).
cargo run -- create-admin ops admin

# 6. The dashboard, in a second terminal.
cd dashboard && npm install && npm run dev   # http://localhost:5173
```

With `SIMULATOR_ENABLED=true` (the `.env.example` default) the dashboard's
**Monitor** page can start simulated controllers and swipe cards, so you
can watch decisions arrive live without any other tools.

## Administrators and login

There is no default account. Create the first administrator with the
`create-admin` command; it reads the password (at least 12 characters) from
standard input:

```sh
cargo run -- create-admin alice admin     # role: admin (read/write) or viewer (read-only)
```

Typing the password at the prompt echoes it on screen; to avoid that, pipe it
in, e.g. from a password manager's CLI.

```sh
# Log in: returns a 15-minute access token and a 7-day refresh token.
curl -s http://127.0.0.1:8080/api/v1/auth/login \
  -H 'content-type: application/json' \
  -d '{"username":"alice","password":"<password>"}'

# Call a protected endpoint with the access token.
curl -s http://127.0.0.1:8080/api/v1/auth/me -H "authorization: Bearer <access_token>"
```

How sessions work:

- Passwords are stored as Argon2id hashes. Hashing runs on Tokio's blocking
  thread pool so it never stalls other requests.
- Every failed login returns the same `401 invalid credentials`, and an
  unknown username costs the same hashing time as a wrong password, so
  neither the response nor its timing reveals which accounts exist.
- Access tokens are HS256 JWTs (15 minutes). They are not stored, so a
  disabled administrator keeps access until the current token expires.
- Refresh tokens are random, stored only as SHA-256 hashes, and work once:
  `POST /auth/refresh` returns a new pair. Presenting an already-rotated
  refresh token again is treated as theft and revokes all of that
  administrator's sessions.
- Login with `"session": "cookie"` (what the dashboard does) delivers the
  refresh token only as an `HttpOnly; Secure; SameSite=Strict` cookie scoped
  to `/api/v1/auth`; refresh and logout then read the cookie. Without it,
  the token is returned in the body, for API clients.
- Failed logins are throttled: 5 per account name or 20 per client address
  in 15 minutes, then `429` with `Retry-After`.
- Logins, failed logins, logouts, detected token reuse and account creation
  are recorded in the append-only `admin_audit_log` table.

Not yet implemented: multi-factor authentication, and disabling or deleting
administrators through the API. See [SECURITY.md](SECURITY.md) for the
security review, the controls in place and the known risks.

Stop the server with `Ctrl+C` (or SIGTERM). It finishes in-flight requests
before exiting. Stop the database with `docker compose down`, or use
`docker compose down -v` to also delete its data volume.

## Configuration

Settings come from environment variables. A `.env` file is loaded if present,
and real environment variables take precedence over it.

| Variable            | Default                                        | Purpose                           |
| ------------------- | ---------------------------------------------- | --------------------------------- |
| `APP_HOST`          | `127.0.0.1`                                    | API bind address                  |
| `APP_PORT`          | `8080`                                         | API port                          |
| `RUST_LOG`          | `smart_access_control=debug,tower_http=debug`  | Log filter                        |
| `LOG_FORMAT`        | `pretty`                                       | `pretty` or `json` log lines      |
| `METRICS_TOKEN`     | unset (no `/metrics`)                          | Bearer token for `/metrics`, ≥ 16 bytes (never logged) |
| `POSTGRES_USER`     | required by compose                            | Database user                     |
| `POSTGRES_PASSWORD` | required by compose                            | Database password                 |
| `POSTGRES_DB`       | required by compose                            | Database name                     |
| `POSTGRES_PORT`     | `55432`                                        | Host port for the compose DB      |
| `DATABASE_URL`      | **required**                                   | App DB connection (never logged)  |
| `DATABASE_MAX_CONNECTIONS` | `10`                                    | Connection pool size              |
| `JWT_SECRET`        | **required**, ≥ 32 bytes                       | Signs access tokens (never logged)|
| `ACCESS_TOKEN_TTL_SECONDS` | `900`                                   | Access token lifetime             |
| `REFRESH_TOKEN_TTL_SECONDS` | `604800`                               | Refresh token lifetime            |
| `CORS_ALLOWED_ORIGINS` | empty                                       | Browser origins allowed (CORS)    |
| `CONTROLLER_TIMEOUT_SECONDS` | `30`                                  | Silence before a controller is offline |
| `SIMULATOR_ENABLED` | `false`                                        | Mount `/simulator/*` (development/demo only) |
| `WS_MAX_CONNECTIONS` | `256`                                         | Concurrent live-event WebSockets  |
| `COOKIE_SECURE`     | `true`                                         | `Secure` flag on the refresh-token cookie |
| `TRUSTED_PROXIES`   | empty                                          | Proxy IPs/CIDRs whose `X-Forwarded-For` is believed |
| `STATIC_DIR`        | unset                                          | Serve the built dashboard from this directory at `/` |
| `TEST_DATABASE_URL` | none (tests only)                              | Local server for test databases   |

The compose database listens on `127.0.0.1:55432` by default, so it does not
clash with a local PostgreSQL on 5432. If the configuration is invalid, the
app logs the error and exits with a non-zero status.

## API

Base path `/api/v1`. All bodies are JSON; timestamps are RFC 3339 (UTC).

**Access rules.** Health and `auth/login|refresh|logout` are public. Every
other route needs `Authorization: Bearer <access token>`. Reads are allowed
for both roles (`admin`, `viewer`); every route that changes data needs
`admin` (a viewer gets `403`).

| Method | Path | Description |
| ------ | ---- | ----------- |
| GET    | `/health` | Liveness: `{status, version}` |
| GET    | `/health/ready` | Readiness: 200 if the database answers, else 503 |
| POST   | `/auth/login` | `{username, password}` → access + refresh token |
| POST   | `/auth/refresh` | `{refresh_token}` → new pair (old token stops working) |
| POST   | `/auth/logout` | `{refresh_token}` → 204 |
| GET    | `/auth/me` | `{id, role}` of the token's administrator |
| GET    | `/users` | List (paginated) |
| POST   | `/users` | `{name, email}` → 201 |
| GET    | `/users/{id}` | One user |
| PATCH  | `/users/{id}` | Any of `{name, email, status}`; status `active`/`suspended`/`archived` |
| DELETE | `/users/{id}` | Archives (soft delete) → 204 |
| GET    | `/cards` | List; `?user_id=` for one user's cards |
| POST   | `/cards` | `{user_id, card_number, expires_at?}` → 201 |
| GET    | `/cards/{id}` | One card, incl. `effective_status` (e.g. `expired`) |
| PATCH  | `/cards/{id}` | `{status: "active" \| "suspended"}` |
| POST   | `/cards/{id}/revoke` | Revokes permanently |
| GET    | `/doors` | List |
| POST   | `/doors` | `{name, location, controller_id}` → 201 (starts `offline`) |
| GET    | `/doors/{id}` | One door |
| PATCH  | `/doors/{id}` | Any of `{name, location, controller_id}` |
| PATCH  | `/doors/{id}/status` | `{status: "online" \| "offline" \| "disabled"}` (`offline` re-enables a disabled door) |
| GET    | `/access-groups` | List |
| POST   | `/access-groups` | `{name, description?}` → 201 |
| GET    | `/access-groups/{id}` | One group |
| PATCH  | `/access-groups/{id}` | `{name?, description?}`; `"description": null` clears it |
| DELETE | `/access-groups/{id}` | Also removes its memberships and permissions |
| GET    | `/access-groups/{id}/members` | `{user_ids: [...]}` |
| POST   | `/access-groups/{id}/members` | `{user_id}` → 204 |
| DELETE | `/access-groups/{id}/members/{user_id}` | → 204 |
| GET    | `/schedules` | List |
| POST   | `/schedules` | `{name, timezone, rules: [{days, start, end}], effective_from?, effective_until?}` → 201 |
| GET    | `/schedules/{id}` | One schedule |
| GET    | `/permissions` | List |
| POST   | `/permissions` | `{group_id, door_id, schedule_id?}` → 201 (no schedule = any time) |
| DELETE | `/permissions/{id}` | → 204 |
| POST   | `/access/decisions` | `{card_number, door_id}` → the recorded event (200 for grant and denial) |
| GET    | `/events` | Access events, newest first; filters below |
| GET    | `/events/stream` | WebSocket: live access events (see below) |
| GET    | `/events/recent` | Latest events, `?limit=` (default 20) |
| GET    | `/events/{id}` | One event |
| GET    | `/audit-log` | Administrative actions, newest first (paginated) |
| GET    | `/controllers` | Registered controllers with status and last contact |
| POST   | `/controllers` | `{controller_id}` → 201 with the controller's secret `key` (shown once) |
| GET    | `/controllers/{id}` | One controller |
| POST   | `/controllers/{id}/rotate-key` | New key; the old one stops working |
| POST   | `/device/heartbeat` | Controller key required → `{controller_id, doors_online}` |
| POST   | `/device/access-requests` | Controller key → `{card_number, door_id}` → the recorded event |
| POST   | `/device/disconnect` | Controller key → 204; its doors go offline |

Schedule rule example: `{"days": ["mon","tue","wed","thu","fri"], "start":
"09:00", "end": "17:00"}` in the schedule's IANA `timezone`; `end <= start`
means the window ends the next day.

**Event filters** (`GET /events` and WebSocket subscriptions): `door_id`,
`user_id`, `card_id`, `decision` (`granted`/`denied`), `reason` (e.g.
`door_offline`), `from` (RFC 3339, inclusive), `until` (exclusive). Set
filters are combined with AND.

**Lists** accept `?limit=` (default 50, max 100) and `?offset=`, and return
`{"items": [...], "limit": n, "offset": m}`.

**Errors** always look like `{"error": {"code": "...", "message": "..."}}`:
`400 invalid_request/invalid_path/invalid_query`, `401 unauthorized`,
`403 forbidden`, `404 not_found`, `409 conflict/invalid_state`,
`413 payload_too_large`, `415 unsupported_media_type`,
`422 validation_failed` (also for unknown or mistyped JSON fields),
`500 internal_error` (details are logged, never returned). Bodies are
limited to 64 KiB.

**Audit.** Every change made through the API is recorded in `admin_audit_log`
with the acting administrator. The entry is written after the change
succeeds, not in the same transaction: if writing it fails, the change stands
and the failure is logged as an error.

**CORS.** Set `CORS_ALLOWED_ORIGINS` (comma-separated, e.g.
`http://localhost:5173`) to let a browser dashboard on another origin call
the API. Empty (the default) allows same-origin requests only.

## Door controllers (simulated)

Controllers here are **software simulations**. Nothing in this project talks
to, or is compatible with, real access control hardware or any vendor's
protocol.

1. An administrator registers a controller: `POST /controllers
   {"controller_id": "ctrl-001"}`. The response contains a secret `key`,
   shown only once (only its SHA-256 hash is stored).
2. Doors are wired to it by `controller_id` (set when creating the door).
3. The controller calls the `/device/*` endpoints with the header
   `X-Controller-Key: <key>`. This credential is separate from
   administrator tokens: neither works in place of the other.
4. Every device call counts as a heartbeat and brings the controller and its
   enabled doors online. A controller silent for `CONTROLLER_TIMEOUT_SECONDS`
   (default 30) is marked offline with its doors by a background check,
   which runs three times per timeout period. `POST /device/disconnect`
   does the same immediately.
5. `POST /device/access-requests` only accepts the controller's own doors;
   any other door gets `403` and nothing is recorded. The controller must
   treat any error as "denied".

`POST /access/decisions` (admin token) remains available as a manual test
tool that bypasses controllers.

### In-process simulator

With `SIMULATOR_ENABLED=true`, the server can run simulated controllers
itself (each is a Tokio task), steered through admin-only endpoints. They use
the same protocol as a device: a controller key, heartbeats every third of
`CONTROLLER_TIMEOUT_SECONDS`, and the same access checks.

| Method | Path | Description |
| ------ | ---- | ----------- |
| POST   | `/simulator/controllers` | `{controller_id}` → start (registers it, or rotates its key if already registered) |
| GET    | `/simulator/controllers` | Running simulations: network up/down, heartbeats, last error |
| GET    | `/simulator/controllers/{id}` | One simulation |
| POST   | `/simulator/controllers/{id}/swipe` | `{card_number, door_id}` → the decision (as the event) |
| POST   | `/simulator/controllers/{id}/outage` | Network drops silently; the backend notices via the timeout |
| POST   | `/simulator/controllers/{id}/disconnect` | Clean disconnect; doors go offline at once |
| POST   | `/simulator/controllers/{id}/reconnect` | Network back; heartbeats immediately |
| DELETE | `/simulator/controllers/{id}` | Stop (disconnects cleanly) |

During an outage a swipe returns `503 controller_offline`: the simulated
reader denies locally and nothing reaches the backend or the event log. On
server shutdown every simulated controller disconnects cleanly first. When
the simulator is disabled (the default), these routes do not exist (404).

A quick demo (after logging in as an admin, `$T` = access token):

```sh
api() { curl -s -X "$1" "http://127.0.0.1:8080/api/v1$2" -H "authorization: Bearer $T" \
          -H 'content-type: application/json' ${3:+-d "$3"}; }
DOOR=$(api POST /doors '{"name":"Lab","location":"B1","controller_id":"sim-1"}' | jq -r .id)
api POST /simulator/controllers '{"controller_id":"sim-1"}'    # door comes online
api POST /simulator/controllers/sim-1/swipe "{\"card_number\":\"CARD-1\",\"door_id\":\"$DOOR\"}"
```

## Live events (WebSocket)

Connect to `ws://<host>/api/v1/events/stream` and, within 5 seconds, send:

```json
{"type": "subscribe", "token": "<access token>", "filter": {"door_id": "...", "decision": "denied"}}
```

`filter` is optional. The server replies `{"type": "subscribed", "expires_at":
"..."}` and then pushes `{"type": "access_event", "event": {...}}` as access
decisions happen. Both roles may subscribe. The token is sent in the first
message rather than the URL because browsers cannot set headers on
WebSockets and URLs end up in logs.

Close codes: `4400` malformed message or filter, `4401` invalid token,
`4408` no subscribe message within 5 s, `4409` access token expired
(reconnect with a fresh token), `1001` server shutting down.

**Delivery guarantees.** Live delivery is best-effort:

- An event is published only after it has been stored, so everything a
  dashboard shows is in the database.
- Publishing never waits for subscribers; a slow dashboard cannot delay
  access decisions. A subscriber more than 1024 events behind loses the
  oldest ones and receives `{"type": "lagged", "missed": n}`. A client that
  cannot accept a message within 10 s is disconnected.
- Events are delivered in order, at most once, and only to clients
  connected at the time. Nothing is replayed on reconnect: fetch missed
  events with `GET /events?from=<last seen occurred_at>`.
- The channel is in-process. With several server instances, each client
  only sees events decided by its own instance.

## Observability

**Request IDs.** Every response has an `X-Request-Id` header, and every log
line written while handling the request carries the same `request_id`. An
incoming `X-Request-Id` (e.g. from a reverse proxy) is kept if it is 1–64
characters of `A–Z a–z 0–9 - _ . :`; anything else is replaced with a new
UUID, so client-supplied text cannot forge or break log lines.

**Logs.** `LOG_FORMAT=json` writes one JSON object per line for log
collectors (Loki, Elasticsearch, CloudWatch, ...); the default `pretty` is
for terminals. `RUST_LOG` sets the levels. Secrets and card numbers are
never logged.

```bash
LOG_FORMAT=json cargo run | jq 'select(.span.request_id == "…")'
```

**Metrics.** With `METRICS_TOKEN` set, `GET /metrics` (outside `/api/v1`)
serves Prometheus text format to callers sending
`Authorization: Bearer <token>`; without it the route does not exist.

| Metric | Type | Labels |
| ------ | ---- | ------ |
| `http_requests_total` | counter | `method`, `path` (route pattern, e.g. `/api/v1/users/{id}`), `status` |
| `http_request_duration_seconds` | histogram | same |
| `access_decisions_total` | counter | `decision` (`granted`/`denied`), `reason` |
| `auth_login_attempts_total` | counter | `outcome` (`success`/`failure`/`throttled`) |
| `auth_refresh_token_reuse_total` | counter | — (a replayed refresh token: possible theft) |
| `websocket_connections` | gauge | — |
| `controllers_expired_total` | counter | — (controllers marked offline by the liveness monitor) |

Paths are labelled by route pattern, never the raw URL, so IDs don't create
unbounded series. Example Prometheus job:

```yaml
scrape_configs:
  - job_name: smart-access-control
    authorization: { credentials: "<METRICS_TOKEN>" }
    static_configs: [{ targets: ["127.0.0.1:8080"] }]
```

## Deployment

A production-style stack runs with Docker Compose:

```text
internet ──► proxy (Caddy) ──────────► app (API + dashboard) ──► db (PostgreSQL)
             HTTPS, HSTS, gzip/zstd     one container,           no published port,
             blocks /metrics            non-root, read-only      internal network only
```

```bash
cp .env.production.example .env.production   # fill in SITE_ADDRESS and the secrets
docker compose -f docker-compose.prod.yml --env-file .env.production up -d --build

# First administrator (password from stdin):
docker compose -f docker-compose.prod.yml --env-file .env.production \
  exec -T app smart-access-control create-admin ops admin < password.txt
```

With a public `SITE_ADDRESS` whose ports 80/443 reach the host, Caddy
obtains a Let's Encrypt certificate automatically. To try it on your
machine, use `SITE_ADDRESS=localhost`, `HSTS_MAX_AGE=0` and, if 80/443 are
taken, `HTTP_PORT=8088 HTTPS_PORT=8443`; Caddy then uses its own local CA
(`curl -k`, or accept the browser warning).

What the pieces do:

- **Image** ([`Dockerfile`](Dockerfile)): three stages. Node builds the
  dashboard, Rust builds the release binary (dependency downloads and build
  output are cached between builds), and the result is copied into
  distroless `cc-debian13`: glibc and CA certificates, no shell, runs as
  uid 65532. About 66 MB. The image's health check runs
  `smart-access-control healthcheck`, which asks `/api/v1/health/ready`
  (there is no curl in the image).
- **Dashboard**: `STATIC_DIR` makes the API serve it, so dashboard and API
  share one origin (no CORS, first-party refresh cookie). Hashed files under
  `/assets` are cached for a year; `index.html` is revalidated on every load;
  unknown paths return `index.html` for client-side routing, except under
  `/api/`, which answers JSON 404s. Pages get their own
  Content-Security-Policy (`default-src 'self'`, no inline scripts, no
  framing); API responses keep `default-src 'none'`.
- **Client addresses**: behind a proxy, every connection comes from the
  proxy. The app believes `X-Forwarded-For` only from `TRUSTED_PROXIES` (the
  proxy's fixed address), reading it from the right and stopping at the
  first address that is not a trusted proxy, so a client cannot choose its
  address by sending the header itself. The per-address login limit then
  applies to each real client.
- **Networks**: the database sits on an `internal` network with the app
  only; the proxy cannot reach it and the app cannot call out.
- **Metrics**: Prometheus scrapes `http://app:8080/metrics` from inside the
  Docker network with `METRICS_TOKEN`; the proxy answers `/metrics` with 404.
- **Migrations** run when the app starts. With several replicas, run them as
  a separate release step instead.

Not included: backups of the `db-data` volume, log shipping, and a
Prometheus server. Connections to PostgreSQL are unencrypted, which is only
acceptable because they never leave the internal Docker network.

## Project layout

```text
src/
├── main.rs              # Entry point: `serve` (default), `create-admin`, `healthcheck`
├── lib.rs               # `run()`: bind listener, serve, graceful shutdown
├── config.rs            # Typed, validated environment configuration
├── domain/              # Entities, schedules, decision engine (`access.rs`), events
├── application/         # Use-case services (incl. access decisions), ports, errors
├── infrastructure/
│   ├── postgres/        # Pool, migrations, Pg*Repository, PgAccessDataSource
│   ├── auth/            # Argon2id hasher, JWT issuer
│   ├── event_hub.rs     # In-process broadcast of live events
│   ├── metrics.rs       # Prometheus recorder and metric descriptions
│   └── ...              # SystemClock, logging (pretty/JSON), shutdown
├── interfaces/http/     # Router, state, extractors, one module per resource,
│                        # observability.rs (request IDs, HTTP metrics, /metrics),
│                        # client_ip.rs (X-Forwarded-For), dashboard.rs (static files)
├── interfaces/background.rs # Liveness monitor (time-driven entry point)
└── simulator/           # Simulated controllers (outside the layers: plays the devices)
migrations/              # Versioned SQL migrations (applied at startup)
deploy/Caddyfile         # Reverse proxy: HTTPS, HSTS, compression
Dockerfile               # Production image (dashboard + API, distroless)
docker-compose.yml       # Development database
docker-compose.prod.yml  # Production-style stack: proxy, app, database
tests/
├── common/mod.rs        # Test database harness
├── health.rs            # Health endpoint tests (in-process router)
├── postgres_*.rs        # Repository tests against real PostgreSQL
├── use_cases.rs         # Services wired to PostgreSQL + the system clock
├── access_decisions.rs  # End-to-end: policy setup -> decisions -> audit log
├── postgres_auth.rs     # Administrators, refresh tokens (incl. races), audit log
├── http_auth.rs         # Login/refresh/logout/roles over HTTP
├── http_management.rs   # Management API: access rules, workflows, errors
├── websocket.rs         # Live event stream over a real socket
├── postgres_controllers.rs # Controller storage, expiry boundary
├── http_device.rs       # Device API, credentials, disconnect/reconnect
├── http_simulator.rs    # Simulator: swipes, outage detection, reconnect
├── observability.rs     # Request IDs, /metrics protection and labels
└── deployment.rs        # Dashboard serving, client addresses behind a proxy
```

Dependencies point inwards: `interfaces → application → domain`, and
`infrastructure` implements traits defined by `application`.

## Testing

`cargo test` runs unit tests and integration tests. The repository and
readiness tests need the compose PostgreSQL running (`docker compose up -d`)
and `TEST_DATABASE_URL` set (it is already in `.env.example`).

Each database test creates its own throwaway database named `sac_test_<uuid>`,
applies the migrations, and drops the database when it passes, so tests run
in parallel and never touch the application database. As a safety rail, the
harness only connects through `TEST_DATABASE_URL` and refuses any host other
than `localhost`, `127.0.0.1` or `::1`.

A failing test keeps its database so you can inspect it. To remove leftovers:

```sh
docker compose exec -T postgres psql -U sac -d postgres -Atc \
  "SELECT 'DROP DATABASE \"' || datname || '\";' FROM pg_database WHERE datname LIKE 'sac_test_%'" \
  | docker compose exec -T postgres psql -U sac -d postgres
```

## Quality checks

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo audit

cd dashboard
npm run lint && npm run typecheck && npm test && npm run build
npm audit
```

## Troubleshooting

| Symptom | Cause and fix |
| ------- | ------------- |
| `JWT_SECRET must be at least 32 bytes` / `is still the example placeholder` | Put the output of `openssl rand -hex 32` in `.env`. |
| `docker compose up` fails: port is already allocated | Another PostgreSQL uses the port. Set `POSTGRES_PORT` in `.env` and the same port in `DATABASE_URL` and `TEST_DATABASE_URL`. |
| macOS: `cargo build` fails with "You have not agreed to the Xcode license agreements" | Run `sudo xcodebuild -license accept`, or build with the Command Line Tools instead: `DEVELOPER_DIR=/Library/Developer/CommandLineTools cargo build`. |
| Database tests fail with "TEST_DATABASE_URL must be set" or a host error | Set `TEST_DATABASE_URL` to a **local** server (see `.env.example`); the harness refuses other hosts on purpose. |
| Production stack: `Pool overlaps with other one on this address space` | Another Docker network uses `10.83.83.0/24`. Set `PROXY_SUBNET` and `PROXY_IP` in `.env.production`. |
| Login answers `429` | The login throttle: 5 failures per name or 20 per client address in 15 minutes. Wait for `Retry-After`, or restart the server (the throttle is in memory). |
| Monitor page shows "Reconnecting…" | The backend is not running or restarted; the stream reconnects by itself with backoff. |
