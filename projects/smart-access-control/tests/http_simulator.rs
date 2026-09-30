//! The in-process controller simulator, driven over its admin API.

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
        http::{AppOptions, AppState, router, router_with_simulator},
    },
    simulator::Simulator,
};
use tower::ServiceExt;

const PASSWORD: &str = "correct horse battery staple";

struct Api {
    app: Router,
    state: AppState,
    simulator: Arc<Simulator>,
    admin: String,
    viewer: String,
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
        let mut tokens = Vec::new();
        for (username, role) in [("ops", Role::Admin), ("watcher", Role::Viewer)] {
            state
                .auth
                .create_administrator(CreateAdministrator {
                    username: username.into(),
                    password: PASSWORD.into(),
                    role,
                })
                .await
                .unwrap();
            tokens.push(
                state
                    .auth
                    .login(username, PASSWORD)
                    .await
                    .unwrap()
                    .access_token,
            );
        }
        let simulator = Arc::new(Simulator::default());
        Self {
            app: router_with_simulator(state.clone(), Arc::clone(&simulator)),
            state,
            simulator,
            admin: tokens[0].clone(),
            viewer: tokens[1].clone(),
        }
    }

    async fn call(&self, token: &str, method: &str, uri: &str, body: Value) -> (StatusCode, Value) {
        let request = Request::builder()
            .method(method)
            .uri(format!("/api/v1{uri}"))
            .header(header::AUTHORIZATION, format!("Bearer {token}"))
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();
        let response = self.app.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (
            status,
            serde_json::from_slice(&bytes).unwrap_or(Value::Null),
        )
    }

    async fn admin(&self, method: &str, uri: &str, body: Value) -> (StatusCode, Value) {
        self.call(&self.admin, method, uri, body).await
    }

    async fn door_status(&self, door: &str) -> String {
        self.admin("GET", &format!("/doors/{door}"), Value::Null)
            .await
            .1["status"]
            .as_str()
            .unwrap()
            .to_owned()
    }

    /// A door on `controller` and a user whose CARD-1 may always open it.
    async fn setup_access(&self, controller: &str) -> String {
        let door = self
            .admin(
                "POST",
                "/doors",
                json!({ "name": "D", "location": "L", "controller_id": controller }),
            )
            .await
            .1;
        let user = self
            .admin(
                "POST",
                "/users",
                json!({ "name": "Alice", "email": "alice@example.com" }),
            )
            .await
            .1;
        self.admin(
            "POST",
            "/cards",
            json!({ "user_id": user["id"], "card_number": "CARD-1" }),
        )
        .await;
        let group = self
            .admin("POST", "/access-groups", json!({ "name": "Staff" }))
            .await
            .1;
        self.admin(
            "POST",
            &format!("/access-groups/{}/members", group["id"].as_str().unwrap()),
            json!({ "user_id": user["id"] }),
        )
        .await;
        self.admin(
            "POST",
            "/permissions",
            json!({ "group_id": group["id"], "door_id": door["id"] }),
        )
        .await;
        door["id"].as_str().unwrap().to_owned()
    }

    async fn wait_for_door(&self, door: &str, wanted: &str) -> bool {
        for _ in 0..50 {
            if self.door_status(door).await == wanted {
                return true;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        false
    }
}

#[tokio::test]
async fn simulated_controller_swipes_cards_end_to_end() {
    let db = TestDb::new().await;
    let api = Api::new(&db, chrono::Duration::seconds(30)).await;
    let door = api.setup_access("sim-001").await;

    let (status, started) = api
        .admin(
            "POST",
            "/simulator/controllers",
            json!({ "controller_id": "sim-001" }),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "{started}");
    assert_eq!(started["simulated"], true);
    assert_eq!(started["network"], "up");

    // Its first heartbeat brings the door online.
    assert!(api.wait_for_door(&door, "online").await);
    let registered = api
        .admin("GET", "/controllers/sim-001", Value::Null)
        .await
        .1;
    assert_eq!(registered["status"], "online");

    let (status, granted) = api
        .admin(
            "POST",
            "/simulator/controllers/sim-001/swipe",
            json!({ "card_number": "CARD-1", "door_id": door }),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{granted}");
    assert_eq!(granted["decision"], "granted");

    let (_, denied) = api
        .admin(
            "POST",
            "/simulator/controllers/sim-001/swipe",
            json!({ "card_number": "CARD-404", "door_id": door }),
        )
        .await;
    assert_eq!(denied["reason"], "unknown_card");

    // Both decisions are in the central event log.
    let events = api.admin("GET", "/events", Value::Null).await.1;
    assert_eq!(events["items"].as_array().unwrap().len(), 2);
    db.cleanup().await;
}

#[tokio::test]
async fn outage_is_detected_and_reconnect_recovers() {
    let db = TestDb::new().await;
    let api = Api::new(&db, chrono::Duration::seconds(1)).await;
    let monitor = spawn_liveness_monitor(api.state.clone());
    let door = api.setup_access("sim-001").await;
    api.admin(
        "POST",
        "/simulator/controllers",
        json!({ "controller_id": "sim-001" }),
    )
    .await;
    assert!(api.wait_for_door(&door, "online").await);

    // The simulated network drops: no clean disconnect, just silence.
    let (_, down) = api
        .admin("POST", "/simulator/controllers/sim-001/outage", Value::Null)
        .await;
    assert_eq!(down["network"], "down");
    assert!(
        api.wait_for_door(&door, "offline").await,
        "the liveness monitor should notice the missing heartbeats"
    );

    // A swipe during the outage is denied locally and never recorded.
    let (status, error) = api
        .admin(
            "POST",
            "/simulator/controllers/sim-001/swipe",
            json!({ "card_number": "CARD-1", "door_id": door }),
        )
        .await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(error["error"]["code"], "controller_offline");
    assert_eq!(
        api.admin("GET", "/events", Value::Null).await.1["items"],
        json!([])
    );

    // Reconnect: heartbeat at once, door back online, swipes work again.
    let (_, up) = api
        .admin(
            "POST",
            "/simulator/controllers/sim-001/reconnect",
            Value::Null,
        )
        .await;
    assert_eq!(up["network"], "up");
    assert_eq!(api.door_status(&door).await, "online");
    let (_, granted) = api
        .admin(
            "POST",
            "/simulator/controllers/sim-001/swipe",
            json!({ "card_number": "CARD-1", "door_id": door }),
        )
        .await;
    assert_eq!(granted["decision"], "granted");

    api.state.begin_shutdown();
    monitor.await.unwrap();
    api.simulator.stop_all().await;
    db.cleanup().await;
}

#[tokio::test]
async fn clean_disconnect_and_stop_take_doors_offline_at_once() {
    let db = TestDb::new().await;
    let api = Api::new(&db, chrono::Duration::seconds(30)).await;
    let door = api.setup_access("sim-001").await;
    api.admin(
        "POST",
        "/simulator/controllers",
        json!({ "controller_id": "sim-001" }),
    )
    .await;
    assert!(api.wait_for_door(&door, "online").await);

    api.admin(
        "POST",
        "/simulator/controllers/sim-001/disconnect",
        Value::Null,
    )
    .await;
    assert_eq!(
        api.door_status(&door).await,
        "offline",
        "no need to wait for a timeout"
    );

    api.admin(
        "POST",
        "/simulator/controllers/sim-001/reconnect",
        Value::Null,
    )
    .await;
    assert_eq!(api.door_status(&door).await, "online");

    let (status, _) = api
        .admin("DELETE", "/simulator/controllers/sim-001", Value::Null)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert_eq!(api.door_status(&door).await, "offline");
    assert_eq!(
        api.admin("GET", "/simulator/controllers", Value::Null)
            .await
            .1,
        json!([])
    );
    db.cleanup().await;
}

#[tokio::test]
async fn restarting_a_simulation_rotates_the_key_and_is_audited() {
    let db = TestDb::new().await;
    let api = Api::new(&db, chrono::Duration::seconds(30)).await;
    let start = json!({ "controller_id": "sim-001" });

    api.admin("POST", "/simulator/controllers", start.clone())
        .await;
    let (status, _) = api
        .admin("POST", "/simulator/controllers", start.clone())
        .await;
    assert_eq!(status, StatusCode::CONFLICT, "already running");

    api.admin("DELETE", "/simulator/controllers/sim-001", Value::Null)
        .await;
    let (status, restarted) = api.admin("POST", "/simulator/controllers", start).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(restarted["network"], "up");

    let log = api.admin("GET", "/audit-log?limit=2", Value::Null).await.1;
    let actions: Vec<&str> = log["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["action"].as_str().unwrap())
        .collect();
    assert_eq!(actions, ["controller_key_rotated", "controller_registered"]);
    api.simulator.stop_all().await;
    db.cleanup().await;
}

#[tokio::test]
async fn viewers_may_look_but_not_touch() {
    let db = TestDb::new().await;
    let api = Api::new(&db, chrono::Duration::seconds(30)).await;
    api.admin(
        "POST",
        "/simulator/controllers",
        json!({ "controller_id": "sim-001" }),
    )
    .await;

    let (status, list) = api
        .call(&api.viewer, "GET", "/simulator/controllers", Value::Null)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list[0]["controller_id"], "sim-001");

    for (method, uri) in [
        ("POST", "/simulator/controllers"),
        ("POST", "/simulator/controllers/sim-001/swipe"),
        ("POST", "/simulator/controllers/sim-001/outage"),
        ("POST", "/simulator/controllers/sim-001/disconnect"),
        ("POST", "/simulator/controllers/sim-001/reconnect"),
        ("DELETE", "/simulator/controllers/sim-001"),
    ] {
        let (status, _) = api.call(&api.viewer, method, uri, json!({})).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{method} {uri}");
    }
    let (status, _) = api
        .call("not-a-token", "GET", "/simulator/controllers", Value::Null)
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    api.simulator.stop_all().await;
    db.cleanup().await;
}

#[tokio::test]
async fn simulator_routes_do_not_exist_unless_enabled() {
    let db = TestDb::new().await;
    let api = Api::new(&db, chrono::Duration::seconds(30)).await;
    let disabled = router(api.state.clone());
    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/simulator/controllers")
        .header(header::AUTHORIZATION, format!("Bearer {}", api.admin))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "controller_id": "sim-001" }).to_string(),
        ))
        .unwrap();

    let response = disabled.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert!(!api.simulator.is_running("sim-001"));
    db.cleanup().await;
}
