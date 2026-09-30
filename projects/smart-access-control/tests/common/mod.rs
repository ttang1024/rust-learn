//! Test database harness shared by the integration tests.
//!
//! Every test gets its own freshly created and migrated database, so tests
//! run in parallel without seeing each other's rows.
//!
//! Safety rails, because these tests create and drop databases:
//! - The server comes from `TEST_DATABASE_URL`, never the app's `DATABASE_URL`.
//! - Only `localhost` / `127.0.0.1` / `::1` hosts are accepted.
//! - Only databases this harness created (`sac_test_<uuid>`) are ever dropped.

// Each integration test file is compiled as its own crate and uses only some
// of these helpers; the rest would otherwise be reported as dead code.
#![allow(dead_code)]

use std::{str::FromStr, sync::Arc};

use chrono::Duration;
use smart_access_control::{
    config::{AuthConfig, JwtSecret},
    infrastructure::{clock::SystemClock, postgres::MIGRATOR},
    interfaces::http::{AppOptions, AppState},
};
use sqlx::{
    AssertSqlSafe, Connection, Executor, PgConnection, PgPool,
    postgres::{PgConnectOptions, PgPoolOptions},
};
use uuid::Uuid;

const LOCAL_HOSTS: [&str; 3] = ["localhost", "127.0.0.1", "::1"];

pub struct TestDb {
    pool: PgPool,
    server: PgConnectOptions,
    name: String,
}

impl TestDb {
    pub async fn new() -> Self {
        let _ = dotenvy::dotenv();
        let url = std::env::var("TEST_DATABASE_URL").expect(
            "TEST_DATABASE_URL must be set to a local PostgreSQL server \
             (see .env.example and the README's Testing section)",
        );
        let server = PgConnectOptions::from_str(&url)
            .expect("TEST_DATABASE_URL is not a valid Postgres URL");
        assert!(
            LOCAL_HOSTS.contains(&server.get_host()),
            "refusing to create test databases on non-local host {:?}",
            server.get_host()
        );

        // The name is built only from a UUID, so interpolating it is safe.
        // `AssertSqlSafe` is how SQLx makes us acknowledge that explicitly.
        let name = format!("sac_test_{}", Uuid::now_v7().simple());
        let mut conn = PgConnection::connect_with(&server)
            .await
            .expect("connect to the test server");
        conn.execute(AssertSqlSafe(format!(r#"CREATE DATABASE "{name}""#)))
            .await
            .expect("create test database");
        conn.close().await.expect("close admin connection");

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect_with(server.clone().database(&name))
            .await
            .expect("connect to test database");
        MIGRATOR.run(&pool).await.expect("apply migrations");

        Self { pool, server, name }
    }

    pub fn pool(&self) -> PgPool {
        self.pool.clone()
    }

    /// Drops the test database.
    ///
    /// This is an explicit `async fn` rather than a `Drop` impl because Rust
    /// has no async destructors. As a side effect, a failing test (which
    /// panics before reaching `cleanup`) keeps its database for inspection.
    pub async fn cleanup(self) {
        self.pool.close().await;
        let mut conn = PgConnection::connect_with(&self.server)
            .await
            .expect("connect to the test server");
        conn.execute(AssertSqlSafe(format!(
            r#"DROP DATABASE IF EXISTS "{}" WITH (FORCE)"#,
            self.name
        )))
        .await
        .expect("drop test database");
    }
}

/// Authentication settings for tests. The secret is a fixed test value.
pub fn auth_config() -> AuthConfig {
    AuthConfig {
        jwt_secret: JwtSecret::new("integration-test-secret-integration-test").unwrap(),
        access_token_ttl: Duration::minutes(15),
        refresh_token_ttl: Duration::days(7),
        cookie_secure: true,
    }
}

/// HTTP application state wired to `pool`, as the server builds it.
pub fn app_state(pool: PgPool) -> AppState {
    AppState::new(
        pool,
        &auth_config(),
        AppOptions {
            controller_timeout: Duration::seconds(30),
            ..AppOptions::default()
        },
        Arc::new(SystemClock),
    )
    .unwrap()
}
