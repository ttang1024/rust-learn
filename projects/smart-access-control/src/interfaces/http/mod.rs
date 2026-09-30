//! HTTP API: router, shared state, extractors and handlers.
//!
//! Access rules: every route except health and login needs a valid access
//! token (`AuthenticatedAdmin`, which admits viewers too); every route that
//! changes data needs the admin role (`RequireAdmin`).

mod auth;
mod cards;
mod client_ip;
mod controllers;
mod dashboard;
mod device;
mod doors;
mod error;
mod events;
mod extract;
mod groups;
mod health;
mod observability;
mod permissions;
mod schedules;
mod simulator;
mod state;
mod users;
mod websocket;

use std::{path::PathBuf, sync::Arc, time::Duration};

use axum::{
    Router,
    extract::DefaultBodyLimit,
    http::{HeaderName, HeaderValue, Method, StatusCode, header},
    routing::{get, post},
};
use tower_http::{
    cors::CorsLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    set_header::SetResponseHeaderLayer,
    timeout::TimeoutLayer,
    trace::TraceLayer,
};

pub use auth::{AuthenticatedAdmin, RequireAdmin};
pub use client_ip::{ClientIp, client_ip};
pub use device::{AuthenticatedController, CONTROLLER_KEY_HEADER};
pub use error::{ApiError, ApiJson};
pub use extract::{ApiPath, ApiQuery, Page, PageParams, double_option, page_request, subject};
pub use simulator::{InProcessLink, SimulatorState};
pub use state::{AppOptions, AppState, PgAuthService, Services};
pub use websocket::close_codes;

/// Base path for every versioned API route.
pub const API_V1: &str = "/api/v1";

/// Largest accepted request body. The API only takes small JSON documents.
pub const MAX_BODY_BYTES: usize = 64 * 1024;

/// A request that takes longer than this gets `408`, so slow clients cannot
/// hold connections and tasks indefinitely. (WebSocket upgrades answer
/// immediately; the stream itself is not subject to this.)
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Response headers added to everything the API returns, unless a handler
/// set its own. The API only serves JSON, so the content policy can forbid
/// everything: nothing it returns should ever run as a page or be framed.
const SECURITY_HEADERS: [(HeaderName, &str); 5] = [
    (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
    (header::X_FRAME_OPTIONS, "DENY"),
    (header::REFERRER_POLICY, "no-referrer"),
    (
        header::CONTENT_SECURITY_POLICY,
        "default-src 'none'; frame-ancestors 'none'",
    ),
    // Responses contain personal and security data: no shared caches.
    (header::CACHE_CONTROL, "no-store"),
];

/// Optional parts of the application.
#[derive(Default)]
pub struct RouterOptions {
    /// Adds the `/simulator/*` routes (`SIMULATOR_ENABLED=true`).
    pub simulator: Option<Arc<crate::simulator::Simulator>>,
    /// Serves the built dashboard from this directory at `/` (`STATIC_DIR`).
    pub dashboard_dir: Option<PathBuf>,
}

/// Builds the application router: the API only.
///
/// Kept separate from `run` so tests can drive it in-process.
pub fn router(state: AppState) -> Router {
    router_with(state, RouterOptions::default())
}

/// The API plus the `/simulator/*` routes.
pub fn router_with_simulator(
    state: AppState,
    simulator: Arc<crate::simulator::Simulator>,
) -> Router {
    router_with(
        state,
        RouterOptions {
            simulator: Some(simulator),
            ..RouterOptions::default()
        },
    )
}

pub fn router_with(state: AppState, options: RouterOptions) -> Router {
    let RouterOptions {
        simulator,
        dashboard_dir,
    } = options;
    let api_v1 = Router::new()
        .route("/health", get(health::live))
        .route("/health/ready", get(health::ready))
        .route("/auth/login", post(auth::login))
        .route("/auth/refresh", post(auth::refresh))
        .route("/auth/logout", post(auth::logout))
        .route("/auth/me", get(auth::me))
        .merge(users::routes())
        .merge(cards::routes())
        .merge(controllers::routes())
        .merge(device::routes())
        .merge(doors::routes())
        .merge(groups::routes())
        .merge(schedules::routes())
        .merge(permissions::routes())
        .merge(events::routes())
        .merge(websocket::routes());

    // Routers with different state types are combined after each receives
    // its state (`with_state` turns `Router<S>` into `Router<()>`).
    let mut api_v1 = api_v1.with_state(state.clone());
    if let Some(simulator) = simulator {
        api_v1 = api_v1.merge(simulator::routes().with_state(SimulatorState {
            app: state.clone(),
            simulator,
        }));
    }
    // `route_layer`: only requests that matched a route are measured, and
    // their route *pattern* is known (bounded metric labels).
    let api_v1 = api_v1
        .route_layer(axum::middleware::from_fn(observability::track_http_metrics))
        // Unknown API paths answer JSON, and never fall through to the dashboard.
        .fallback(api_not_found);

    let mut app = Router::new().nest(API_V1, api_v1);
    if state.metrics_token.is_some() {
        app = app.route(
            "/metrics",
            get(observability::metrics).with_state(state.clone()),
        );
    }

    if let Some(dir) = dashboard_dir {
        app = app.merge(dashboard::routes(&dir));
    }

    // Layers go on last so they wrap every route, the simulator's included.
    let mut app =
        app.layer(DefaultBodyLimit::max(MAX_BODY_BYTES))
            .layer(TimeoutLayer::with_status_code(
                StatusCode::REQUEST_TIMEOUT,
                REQUEST_TIMEOUT,
            ));
    for (name, value) in SECURITY_HEADERS {
        app = app.layer(SetResponseHeaderLayer::if_not_present(
            name,
            HeaderValue::from_static(value),
        ));
    }
    // Order matters: the last layer added runs first on a request.
    // 1. drop unusable incoming ids, 2. assign an id if missing,
    // 3. copy it to the response, 4. open the request span with it.
    app.layer(TraceLayer::new_for_http().make_span_with(observability::request_span))
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(axum::middleware::map_request(
            observability::drop_invalid_request_id,
        ))
}

async fn api_not_found() -> ApiError {
    ApiError::new(StatusCode::NOT_FOUND, "not_found", "no such endpoint")
}

/// CORS for a browser dashboard served from another origin (e.g. the Vite
/// dev server). `None` when no origins are configured: browsers then only
/// allow same-origin calls, the safest default.
///
/// Tokens travel in the `Authorization` header, not cookies, so credentials
/// mode is not enabled.
pub fn cors_layer(allowed_origins: &[String]) -> Option<CorsLayer> {
    if allowed_origins.is_empty() {
        return None;
    }
    let origins: Vec<HeaderValue> = allowed_origins
        .iter()
        .filter_map(|origin| HeaderValue::from_str(origin).ok())
        .collect();
    Some(
        CorsLayer::new()
            .allow_origin(origins)
            .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
            .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
            .max_age(Duration::from_secs(600)),
    )
}
