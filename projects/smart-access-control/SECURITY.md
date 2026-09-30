# Security review

Scope: the backend (`src/`), the dashboard (`dashboard/`) and their
configuration, as of Phase 8c. This is a **learning project and a
software-only simulation**. It has not had an independent security review
and must not be connected to real doors, locks or building systems.

## What is being protected

| Asset | Why it matters |
| ----- | -------------- |
| Access decisions | A wrong "granted" opens a door |
| Access policy (users, cards, groups, permissions, schedules) | Whoever can change it decides who gets in |
| Administrator sessions | They carry the power to change policy |
| Controller keys | A controller can request decisions and take doors offline |
| Audit trails (`access_events`, `admin_audit_log`) | Evidence of what happened and who did it |
| Personal data (names, emails, movements) | Privacy |

## Controls in place

### Authentication and sessions

- Passwords: Argon2id (OWASP minimum parameters), 12–128 characters,
  hashed on a blocking thread pool. The database rejects any stored value
  that is not an Argon2id PHC string.
- Every failed login answers the same `401`, and unknown usernames cost the
  same hashing time as wrong passwords (no account enumeration).
- **Login throttling** (in memory, per process): 5 failures per account name
  or 20 per client address within 15 minutes block further attempts with
  `429` + `Retry-After`, even with the correct password. Unknown names are
  throttled identically. Tables are capped at 10,000 entries.
- Access tokens: HS256 JWTs, 15 minutes, `iss`/`aud` checked, only HS256
  accepted (no `alg: none` or algorithm confusion). `JWT_SECRET` must be at
  least 32 bytes and is refused if it is the `.env.example` placeholder.
- Refresh tokens: 256 random bits, stored only as SHA-256, single use.
  Replaying a rotated token revokes all of that administrator's sessions.
  Rotation is one atomic SQL statement (tested with ten concurrent
  requests).
- **The dashboard never sees the refresh token.** It logs in with
  `"session": "cookie"`, and the server sets
  `sac_refresh=…; HttpOnly; Secure; SameSite=Strict; Path=/api/v1/auth`.
  Page scripts cannot read it (verified in a browser), so an XSS bug cannot
  steal a long-lived session. The access token lives in memory only; nothing
  is written to `localStorage` or `sessionStorage`.
- CSRF: the cookie is `SameSite=Strict` and only sent to `/api/v1/auth/*`;
  those endpoints additionally require a JSON body, which a cross-site HTML
  form cannot send (tested: a form post gets `415`).

### Authorization

- Every route except health and login/refresh/logout requires a token;
  every write requires the `admin` role. A test sends every one of the
  write routes as an anonymous caller and as a viewer.
- Controllers use a separate credential (`X-Controller-Key`, stored
  hashed); admin tokens and controller keys are not interchangeable.
  A controller can only act on its own doors.
- WebSocket subscribers authenticate in the first message (not the URL,
  which would end up in logs), and the stream closes when the token expires.

### Access decisions

- Deny by default; every missing fact denies. The engine re-checks that the
  loaded card, holder and door match the request, so a faulty data loader
  cannot grant access. The decision time is the server's clock, never a
  device-supplied timestamp.
- A decision is only returned after its event is stored; if storing fails
  the request fails, so a door never opens without an audit record.
- Both audit tables are append-only, enforced by database triggers
  (`UPDATE`, `DELETE` and `TRUNCATE` are rejected even from a SQL console).

### Input, transport and resource limits

- All SQL is parameterized. Unknown JSON fields are rejected (`422`).
- Request bodies: 64 KiB. Requests: 30 s timeout (`408`).
- Live-event WebSockets: capped by `WS_MAX_CONNECTIONS` (default 256);
  unauthenticated sockets are closed after 5 s, slow consumers after 10 s.
- Response headers on everything: `X-Content-Type-Options: nosniff`,
  `X-Frame-Options: DENY`, `Referrer-Policy: no-referrer`,
  `Content-Security-Policy: default-src 'none'; frame-ancestors 'none'`,
  `Cache-Control: no-store`.
- CORS: off unless `CORS_ALLOWED_ORIGINS` lists exact origins; `*` is
  refused.

### Secrets and logging

- Secrets come from the environment; `.env` is git-ignored. `DatabaseUrl`,
  `JwtSecret`, `Password`, `PasswordHash`, token pairs and issued controller
  keys have redacted `Debug` output; request types with secrets have no
  `Debug` at all.
- A review of every log statement found no secret values logged. Card
  numbers are logged nowhere in normal operation.
- Internal errors are logged in full but answered with a generic `500`.
- Client-supplied `X-Request-Id` values are only kept if short and made of
  safe characters, so they cannot inject fake log lines or fields.
- `/metrics` exists only when `METRICS_TOKEN` is set and requires it as a
  bearer token (compared in constant time). Metric labels never contain IDs,
  card numbers or usernames.

### Deployment

- The container runs as a non-root user on a read-only filesystem with all
  Linux capabilities dropped and `no-new-privileges`; the image has no
  shell or package manager.
- The database has no published port and shares an `internal` network with
  the app only.
- `X-Forwarded-For` is believed only from `TRUSTED_PROXIES`, read
  right-to-left (`client_ip.rs`): a client cannot pick its own address to
  escape the per-address login limit (tested through the real proxy).
- The dashboard is served with `default-src 'self'` and no inline scripts;
  `/metrics` is not reachable through the proxy.

### Dependencies

- `cargo audit`: no known vulnerabilities. The JWT library's `rust_crypto`
  backend pulled in the `rsa` crate (RUSTSEC-2023-0071, unfixed timing
  side channel); we never used RSA, but switched to the `aws_lc_rs` backend
  so the crate is not present at all.
- `npm audit`: no known vulnerabilities (runtime and development).

## Known risks and limitations

These are accepted for a demonstration system and should be addressed
before anything production-like:

1. **Throttling is per process and in memory.** A restart clears it, and
   multiple instances do not share counts.
2. **Client addresses depend on `TRUSTED_PROXIES` being right.** Too narrow
   (proxy not listed): all clients share the proxy's address and its login
   limit. Too wide (e.g. a whole network others can join): those hosts can
   forge `X-Forwarded-For`. The compose stack trusts exactly one fixed
   address.
3. **Account lockout can be abused**: anyone can block a known admin name
   for 15 minutes by failing five logins. That is the usual trade-off of
   throttling by name; there is no CAPTCHA or second factor.
4. **No multi-factor authentication** for administrators.
5. **Access tokens are stateless**: a disabled administrator keeps access
   for up to 15 minutes, until the token expires.
6. **Audit writes are not atomic with the change they describe**: if
   writing the entry fails, the change stands and the failure is logged.
7. **Internal error logs may contain values from constraint violations**
   (e.g. a duplicate card number or username), never passwords or tokens.
8. **HTTPS is terminated by the proxy.** The compose stack's Caddy sends
   `Strict-Transport-Security` and redirects HTTP to HTTPS; the backend
   itself speaks plain HTTP and must not be exposed directly. Database
   connections are unencrypted and stay on an internal Docker network.
9. **The simulator** (`SIMULATOR_ENABLED=true`) lets admins create
   controllers and generate decisions. It is off by default and logs a
   warning when enabled; keep it off outside development.
10. **Controller keys are bearer secrets over HTTP**: without TLS they can
    be intercepted, like any other credential.

## Reporting

This is a personal learning repository. Please report issues through the
repository's issue tracker rather than exploiting them.
