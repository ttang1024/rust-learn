//! Smart Access Control: a software-simulated access control backend.
//!
//! Layers (Clean Architecture). Dependencies point inwards only:
//!
//! ```text
//! interfaces ──► application ──► domain
//!      │               ▲
//!      └──► infrastructure (implements application traits)
//! ```
//!
//! `simulator` stands outside these layers: it plays the role of the door
//! controllers, reaching the system only through a `ControllerLink`.

pub mod application;
pub mod config;
pub mod domain;
pub mod infrastructure;
pub mod interfaces;
pub mod simulator;

use std::{error::Error, io, net::SocketAddr, path::PathBuf, sync::Arc};

use sqlx::PgPool;
use thiserror::Error;
use tokio::net::TcpListener;

use crate::{
    application::{ApplicationError, BoxError, CreateAdministrator},
    config::AppConfig,
    domain::Administrator,
    infrastructure::{clock::SystemClock, postgres, shutdown},
    interfaces::{
        background::spawn_liveness_monitor,
        http::{AppOptions, AppState, RouterOptions, cors_layer, router_with},
    },
    simulator::Simulator,
};

#[derive(Debug, Error)]
pub enum StartupError {
    #[error("failed to connect to the database")]
    Database(#[source] sqlx::Error),

    #[error("failed to apply database migrations")]
    Migrate(#[source] sqlx::migrate::MigrateError),

    #[error("failed to initialise authentication")]
    Auth(#[source] BoxError),

    #[error("STATIC_DIR {0:?} does not contain index.html (build the dashboard first)")]
    Dashboard(PathBuf),

    #[error("HTTP server error")]
    Server(#[from] io::Error),
}

#[derive(Debug, Error)]
pub enum CommandError {
    #[error(transparent)]
    Startup(#[from] StartupError),

    #[error(transparent)]
    Application(#[from] ApplicationError),
}

/// Renders an error and all its causes: "outer: inner: root".
///
/// `Display` on a wrapper error only shows the top level; walking
/// `Error::source()` shows why it happened.
pub fn error_chain(err: &dyn Error) -> String {
    let mut message = err.to_string();
    let mut source = err.source();
    while let Some(cause) = source {
        message.push_str(": ");
        message.push_str(&cause.to_string());
        source = cause.source();
    }
    message
}

/// Connects to the database, applies pending migrations and wires the
/// services.
///
/// Migrations run at startup for a simple demo workflow. A production
/// deployment would usually run them as a separate release step.
async fn start(config: &AppConfig) -> Result<AppState, StartupError> {
    let pool: PgPool = postgres::connect(&config.database)
        .await
        .map_err(StartupError::Database)?;
    postgres::MIGRATOR
        .run(&pool)
        .await
        .map_err(StartupError::Migrate)?;
    tracing::info!("database ready, migrations applied");

    AppState::new(
        pool,
        &config.auth,
        AppOptions {
            controller_timeout: config.controller_timeout,
            max_ws_connections: config.ws_max_connections,
            metrics_token: config.metrics_token.clone(),
            trusted_proxies: config.trusted_proxies.clone(),
        },
        Arc::new(SystemClock),
    )
    .map_err(StartupError::Auth)
}

/// Serves the API until a shutdown signal arrives.
pub async fn run(config: AppConfig) -> Result<(), StartupError> {
    if let Some(dir) = &config.static_dir {
        // Fail at startup, not with a 404 on the first page load.
        if !dir.join("index.html").is_file() {
            return Err(StartupError::Dashboard(dir.clone()));
        }
        tracing::info!(dir = %dir.display(), "serving the dashboard");
    }

    let state = start(&config).await?;
    let pool = state.db.clone();

    let monitor = spawn_liveness_monitor(state.clone());

    let listener = TcpListener::bind(config.socket_addr()).await?;
    tracing::info!(addr = %listener.local_addr()?, "listening");

    let simulator = config
        .simulator_enabled
        .then(|| Arc::new(Simulator::default()));
    if simulator.is_some() {
        tracing::warn!(
            "SIMULATOR ENABLED: /api/v1/simulator/* can create simulated controllers; \
             do not enable in production"
        );
    }
    let mut app = router_with(
        state.clone(),
        RouterOptions {
            simulator: simulator.clone(),
            dashboard_dir: config.static_dir.clone(),
        },
    );
    if let Some(cors) = cors_layer(&config.cors_allowed_origins) {
        app = app.layer(cors);
    }
    // On Ctrl+C / SIGTERM: stop accepting connections, tell WebSocket
    // clients to go away, then wait for in-flight requests to finish.
    let stopping = state.clone();
    // Record each connection's peer address (used by the login throttle).
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(async move {
        shutdown::signal().await;
        stopping.begin_shutdown();
    })
    .await?;

    // Simulated controllers disconnect cleanly (their doors go offline)
    // while the database is still available.
    if let Some(simulator) = &simulator {
        simulator.stop_all().await;
    }
    // The monitor stops on the shutdown signal; wait so it never runs a
    // query against a closing pool.
    let _ = monitor.await;
    // Let in-flight queries finish and close connections politely.
    pool.close().await;
    Ok(())
}

/// The `create-admin` command: bootstraps an administrator account.
pub async fn create_administrator(
    config: AppConfig,
    cmd: CreateAdministrator,
) -> Result<Administrator, CommandError> {
    let state = start(&config).await?;
    let admin = state.auth.create_administrator(cmd).await?;
    state.db.close().await;
    Ok(admin)
}
