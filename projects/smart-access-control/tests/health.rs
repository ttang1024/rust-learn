//! API integration tests for the health endpoints.
//!
//! The router is exercised in-process with `tower::ServiceExt::oneshot`,
//! so no port is bound and tests can run in parallel.

mod common;

use std::time::Duration;

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use common::{TestDb, app_state};
use http_body_util::BodyExt;
use serde_json::Value;
use smart_access_control::interfaces::http::router;
use sqlx::{PgPool, postgres::PgPoolOptions};
use tower::ServiceExt;

/// A pool that never connects until used, pointing at a closed port.
/// Lets liveness tests run without any database.
fn unreachable_pool() -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(1))
        .connect_lazy("postgres://nobody:nothing@127.0.0.1:1/none")
        .unwrap()
}

fn app(db: PgPool) -> Router {
    router(app_state(db))
}

async fn get(app: Router, uri: &str) -> (StatusCode, Value) {
    let response = app
        .oneshot(Request::get(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json = serde_json::from_slice(&body).unwrap_or(Value::Null);
    (status, json)
}

#[tokio::test]
async fn liveness_does_not_need_the_database() {
    let (status, json) = get(app(unreachable_pool()), "/api/v1/health").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["status"], "ok");
    assert_eq!(json["version"], env!("CARGO_PKG_VERSION"));
}

#[tokio::test]
async fn readiness_fails_without_database() {
    let (status, json) = get(app(unreachable_pool()), "/api/v1/health/ready").await;

    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    // Generic body: no connection details leak to clients.
    assert_eq!(json, serde_json::json!({ "status": "unavailable" }));
}

#[tokio::test]
async fn readiness_succeeds_with_database() {
    let db = TestDb::new().await;

    let (status, json) = get(app(db.pool()), "/api/v1/health/ready").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["status"], "ready");
    db.cleanup().await;
}

#[tokio::test]
async fn unknown_route_returns_not_found() {
    let (status, _) = get(app(unreachable_pool()), "/api/v1/does-not-exist").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn health_is_only_served_under_versioned_path() {
    let (status, _) = get(app(unreachable_pool()), "/health").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn every_response_carries_security_headers() {
    for uri in ["/api/v1/health", "/api/v1/users", "/api/v1/no-such-route"] {
        let response = app(unreachable_pool())
            .oneshot(Request::get(uri).body(Body::empty()).unwrap())
            .await
            .unwrap();
        let headers = response.headers();
        // Success, 401 and 404 alike.
        assert_eq!(headers["x-content-type-options"], "nosniff", "{uri}");
        assert_eq!(headers["x-frame-options"], "DENY", "{uri}");
        assert_eq!(headers["referrer-policy"], "no-referrer", "{uri}");
        assert_eq!(
            headers["content-security-policy"], "default-src 'none'; frame-ancestors 'none'",
            "{uri}"
        );
        assert_eq!(headers["cache-control"], "no-store", "{uri}");
    }
}
