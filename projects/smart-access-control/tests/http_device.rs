//! The controller-facing device API over HTTP, and controller liveness.

mod common;

use std::{sync::Arc, time::Duration};

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
};
use common::{TestDb, auth_config};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use smart_access_control::{
    application::CreateAdministrator,
    domain::Role,
    infrastructure::clock::SystemClock,
    interfaces::{
        background::spawn_liveness_monitor,
        http::{AppOptions, AppState, CONTROLLER_KEY_HEADER, router},
    },
};
use tower::ServiceExt;

const PASSWORD: &str = "correct horse battery staple";

struct Api {
    app: Router,
    state: AppState,
    admin_token: String,
}

struct Reply {
    status: StatusCode,
    json: Value,
}

async fn send(app: &Router, request: Request<Body>) -> Reply {
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    Reply {
        status,
        json: serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    }
}

fn json_request(method: &str, uri: &str, auth: (&str, &str), body: Value) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header(auth.0, auth.1)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

impl Api {
    async fn new(db: &TestDb, controller_timeout: chrono::Duration) -> Self {
        let state = AppState::new(
            db.pool(),
            &auth_config(),
            AppOptions {
                controller_timeout,
                ..AppOptions::default()
            },
            Arc::new(SystemClock),
        )
        .unwrap();
        state
            .auth
            .create_administrator(CreateAdministrator {
                username: "ops".into(),
                password: PASSWORD.into(),
                role: Role::Admin,
            })
            .await
            .unwrap();
        let admin_token = state
            .auth
            .login("ops", PASSWORD)
            .await
            .unwrap()
            .access_token;
        Self {
            app: router(state.clone()),
            state,
            admin_token,
        }
    }

    async fn admin(&self, method: &str, uri: &str, body: Value) -> Reply {
        let bearer = format!("Bearer {}", self.admin_token);
        send(
            &self.app,
            json_request(method, uri, ("authorization", &bearer), body),
        )
        .await
    }

    async fn device(&self, key: &str, path: &str, body: Value) -> Reply {
        let uri = format!("/api/v1/device/{path}");
        send(
            &self.app,
            json_request("POST", &uri, (CONTROLLER_KEY_HEADER, key), body),
        )
        .await
    }

    /// Registers a controller; returns its key.
    async fn controller(&self, id: &str) -> String {
        let reply = self
            .admin(
                "POST",
                "/api/v1/controllers",
                json!({ "controller_id": id }),
            )
            .await;
        assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.json);
        reply.json["key"].as_str().unwrap().to_owned()
    }

    /// Creates a door wired to `controller`; returns its id.
    async fn door(&self, controller: &str) -> String {
        let reply = self
            .admin(
                "POST",
                "/api/v1/doors",
                json!({ "name": "Door", "location": "Lab", "controller_id": controller }),
            )
            .await;
        reply.json["id"].as_str().unwrap().to_owned()
    }

    async fn door_status(&self, door: &str) -> String {
        let reply = self
            .admin("GET", &format!("/api/v1/doors/{door}"), Value::Null)
            .await;
        reply.json["status"].as_str().unwrap().to_owned()
    }

    /// A user holding CARD-1 with 24/7 access to `door`.
    async fn grant_card(&self, door: &str) {
        let user = self
            .admin(
                "POST",
                "/api/v1/users",
                json!({ "name": "Alice", "email": "alice@example.com" }),
            )
            .await
            .json;
        self.admin(
            "POST",
            "/api/v1/cards",
            json!({ "user_id": user["id"], "card_number": "CARD-1" }),
        )
        .await;
        let group = self
            .admin("POST", "/api/v1/access-groups", json!({ "name": "Staff" }))
            .await
            .json;
        self.admin(
            "POST",
            &format!(
                "/api/v1/access-groups/{}/members",
                group["id"].as_str().unwrap()
            ),
            json!({ "user_id": user["id"] }),
        )
        .await;
        self.admin(
            "POST",
            "/api/v1/permissions",
            json!({ "group_id": group["id"], "door_id": door }),
        )
        .await;
    }
}

#[tokio::test]
async fn controller_lifecycle_over_the_device_api() {
    let db = TestDb::new().await;
    let api = Api::new(&db, chrono::Duration::seconds(30)).await;
    let key = api.controller("ctrl-001").await;
    let door = api.door("ctrl-001").await;
    api.grant_card(&door).await;
    assert_eq!(api.door_status(&door).await, "offline");

    let heartbeat = api.device(&key, "heartbeat", json!({})).await;
    assert_eq!(heartbeat.status, StatusCode::OK);
    assert_eq!(
        heartbeat.json,
        json!({ "controller_id": "ctrl-001", "doors_online": 1 })
    );
    assert_eq!(api.door_status(&door).await, "online");

    let decision = api
        .device(
            &key,
            "access-requests",
            json!({ "card_number": "CARD-1", "door_id": door }),
        )
        .await;
    assert_eq!(decision.status, StatusCode::OK, "{}", decision.json);
    assert_eq!(decision.json["decision"], "granted");

    let bye = api.device(&key, "disconnect", json!({})).await;
    assert_eq!(bye.status, StatusCode::NO_CONTENT);
    assert_eq!(api.door_status(&door).await, "offline");
    let listed = api
        .admin("GET", "/api/v1/controllers/ctrl-001", Value::Null)
        .await;
    assert_eq!(listed.json["status"], "offline");

    // An access request is itself a sign of life: the door comes back online.
    let again = api
        .device(
            &key,
            "access-requests",
            json!({ "card_number": "CARD-1", "door_id": door }),
        )
        .await;
    assert_eq!(again.json["decision"], "granted");
    db.cleanup().await;
}

#[tokio::test]
async fn controllers_only_act_on_their_own_doors() {
    let db = TestDb::new().await;
    let api = Api::new(&db, chrono::Duration::seconds(30)).await;
    let key = api.controller("ctrl-001").await;
    api.controller("ctrl-002").await;
    let theirs = api.door("ctrl-002").await;

    for door in [theirs.as_str(), "0190a0a0-0000-7000-8000-000000000000"] {
        let reply = api
            .device(
                &key,
                "access-requests",
                json!({ "card_number": "CARD-1", "door_id": door }),
            )
            .await;
        assert_eq!(reply.status, StatusCode::FORBIDDEN, "{door}");
    }
    // Rejected requests are not access decisions: nothing was recorded.
    let events = api.admin("GET", "/api/v1/events", Value::Null).await;
    assert_eq!(events.json["items"], json!([]));
    db.cleanup().await;
}

#[tokio::test]
async fn device_and_admin_credentials_do_not_mix() {
    let db = TestDb::new().await;
    let api = Api::new(&db, chrono::Duration::seconds(30)).await;
    let key = api.controller("ctrl-001").await;

    // No key, a wrong key, and an admin token on a device route: all 401.
    let bearer = format!("Bearer {}", api.admin_token);
    let attempts = [
        json_request(
            "POST",
            "/api/v1/device/heartbeat",
            ("x-other", "x"),
            json!({}),
        ),
        json_request(
            "POST",
            "/api/v1/device/heartbeat",
            (CONTROLLER_KEY_HEADER, "wrong"),
            json!({}),
        ),
        json_request(
            "POST",
            "/api/v1/device/heartbeat",
            ("authorization", &bearer),
            json!({}),
        ),
    ];
    for request in attempts {
        let reply = send(&api.app, request).await;
        assert_eq!(reply.status, StatusCode::UNAUTHORIZED);
        assert_eq!(reply.json["error"]["message"], "invalid controller key");
    }

    // A controller key is not an admin credential.
    let reply = send(
        &api.app,
        json_request(
            "GET",
            "/api/v1/users",
            (CONTROLLER_KEY_HEADER, &key),
            Value::Null,
        ),
    )
    .await;
    assert_eq!(reply.status, StatusCode::UNAUTHORIZED);
    db.cleanup().await;
}

#[tokio::test]
async fn keys_are_shown_once_and_can_be_rotated() {
    let db = TestDb::new().await;
    let api = Api::new(&db, chrono::Duration::seconds(30)).await;
    let old = api.controller("ctrl-001").await;

    let listed = api.admin("GET", "/api/v1/controllers", Value::Null).await;
    assert!(
        !listed.json.to_string().contains(&old),
        "keys never appear in listings"
    );

    let rotated = api
        .admin(
            "POST",
            "/api/v1/controllers/ctrl-001/rotate-key",
            Value::Null,
        )
        .await;
    let new = rotated.json["key"].as_str().unwrap();
    assert_ne!(new, old);
    assert_eq!(
        api.device(&old, "heartbeat", json!({})).await.status,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        api.device(new, "heartbeat", json!({})).await.status,
        StatusCode::OK
    );

    let audit = api
        .admin("GET", "/api/v1/audit-log?limit=2", Value::Null)
        .await;
    let actions: Vec<&str> = audit.json["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["action"].as_str().unwrap())
        .collect();
    assert_eq!(actions, ["controller_key_rotated", "controller_registered"]);
    db.cleanup().await;
}

#[tokio::test]
async fn registration_rejects_bad_and_duplicate_ids() {
    let db = TestDb::new().await;
    let api = Api::new(&db, chrono::Duration::seconds(30)).await;
    api.controller("ctrl-001").await;

    let duplicate = api
        .admin(
            "POST",
            "/api/v1/controllers",
            json!({ "controller_id": "ctrl-001" }),
        )
        .await;
    assert_eq!(duplicate.status, StatusCode::CONFLICT);
    let invalid = api
        .admin(
            "POST",
            "/api/v1/controllers",
            json!({ "controller_id": "no spaces!" }),
        )
        .await;
    assert_eq!(invalid.status, StatusCode::UNPROCESSABLE_ENTITY);
    let missing = api
        .admin("GET", "/api/v1/controllers/ghost", Value::Null)
        .await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
    db.cleanup().await;
}

/// Disconnection and reconnection, driven by the real background monitor.
#[tokio::test]
async fn silent_controller_goes_offline_and_recovers() {
    let db = TestDb::new().await;
    let api = Api::new(&db, chrono::Duration::seconds(1)).await;
    let monitor = spawn_liveness_monitor(api.state.clone());
    let key = api.controller("ctrl-001").await;
    let door = api.door("ctrl-001").await;

    api.device(&key, "heartbeat", json!({})).await;
    assert_eq!(api.door_status(&door).await, "online");

    // The controller "loses its network": no more heartbeats.
    let mut went_offline = false;
    for _ in 0..40 {
        tokio::time::sleep(Duration::from_millis(100)).await;
        if api.door_status(&door).await == "offline" {
            went_offline = true;
            break;
        }
    }
    assert!(
        went_offline,
        "door should go offline after the heartbeat timeout"
    );

    // Reconnect.
    api.device(&key, "heartbeat", json!({})).await;
    assert_eq!(api.door_status(&door).await, "online");

    // The monitor stops with the server.
    api.state.begin_shutdown();
    tokio::time::timeout(Duration::from_secs(2), monitor)
        .await
        .expect("monitor should stop on shutdown")
        .unwrap();
    db.cleanup().await;
}
