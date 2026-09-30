//! The management API over HTTP: access rules, full workflows, error mapping.

mod common;

use axum::{
    Router,
    body::Body,
    http::{HeaderMap, Method, Request, StatusCode, header},
};
use common::{TestDb, app_state};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use smart_access_control::{
    application::CreateAdministrator,
    domain::Role,
    interfaces::http::{cors_layer, router},
};
use tower::ServiceExt;

const PASSWORD: &str = "correct horse battery staple";

/// A logged-in API client.
struct Client {
    app: Router,
    token: Option<String>,
}

struct Reply {
    status: StatusCode,
    headers: HeaderMap,
    json: Value,
}

impl Client {
    async fn send(&self, method: Method, uri: &str, body: Option<Value>) -> Reply {
        let mut request = Request::builder().method(method).uri(uri);
        if let Some(token) = &self.token {
            request = request.header(header::AUTHORIZATION, format!("Bearer {token}"));
        }
        let body = match body {
            Some(json) => {
                request = request.header(header::CONTENT_TYPE, "application/json");
                Body::from(json.to_string())
            }
            None => Body::empty(),
        };
        let response = self
            .app
            .clone()
            .oneshot(request.body(body).unwrap())
            .await
            .unwrap();
        let status = response.status();
        let headers = response.headers().clone();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        Reply {
            status,
            headers,
            json: serde_json::from_slice(&bytes).unwrap_or(Value::Null),
        }
    }

    async fn get(&self, uri: &str) -> Reply {
        self.send(Method::GET, uri, None).await
    }

    async fn post(&self, uri: &str, body: Value) -> Reply {
        self.send(Method::POST, uri, Some(body)).await
    }

    async fn patch(&self, uri: &str, body: Value) -> Reply {
        self.send(Method::PATCH, uri, Some(body)).await
    }

    async fn delete(&self, uri: &str) -> Reply {
        self.send(Method::DELETE, uri, None).await
    }

    /// POST that must succeed with 201; returns the created resource.
    async fn create(&self, uri: &str, body: Value) -> Value {
        let reply = self.post(uri, body).await;
        assert_eq!(
            reply.status,
            StatusCode::CREATED,
            "POST {uri}: {}",
            reply.json
        );
        reply.json
    }
}

struct Clients {
    admin: Client,
    viewer: Client,
    anonymous: Client,
    admin_id: String,
}

async fn clients(db: &TestDb) -> Clients {
    let state = app_state(db.pool());
    let app = router(state.clone());
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
        let login = Client {
            app: app.clone(),
            token: None,
        }
        .post(
            "/api/v1/auth/login",
            json!({ "username": username, "password": PASSWORD }),
        )
        .await;
        tokens.push(login.json["access_token"].as_str().unwrap().to_owned());
    }
    let admin = Client {
        app: app.clone(),
        token: Some(tokens[0].clone()),
    };
    let admin_id = admin.get("/api/v1/auth/me").await.json["id"]
        .as_str()
        .unwrap()
        .to_owned();
    Clients {
        admin,
        viewer: Client {
            app: app.clone(),
            token: Some(tokens[1].clone()),
        },
        anonymous: Client { app, token: None },
        admin_id,
    }
}

fn id(resource: &Value) -> &str {
    resource["id"].as_str().unwrap()
}

/// Every route of the management API, so a newly added route without the
/// right extractor is noticed here. `{id}` is a random, nonexistent UUID:
/// authentication and authorization run before anything else.
const X: &str = "0190a0a0-0000-7000-8000-000000000000";

fn read_routes() -> Vec<String> {
    [
        "/users",
        "/users/{id}",
        "/cards",
        "/cards/{id}",
        "/doors",
        "/doors/{id}",
        "/access-groups",
        "/access-groups/{id}",
        "/access-groups/{id}/members",
        "/schedules",
        "/schedules/{id}",
        "/permissions",
        "/events",
        "/events/recent",
        "/events/{id}",
        "/audit-log",
        "/controllers",
        "/controllers/ctrl-x",
    ]
    .iter()
    .map(|path| format!("/api/v1{}", path.replace("{id}", X)))
    .collect()
}

fn write_routes() -> Vec<(Method, String)> {
    [
        (Method::POST, "/users"),
        (Method::PATCH, "/users/{id}"),
        (Method::DELETE, "/users/{id}"),
        (Method::POST, "/cards"),
        (Method::PATCH, "/cards/{id}"),
        (Method::POST, "/cards/{id}/revoke"),
        (Method::POST, "/doors"),
        (Method::PATCH, "/doors/{id}"),
        (Method::PATCH, "/doors/{id}/status"),
        (Method::POST, "/access-groups"),
        (Method::PATCH, "/access-groups/{id}"),
        (Method::DELETE, "/access-groups/{id}"),
        (Method::POST, "/access-groups/{id}/members"),
        (Method::DELETE, "/access-groups/{id}/members/{id}"),
        (Method::POST, "/schedules"),
        (Method::POST, "/permissions"),
        (Method::DELETE, "/permissions/{id}"),
        (Method::POST, "/access/decisions"),
        (Method::POST, "/controllers"),
        (Method::POST, "/controllers/ctrl-x/rotate-key"),
    ]
    .into_iter()
    .map(|(method, path)| (method, format!("/api/v1{}", path.replace("{id}", X))))
    .collect()
}

#[tokio::test]
async fn every_route_requires_a_token_and_writes_require_admin() {
    let db = TestDb::new().await;
    let c = clients(&db).await;

    for uri in read_routes() {
        let anonymous = c.anonymous.get(&uri).await;
        assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED, "GET {uri}");
        // Viewers may read: anything but 401/403 (a random id gives 404).
        let viewer = c.viewer.get(&uri).await.status;
        assert!(
            viewer == StatusCode::OK || viewer == StatusCode::NOT_FOUND,
            "GET {uri}: {viewer}"
        );
    }

    for (method, uri) in write_routes() {
        let body = Some(json!({}));
        let anonymous = c.anonymous.send(method.clone(), &uri, body.clone()).await;
        assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED, "{method} {uri}");
        let viewer = c.viewer.send(method.clone(), &uri, body).await;
        assert_eq!(viewer.status, StatusCode::FORBIDDEN, "{method} {uri}");
        assert_eq!(viewer.json["error"]["code"], "forbidden");
    }

    // A forbidden write with a valid body changes nothing.
    let attempt = c
        .viewer
        .post(
            "/api/v1/users",
            json!({ "name": "V", "email": "v@example.com" }),
        )
        .await;
    assert_eq!(attempt.status, StatusCode::FORBIDDEN);
    assert_eq!(c.admin.get("/api/v1/users").await.json["items"], json!([]));
    db.cleanup().await;
}

#[tokio::test]
async fn full_access_scenario_over_http() {
    let db = TestDb::new().await;
    let c = clients(&db).await;
    let api = &c.admin;

    let alice = api
        .create(
            "/api/v1/users",
            json!({ "name": "Alice", "email": "alice@example.com" }),
        )
        .await;
    let card = api
        .create(
            "/api/v1/cards",
            json!({ "user_id": id(&alice), "card_number": "card-10001", "expires_at": "2099-01-01T00:00:00Z" }),
        )
        .await;
    assert_eq!(card["card_number"], "CARD-10001");
    assert_eq!(card["effective_status"], "active");

    let door = api
        .create(
            "/api/v1/doors",
            json!({ "name": "Main Entrance", "location": "Building A", "controller_id": "ctrl-001" }),
        )
        .await;
    assert_eq!(door["status"], "offline");
    let decide = || {
        api.post(
            "/api/v1/access/decisions",
            json!({ "card_number": "CARD-10001", "door_id": id(&door) }),
        )
    };
    assert_eq!(decide().await.json["reason"], "door_offline");

    let online = api
        .patch(
            &format!("/api/v1/doors/{}/status", id(&door)),
            json!({ "status": "online" }),
        )
        .await;
    assert_eq!(online.json["status"], "online");
    assert_eq!(decide().await.json["reason"], "permission_denied");

    let staff = api
        .create(
            "/api/v1/access-groups",
            json!({ "name": "Staff", "description": "Everyone" }),
        )
        .await;
    let members_uri = format!("/api/v1/access-groups/{}/members", id(&staff));
    assert_eq!(api.get(&members_uri).await.json, json!({ "user_ids": [] }));
    let joined = api
        .post(
            &format!("/api/v1/access-groups/{}/members", id(&staff)),
            json!({ "user_id": id(&alice) }),
        )
        .await;
    assert_eq!(joined.status, StatusCode::NO_CONTENT);
    assert_eq!(
        api.get(&members_uri).await.json,
        json!({ "user_ids": [id(&alice)] })
    );

    let always = api
        .create(
            "/api/v1/schedules",
            json!({
                "name": "Always",
                "timezone": "Europe/London",
                "rules": [{ "days": ["mon","tue","wed","thu","fri","sat","sun"], "start": "00:00", "end": "00:00" }]
            }),
        )
        .await;
    assert_eq!(always["rules"][0]["days"].as_array().unwrap().len(), 7);
    let permission = api
        .create(
            "/api/v1/permissions",
            json!({ "group_id": id(&staff), "door_id": id(&door), "schedule_id": id(&always) }),
        )
        .await;

    let granted = decide().await;
    assert_eq!(granted.status, StatusCode::OK);
    assert_eq!(granted.json["decision"], "granted");
    assert_eq!(granted.json["reason"], Value::Null);
    assert_eq!(granted.json["user_id"], alice["id"]);

    // The event is readable individually and in the recent feed (newest first).
    let event = api
        .get(&format!("/api/v1/events/{}", id(&granted.json)))
        .await;
    assert_eq!(event.json, granted.json);
    let recent = api.get("/api/v1/events/recent?limit=5").await.json;
    assert_eq!(recent[0], granted.json);
    assert_eq!(recent.as_array().unwrap().len(), 3);

    // Suspending the user denies access immediately.
    let suspended = api
        .patch(
            &format!("/api/v1/users/{}", id(&alice)),
            json!({ "status": "suspended" }),
        )
        .await;
    assert_eq!(suspended.json["status"], "suspended");
    assert_eq!(decide().await.json["reason"], "user_suspended");

    // Revoking the permission removes access too.
    api.patch(
        &format!("/api/v1/users/{}", id(&alice)),
        json!({ "status": "active" }),
    )
    .await;
    assert_eq!(decide().await.json["decision"], "granted");
    let revoked = api
        .delete(&format!("/api/v1/permissions/{}", id(&permission)))
        .await;
    assert_eq!(revoked.status, StatusCode::NO_CONTENT);
    assert_eq!(decide().await.json["reason"], "permission_denied");
    db.cleanup().await;
}

#[tokio::test]
async fn errors_map_to_json_statuses() {
    let db = TestDb::new().await;
    let c = clients(&db).await;
    let api = &c.admin;
    api.create(
        "/api/v1/users",
        json!({ "name": "Alice", "email": "alice@example.com" }),
    )
    .await;

    let cases = [
        (api.get("/api/v1/users/00000000-0000-0000-0000-000000000000").await, 404, "not_found"),
        (api.get("/api/v1/users/not-a-uuid").await, 400, "invalid_path"),
        (api.get("/api/v1/users?limit=lots").await, 400, "invalid_query"),
        (api.get("/api/v1/users?page=2").await, 400, "invalid_query"),
        (
            api.post("/api/v1/users", json!({ "name": "A", "email": "ALICE@example.com" })).await,
            409,
            "conflict",
        ),
        (
            api.post("/api/v1/users", json!({ "name": "A", "email": "nope" })).await,
            422,
            "validation_failed",
        ),
        (
            api.post("/api/v1/users", json!({ "name": "A", "email": "a@b.co", "admin": true })).await,
            422,
            "invalid_request",
        ),
        (
            api.post("/api/v1/cards", json!({ "user_id": "00000000-0000-0000-0000-000000000000", "card_number": "CARD-1" })).await,
            404,
            "not_found",
        ),
    ];
    for (reply, status, code) in cases {
        assert_eq!(reply.status.as_u16(), status, "{}", reply.json);
        assert_eq!(reply.json["error"]["code"], code, "{}", reply.json);
    }
    db.cleanup().await;
}

#[tokio::test]
async fn state_changes_follow_the_domain_rules() {
    let db = TestDb::new().await;
    let c = clients(&db).await;
    let api = &c.admin;
    let alice = api
        .create(
            "/api/v1/users",
            json!({ "name": "Alice", "email": "alice@example.com" }),
        )
        .await;
    let card = api
        .create(
            "/api/v1/cards",
            json!({ "user_id": id(&alice), "card_number": "CARD-1" }),
        )
        .await;
    let card_uri = format!("/api/v1/cards/{}", id(&card));

    // Revoking goes through its own endpoint, and is final.
    let via_patch = api.patch(&card_uri, json!({ "status": "revoked" })).await;
    assert_eq!(via_patch.status, StatusCode::UNPROCESSABLE_ENTITY);
    let revoked = api.post(&format!("{card_uri}/revoke"), json!({})).await;
    assert_eq!(revoked.json["status"], "revoked");
    let again = api.patch(&card_uri, json!({ "status": "active" })).await;
    assert_eq!(again.status, StatusCode::CONFLICT);
    assert_eq!(again.json["error"]["code"], "invalid_state");

    // Setting the current status again is a harmless no-op.
    let same = api
        .patch(
            &format!("/api/v1/users/{}", id(&alice)),
            json!({ "status": "active" }),
        )
        .await;
    assert_eq!(same.status, StatusCode::OK);

    // DELETE archives; the user can still be read.
    let deleted = api.delete(&format!("/api/v1/users/{}", id(&alice))).await;
    assert_eq!(deleted.status, StatusCode::NO_CONTENT);
    let archived = api.get(&format!("/api/v1/users/{}", id(&alice))).await;
    assert_eq!(archived.json["status"], "archived");

    // A disabled door comes back as offline when set to "offline".
    let door = api
        .create(
            "/api/v1/doors",
            json!({ "name": "D", "location": "L", "controller_id": "c1" }),
        )
        .await;
    let status_uri = format!("/api/v1/doors/{}/status", id(&door));
    assert_eq!(
        api.patch(&status_uri, json!({ "status": "disabled" }))
            .await
            .json["status"],
        "disabled"
    );
    assert_eq!(
        api.patch(&status_uri, json!({ "status": "online" }))
            .await
            .status,
        StatusCode::CONFLICT
    );
    assert_eq!(
        api.patch(&status_uri, json!({ "status": "offline" }))
            .await
            .json["status"],
        "offline"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn patch_distinguishes_missing_from_null() {
    let db = TestDb::new().await;
    let c = clients(&db).await;
    let api = &c.admin;
    let group = api
        .create(
            "/api/v1/access-groups",
            json!({ "name": "Staff", "description": "Everyone" }),
        )
        .await;
    let uri = format!("/api/v1/access-groups/{}", id(&group));

    let renamed = api.patch(&uri, json!({ "name": "All Staff" })).await.json;
    assert_eq!(
        renamed["description"], "Everyone",
        "missing field: unchanged"
    );

    let cleared = api.patch(&uri, json!({ "description": null })).await.json;
    assert_eq!(
        cleared["description"],
        Value::Null,
        "explicit null: cleared"
    );
    assert_eq!(cleared["name"], "All Staff");
    db.cleanup().await;
}

#[tokio::test]
async fn pagination_is_clamped_and_reported() {
    let db = TestDb::new().await;
    let c = clients(&db).await;
    for n in 0..3 {
        c.admin
            .create(
                "/api/v1/users",
                json!({ "name": format!("U{n}"), "email": format!("u{n}@example.com") }),
            )
            .await;
    }

    let page = c.admin.get("/api/v1/users?limit=2&offset=1").await.json;
    assert_eq!(
        (page["limit"].as_u64(), page["offset"].as_u64()),
        (Some(2), Some(1))
    );
    let names: Vec<&str> = page["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|u| u["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["U1", "U2"]);

    let huge = c.admin.get("/api/v1/users?limit=100000").await.json;
    assert_eq!(huge["limit"], 100);
    db.cleanup().await;
}

#[tokio::test]
async fn administrative_changes_are_audited_with_the_actor() {
    let db = TestDb::new().await;
    let c = clients(&db).await;
    let alice = c
        .admin
        .create(
            "/api/v1/users",
            json!({ "name": "Alice", "email": "alice@example.com" }),
        )
        .await;
    c.admin
        .patch(
            &format!("/api/v1/users/{}", id(&alice)),
            json!({ "name": "Alice B", "status": "suspended" }),
        )
        .await;

    let log = c.viewer.get("/api/v1/audit-log?limit=3").await.json;
    let entries = log["items"].as_array().unwrap();
    let actions: Vec<&str> = entries
        .iter()
        .map(|e| e["action"].as_str().unwrap())
        .collect();
    assert_eq!(
        actions,
        ["user_suspended", "user_updated", "user_registered"]
    );
    for entry in entries {
        assert_eq!(entry["administrator_id"], c.admin_id.as_str());
        assert_eq!(entry["subject"], format!("user:{}", id(&alice)));
    }
    db.cleanup().await;
}

#[tokio::test]
async fn cors_allows_only_configured_origins() {
    let db = TestDb::new().await;
    let app = router(app_state(db.pool()))
        .layer(cors_layer(&["http://localhost:5173".to_owned()]).unwrap());
    let preflight = |origin: &'static str| {
        Request::builder()
            .method(Method::OPTIONS)
            .uri("/api/v1/users")
            .header(header::ORIGIN, origin)
            .header(header::ACCESS_CONTROL_REQUEST_METHOD, "POST")
            .header(
                header::ACCESS_CONTROL_REQUEST_HEADERS,
                "authorization,content-type",
            )
            .body(Body::empty())
            .unwrap()
    };

    let allowed = app
        .clone()
        .oneshot(preflight("http://localhost:5173"))
        .await
        .unwrap();
    assert_eq!(
        allowed.headers()[header::ACCESS_CONTROL_ALLOW_ORIGIN],
        "http://localhost:5173"
    );

    let other = app
        .oneshot(preflight("https://evil.example"))
        .await
        .unwrap();
    assert!(
        !other
            .headers()
            .contains_key(header::ACCESS_CONTROL_ALLOW_ORIGIN)
    );

    assert!(
        cors_layer(&[]).is_none(),
        "no origins configured: no CORS layer"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn admin_profile_never_exposes_secrets() {
    let db = TestDb::new().await;
    let c = clients(&db).await;
    let me = c.admin.get("/api/v1/auth/me").await;
    assert_eq!(
        me.headers.get(header::CONTENT_TYPE).unwrap(),
        "application/json"
    );
    let text = me.json.to_string();
    assert!(
        !text.contains("argon2") && !text.contains("password"),
        "{text}"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn events_can_be_filtered_over_http() {
    let db = TestDb::new().await;
    let c = clients(&db).await;
    let api = &c.admin;
    // A real card, so its attempt gets past the card checks to the door check
    // (the engine checks the card before the door).
    let alice = api
        .create(
            "/api/v1/users",
            json!({ "name": "Alice", "email": "alice@example.com" }),
        )
        .await;
    api.create(
        "/api/v1/cards",
        json!({ "user_id": id(&alice), "card_number": "CARD-1" }),
    )
    .await;
    let door = api
        .create(
            "/api/v1/doors",
            json!({ "name": "D", "location": "L", "controller_id": "c1" }),
        )
        .await;
    let decide = |card: &'static str| {
        api.post(
            "/api/v1/access/decisions",
            json!({ "card_number": card, "door_id": id(&door) }),
        )
    };
    decide("CARD-1").await; // door offline
    api.patch(
        &format!("/api/v1/doors/{}/status", id(&door)),
        json!({ "status": "online" }),
    )
    .await;
    decide("CARD-2").await; // unknown card (checked before the door)

    let offline = api
        .get(&format!(
            "/api/v1/events?door_id={}&reason=door_offline",
            id(&door)
        ))
        .await
        .json;
    let items = offline["items"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["card_number"], "CARD-1");

    let denied = api.get("/api/v1/events?decision=denied").await.json;
    assert_eq!(denied["items"].as_array().unwrap().len(), 2);
    let granted = api.get("/api/v1/events?decision=granted").await.json;
    assert_eq!(granted["items"], json!([]));

    for bad in [
        "/api/v1/events?decision=maybe",
        "/api/v1/events?reason=bad_luck",
        "/api/v1/events?decision=granted&reason=door_offline",
        "/api/v1/events?from=2026-10-02T00:00:00Z&until=2026-10-01T00:00:00Z",
    ] {
        let reply = api.get(bad).await;
        assert_eq!(
            reply.status,
            StatusCode::UNPROCESSABLE_ENTITY,
            "{bad}: {}",
            reply.json
        );
    }
    assert_eq!(
        api.get("/api/v1/events?from=yesterday").await.status,
        StatusCode::BAD_REQUEST
    );
    db.cleanup().await;
}
