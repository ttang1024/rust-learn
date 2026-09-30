//! Shared handler state: every application service, wired to PostgreSQL.
//!
//! This is the composition root of the HTTP server, the one place that
//! knows which infrastructure implements which port.

use std::{ops::Deref, sync::Arc};

use metrics_exporter_prometheus::PrometheusHandle;
use sqlx::PgPool;
use tokio::sync::{Semaphore, watch};

use crate::{
    application::{
        AccessDecisionService, AccessEventService, AccessGroupService, AuditTrail, AuthService,
        BoxError, CardService, Clock, ControllerService, DoorService, LoginThrottle,
        PermissionService, ScheduleService, ThrottlePolicy, UserService,
    },
    config::{
        AuthConfig, DEFAULT_CONTROLLER_TIMEOUT_SECONDS, DEFAULT_WS_MAX_CONNECTIONS, MetricsToken,
        TrustedProxies,
    },
    infrastructure::{
        auth::{Argon2PasswordHasher, JwtTokenIssuer, RandomSecrets},
        event_hub::BroadcastEventHub,
        metrics,
        postgres::{
            PgAccessDataSource, PgAccessEventRepository, PgAccessGroupRepository,
            PgAdministratorRepository, PgAuditLogRepository, PgCardRepository,
            PgControllerRepository, PgDoorRepository, PgPermissionRepository,
            PgRefreshTokenRepository, PgScheduleRepository, PgUserRepository,
        },
    },
};

pub type PgAuthService = AuthService<
    PgAdministratorRepository,
    PgRefreshTokenRepository,
    PgAuditLogRepository,
    Argon2PasswordHasher,
>;

pub struct Services {
    pub db: PgPool,
    pub clock: Arc<dyn Clock>,
    pub auth: PgAuthService,
    pub users: UserService<PgUserRepository>,
    pub cards: CardService<PgCardRepository, PgUserRepository>,
    pub doors: DoorService<PgDoorRepository>,
    pub controllers: ControllerService<PgControllerRepository, PgDoorRepository>,
    pub groups: AccessGroupService<PgAccessGroupRepository>,
    pub schedules: ScheduleService<PgScheduleRepository>,
    pub permissions: PermissionService<PgPermissionRepository>,
    pub access: AccessDecisionService<PgAccessDataSource, PgAccessEventRepository>,
    pub events: AccessEventService<PgAccessEventRepository>,
    pub audit: AuditTrail<PgAuditLogRepository>,
    /// Live access events for WebSocket subscribers.
    pub event_hub: BroadcastEventHub,
    /// Flips to `true` when the server stops, so long-lived connections
    /// (WebSockets) can close cleanly instead of holding up shutdown.
    pub shutdown: watch::Sender<bool>,
    /// One permit per open event-stream WebSocket: caps the memory and
    /// tasks that long-lived connections can pin.
    pub ws_permits: Arc<Semaphore>,
    /// Slows password guessing on `/auth/login`.
    pub login_throttle: LoginThrottle,
    /// Whether the refresh-token cookie carries the `Secure` attribute.
    pub cookie_secure: bool,
    /// Renders the recorded metrics for `GET /metrics`.
    pub metrics: PrometheusHandle,
    /// Enables `GET /metrics` (for scrapers presenting it); `None` = no endpoint.
    pub metrics_token: Option<MetricsToken>,
    /// Proxies whose `X-Forwarded-For` is believed (see `ClientIp`).
    pub trusted_proxies: TrustedProxies,
}

/// Server settings beyond authentication, with sensible defaults so callers
/// (and tests) set only what they care about.
#[derive(Debug, Clone)]
pub struct AppOptions {
    pub controller_timeout: chrono::Duration,
    pub max_ws_connections: usize,
    pub metrics_token: Option<MetricsToken>,
    pub trusted_proxies: TrustedProxies,
}

impl Default for AppOptions {
    fn default() -> Self {
        Self {
            controller_timeout: chrono::Duration::seconds(
                DEFAULT_CONTROLLER_TIMEOUT_SECONDS.into(),
            ),
            max_ws_connections: DEFAULT_WS_MAX_CONNECTIONS,
            metrics_token: None,
            trusted_proxies: TrustedProxies::default(),
        }
    }
}

/// Cheap to clone (one `Arc`), as Axum clones the state for every request.
///
/// `Deref` lets handlers write `state.users` instead of `state.0.users`.
#[derive(Clone)]
pub struct AppState(Arc<Services>);

impl Deref for AppState {
    type Target = Services;

    fn deref(&self) -> &Services {
        &self.0
    }
}

impl AppState {
    /// Tells long-lived connections to close. Idempotent.
    pub fn begin_shutdown(&self) {
        self.shutdown.send_replace(true);
    }

    /// Resolves once shutdown has begun, including if it began *before* this
    /// call. (`watch::Receiver::changed` would miss a change that happened
    /// before subscribing; `wait_for` checks the current value first.)
    ///
    /// `wait_for` returns a read guard on the watched value. Awaiting it here
    /// and returning `()` drops the guard inside this function; used directly
    /// in `select!`, the guard would stay alive while the branch body runs
    /// more `.await`s. Holding a lock guard across `.await` can block other
    /// tasks, and the compiler rejects it because the guard is not `Send`.
    pub async fn shutdown_requested(&self) {
        let mut receiver = self.shutdown.subscribe();
        // An error means the sender is gone, which also means "stop".
        let _ = receiver.wait_for(|stopping| *stopping).await;
    }

    pub fn new(
        pool: PgPool,
        config: &AuthConfig,
        options: AppOptions,
        clock: Arc<dyn Clock>,
    ) -> Result<Self, BoxError> {
        let tokens = Arc::new(JwtTokenIssuer::new(
            &config.jwt_secret,
            config.access_token_ttl,
        ));
        let users = PgUserRepository::new(pool.clone());
        let events = PgAccessEventRepository::new(pool.clone());
        let audit_log = PgAuditLogRepository::new(pool.clone());
        let event_hub = BroadcastEventHub::default();

        let services = Services {
            auth: AuthService::new(
                PgAdministratorRepository::new(pool.clone()),
                PgRefreshTokenRepository::new(pool.clone()),
                audit_log.clone(),
                Argon2PasswordHasher::new()?,
                tokens,
                clock.clone(),
                config.refresh_token_ttl,
            ),
            users: UserService::new(users.clone(), clock.clone()),
            cards: CardService::new(PgCardRepository::new(pool.clone()), users, clock.clone()),
            doors: DoorService::new(PgDoorRepository::new(pool.clone()), clock.clone()),
            controllers: ControllerService::new(
                PgControllerRepository::new(pool.clone()),
                PgDoorRepository::new(pool.clone()),
                Arc::new(RandomSecrets),
                clock.clone(),
                options.controller_timeout,
            ),
            groups: AccessGroupService::new(PgAccessGroupRepository::new(pool.clone())),
            schedules: ScheduleService::new(PgScheduleRepository::new(pool.clone())),
            permissions: PermissionService::new(
                PgPermissionRepository::new(pool.clone()),
                clock.clone(),
            ),
            access: AccessDecisionService::new(
                PgAccessDataSource::new(pool.clone()),
                events.clone(),
                Arc::new(event_hub.clone()),
                clock.clone(),
            ),
            events: AccessEventService::new(events),
            audit: AuditTrail::new(audit_log, clock.clone()),
            event_hub,
            shutdown: watch::Sender::new(false),
            ws_permits: Arc::new(Semaphore::new(options.max_ws_connections)),
            metrics: metrics::install(),
            metrics_token: options.metrics_token,
            trusted_proxies: options.trusted_proxies,
            login_throttle: LoginThrottle::new(clock.clone(), ThrottlePolicy::default()),
            cookie_secure: config.cookie_secure,
            clock,
            db: pool,
        };
        Ok(Self(Arc::new(services)))
    }
}
