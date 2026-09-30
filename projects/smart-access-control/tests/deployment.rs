//! Production wiring: the dashboard served by the API, and client
//! addresses behind a reverse proxy.

mod common;

use std::{net::SocketAddr, path::PathBuf, sync::Arc};

use axum::{
    Router,
    body::Body,
    extract::connect_info::MockConnectInfo,
    http::{Request, StatusCode, header},
};
use common::{TestDb, app_state, auth_config};
use http_body_util::BodyExt;
use serde_json::json;
use smart_access_control::{
    config::{IpNetwork, TrustedProxies},
    infrastructure::clock::SystemClock,
    interfaces::http::{AppOptions, AppState, RouterOptions, router_with},
};
use tower::ServiceExt;

/// A throwaway directory shaped like `dashboard/dist`.
struct BuiltDashboard(PathBuf);

impl BuiltDashboard {
    fn new() -> Self {
        let dir = std::env::temp_dir().join(format!("sac-dist-{}", uuid::Uuid::now_v7()));
        std::fs::create_dir_all(dir.join("assets")).unwrap();
        std::fs::write(dir.join("index.html"), "<!doctype html><div id=root></div>").unwrap();
        std::fs::write(dir.join("assets/index-abc123.js"), "console.log(1)").unwrap();
        Self(dir)
    }
}

impl Drop for BuiltDashboard {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

struct Reply {
    status: StatusCode,
    headers: axum::http::HeaderMap,
    body: String,
}

async fn send(app: &Router, request: Request<Body>) -> Reply {
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    Reply {
        status,
        headers,
        body: String::from_utf8_lossy(&body).into_owned(),
    }
}

async fn get(app: &Router, uri: &str) -> Reply {
    send(app, Request::get(uri).body(Body::empty()).unwrap()).await
}

#[tokio::test]
async fn serves_the_dashboard_next_to_the_api() {
    let db = TestDb::new().await;
    let dist = BuiltDashboard::new();
    let app = router_with(
        app_state(db.pool()),
        RouterOptions {
            dashboard_dir: Some(dist.0.clone()),
            ..RouterOptions::default()
        },
    );

    let index = get(&app, "/").await;
    assert_eq!(index.status, StatusCode::OK);
    assert!(index.body.contains("id=root"));
    assert_eq!(index.headers[header::CACHE_CONTROL], "no-cache");
    let csp = index.headers[header::CONTENT_SECURITY_POLICY]
        .to_str()
        .unwrap();
    assert!(csp.starts_with("default-src 'self'"), "{csp}");
    assert!(csp.contains("frame-ancestors 'none'"));
    assert!(!csp.contains("unsafe-inline"));
    assert_eq!(index.headers[header::X_CONTENT_TYPE_OPTIONS], "nosniff");
    assert!(index.headers.contains_key("x-request-id"));

    // Client-side routes load the app, which then routes in the browser.
    let deep_link = get(&app, "/doors/0190a0a0-dead-7000-8000-00000000beef").await;
    assert_eq!(deep_link.status, StatusCode::OK);
    assert!(deep_link.body.contains("id=root"));

    let asset = get(&app, "/assets/index-abc123.js").await;
    assert_eq!(asset.status, StatusCode::OK);
    assert_eq!(asset.body, "console.log(1)");
    assert!(
        asset.headers[header::CACHE_CONTROL]
            .to_str()
            .unwrap()
            .contains("immutable")
    );

    // A missing asset is a real 404 and must not be cached.
    let missing = get(&app, "/assets/index-old999.js").await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
    assert_eq!(missing.headers[header::CACHE_CONTROL], "no-store");

    // Files outside the directory stay unreachable.
    let traversal = get(&app, "/assets/../../../../etc/passwd").await;
    assert!(!traversal.body.contains("root:"));

    // The API is unchanged: JSON, strict policy, no caching.
    let health = get(&app, "/api/v1/health").await;
    assert_eq!(health.status, StatusCode::OK);
    assert_eq!(health.headers[header::CACHE_CONTROL], "no-store");
    assert!(
        health.headers[header::CONTENT_SECURITY_POLICY]
            .to_str()
            .unwrap()
            .starts_with("default-src 'none'")
    );
    // Unknown API paths never fall through to the dashboard.
    let unknown = get(&app, "/api/v1/no-such-thing").await;
    assert_eq!(unknown.status, StatusCode::NOT_FOUND);
    assert!(unknown.body.contains("not_found"), "{}", unknown.body);
    db.cleanup().await;
}

#[tokio::test]
async fn unknown_api_paths_answer_json_without_a_dashboard() {
    let db = TestDb::new().await;
    let app = router_with(app_state(db.pool()), RouterOptions::default());

    let unknown = get(&app, "/api/v1/no-such-thing").await;
    assert_eq!(unknown.status, StatusCode::NOT_FOUND);
    assert_eq!(unknown.headers[header::CONTENT_TYPE], "application/json");
    assert_eq!(get(&app, "/").await.status, StatusCode::NOT_FOUND);
    db.cleanup().await;
}

const PROXY: &str = "172.30.83.10";

/// The app as deployed: every connection comes from `peer`, and only
/// `PROXY` is trusted to report client addresses.
fn behind(db: &TestDb, peer: &str) -> Router {
    let state = AppState::new(
        db.pool(),
        &auth_config(),
        AppOptions {
            trusted_proxies: TrustedProxies::new(vec![IpNetwork::parse(PROXY).unwrap()]),
            ..AppOptions::default()
        },
        Arc::new(SystemClock),
    )
    .unwrap();
    let peer: SocketAddr = format!("{peer}:40000").parse().unwrap();
    router_with(state, RouterOptions::default()).layer(MockConnectInfo(peer))
}

async fn failed_login(app: &Router, username: &str, forwarded_for: &str) -> StatusCode {
    let request = Request::post("/api/v1/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-forwarded-for", forwarded_for)
        .body(Body::from(
            json!({ "username": username, "password": "wrong password" }).to_string(),
        ))
        .unwrap();
    send(app, request).await.status
}

/// The per-address limit is 20 failures; distinct usernames keep the
/// per-name limit (5) out of the way.
#[tokio::test]
async fn behind_a_trusted_proxy_each_client_has_its_own_limit() {
    let db = TestDb::new().await;
    let app = behind(&db, PROXY);

    for i in 0..20 {
        let status = failed_login(&app, &format!("user{i}"), "198.51.100.1").await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }
    assert_eq!(
        failed_login(&app, "another", "198.51.100.1").await,
        StatusCode::TOO_MANY_REQUESTS
    );
    // Other clients behind the same proxy are not affected...
    assert_eq!(
        failed_login(&app, "another", "198.51.100.2").await,
        StatusCode::UNAUTHORIZED
    );
    // ...and the blocked client cannot escape by prepending an address.
    assert_eq!(
        failed_login(&app, "yet-another", "203.0.113.50, 198.51.100.1").await,
        StatusCode::TOO_MANY_REQUESTS
    );
    db.cleanup().await;
}

#[tokio::test]
async fn untrusted_callers_cannot_escape_the_limit_with_a_forged_header() {
    let db = TestDb::new().await;
    let app = behind(&db, "203.0.113.9");

    for i in 0..20 {
        // A new "client address" every time: ignored, the peer counts.
        let status = failed_login(&app, &format!("user{i}"), &format!("192.0.2.{i}")).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }
    assert_eq!(
        failed_login(&app, "another", "192.0.2.99").await,
        StatusCode::TOO_MANY_REQUESTS
    );
    db.cleanup().await;
}
