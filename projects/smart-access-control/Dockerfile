# syntax=docker/dockerfile:1
#
# Production image: the API binary plus the built dashboard, in a minimal
# runtime image (no shell, no package manager, non-root).
#
#   docker build -t smart-access-control .
#
# Three stages; only the last one ships.

# ---- 1. Dashboard: static files -------------------------------------------
FROM node:24-alpine AS dashboard
WORKDIR /dashboard
# Dependencies first: this layer is reused until the lock file changes.
COPY dashboard/package.json dashboard/package-lock.json ./
RUN npm ci --no-audit --no-fund
COPY dashboard/ ./
RUN npm run build

# ---- 2. API: release binary -----------------------------------------------
# Same Debian release (trixie) as the runtime image, so the binary links
# against the same glibc.
FROM rust:1.98.1-slim-trixie AS api
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
# Embedded into the binary by `sqlx::migrate!` at compile time.
COPY migrations/ migrations/
# Cache mounts keep the crate registry and build outputs between builds
# without putting them in the image. `target/` lives only in the cache, so
# the binary is copied out in the same step.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    cargo build --release --locked \
    && cp target/release/smart-access-control /smart-access-control

# ---- 3. Runtime -----------------------------------------------------------
# Distroless "cc": glibc, libgcc, CA certificates and nothing else. The
# `nonroot` tag runs as uid 65532.
FROM gcr.io/distroless/cc-debian13:nonroot
COPY --from=api /smart-access-control /usr/local/bin/smart-access-control
COPY --from=dashboard /dashboard/dist /app/dashboard

ENV APP_HOST=0.0.0.0 \
    APP_PORT=8080 \
    STATIC_DIR=/app/dashboard \
    LOG_FORMAT=json \
    RUST_LOG=smart_access_control=info,tower_http=info
EXPOSE 8080
USER nonroot
HEALTHCHECK --interval=10s --timeout=5s --start-period=10s --retries=3 \
    CMD ["smart-access-control", "healthcheck"]
ENTRYPOINT ["smart-access-control"]
