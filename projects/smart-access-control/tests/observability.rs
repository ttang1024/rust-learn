//! Request ids and the Prometheus metrics endpoint.

mod common;

use std::sync::Arc;

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
};
use common::{TestDb, app_state, auth_config};
use http_body_util::BodyExt;
use serde_json::json;
use smart_access_control::{
    application::{AccessRequest, CreateAdministrator},
    config::MetricsToken,
    domain::{DoorId, Role},
    infrastructure::clock::SystemClock,
    interfaces::http::{AppOptions, AppState, router},
};
use tower::ServiceExt;

const TOKEN: &str = "prometheus-scrape-token";

fn with_metrics(db: &TestDb) -> AppState {
    AppState::new(
        db.pool(),
        &auth_config(),
        AppOptions {
            metrics_token: Some(MetricsToken::new(TOKEN).unwrap()),
            ..AppOptions::default()
        },
        Arc::new(SystemClock),
    )
    .unwrap()
}

async fn get(
    app: &Router,
    uri: &str,
    headers: &[(&str, &str)],
) -> (StatusCode, axum::http::HeaderMap, String) {
    let mut request = Request::get(uri);
    for (name, value) in headers {
        request = request.header(*name, *value);
    }
    let response = app
        .clone()
        .oneshot(request.body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    (status, headers, String::from_utf8_lossy(&body).into_owned())
}

async fn scrape(app: &Router) -> String {
    let (status, headers, body) = get(
        app,
        "/metrics",
        &[("authorization", &format!("Bearer {TOKEN}"))],
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        headers[header::CONTENT_TYPE]
            .to_str()
            .unwrap()
            .starts_with("text/plain")
    );
    body
}

#[tokio::test]
async fn every_response_gets_a_request_id() {
    let db = TestDb::new().await;
    let app = router(app_state(db.pool()));

    let (_, generated, _) = get(&app, "/api/v1/health", &[]).await;
    let id = generated["x-request-id"].to_str().unwrap();
    assert_eq!(id.len(), 36, "a UUID: {id}");

    // A well-formed id from a proxy is kept, so logs can be correlated.
    let (_, kept, _) = get(&app, "/api/v1/health", &[("x-request-id", "edge-7f3a.42")]).await;
    assert_eq!(kept["x-request-id"], "edge-7f3a.42");

    // Anything unusual is replaced before it can reach the logs.
    let long = "a".repeat(100);
    for bad in ["", long.as_str(), "has space", "semi;colon", "<script>"] {
        let (_, replaced, _) = get(&app, "/api/v1/health", &[("x-request-id", bad)]).await;
        let id = replaced["x-request-id"].to_str().unwrap();
        assert_ne!(id, bad);
        assert_eq!(id.len(), 36, "{bad:?} should be replaced by a UUID");
    }

    // Errors carry an id too.
    let (status, headers, _) = get(&app, "/api/v1/users", &[]).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert!(headers.contains_key("x-request-id"));
    db.cleanup().await;
}

#[tokio::test]
async fn metrics_endpoint_does_not_exist_without_a_token() {
    let db = TestDb::new().await;
    let app = router(app_state(db.pool()));

    let (status, _, _) = get(&app, "/metrics", &[]).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    db.cleanup().await;
}

#[tokio::test]
async fn metrics_require_the_scrape_token() {
    let db = TestDb::new().await;
    let app = router(with_metrics(&db));

    for headers in [
        vec![],
        vec![("authorization", "Bearer wrong-token-wrong-token")],
    ] {
        let (status, _, body) = get(&app, "/metrics", &headers).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert!(!body.contains("http_requests_total"));
    }
    db.cleanup().await;
}

#[tokio::test]
async fn http_metrics_use_route_patterns_not_raw_paths() {
    let db = TestDb::new().await;
    let app = router(with_metrics(&db));
    let raw_id = "0190a0a0-dead-7000-8000-00000000beef";

    get(&app, "/api/v1/health", &[]).await;
    get(&app, &format!("/api/v1/users/{raw_id}"), &[]).await; // 401, still measured

    let body = scrape(&app).await;
    assert!(
        body.contains(r#"http_requests_total{method="GET",path="/api/v1/health",status="200"}"#),
        "{body}"
    );
    assert!(
        body.contains(r#"path="/api/v1/users/{id}",status="401""#),
        "{body}"
    );
    assert!(!body.contains(raw_id), "raw ids must never become labels");
    assert!(body.contains("http_request_duration_seconds_bucket"));
    db.cleanup().await;
}

#[tokio::test]
async fn business_metrics_are_recorded() {
    let db = TestDb::new().await;
    let state = with_metrics(&db);
    let app = router(state.clone());

    // A decision (denied: unknown card and door) and a failed login.
    state
        .access
        .decide(AccessRequest {
            card_number: "CARD-404".into(),
            door_id: DoorId::generate(),
        })
        .await
        .unwrap();
    state
        .auth
        .create_administrator(CreateAdministrator {
            username: "ops".into(),
            password: "correct horse battery staple".into(),
            role: Role::Admin,
        })
        .await
        .unwrap();
    let login = Request::post("/api/v1/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "username": "ops", "password": "nope" }).to_string(),
        ))
        .unwrap();
    app.clone().oneshot(login).await.unwrap();

    let body = scrape(&app).await;
    assert!(
        body.contains(r#"access_decisions_total{decision="denied",reason="unknown_card"}"#),
        "{body}"
    );
    assert!(
        body.contains(r#"auth_login_attempts_total{outcome="failure"}"#),
        "{body}"
    );
    assert!(body.contains("# HELP access_decisions_total"), "described");
    db.cleanup().await;
}
