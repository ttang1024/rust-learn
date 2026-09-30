//! The authentication API over HTTP, against a real database.

mod common;

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
    routing::get,
};
use common::{TestDb, app_state};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use smart_access_control::{
    application::CreateAdministrator,
    domain::Role,
    interfaces::http::{AppState, RequireAdmin, router},
};
use tower::ServiceExt;

const PASSWORD: &str = "correct horse battery staple";

struct Response {
    status: StatusCode,
    headers: axum::http::HeaderMap,
    json: Value,
}

async fn send(app: &Router, request: Request<Body>) -> Response {
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    Response {
        status,
        headers,
        json,
    }
}

fn post_json(uri: &str, body: &Value) -> Request<Body> {
    Request::post(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn get_with_token(uri: &str, token: &str) -> Request<Body> {
    Request::get(uri)
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap()
}

async fn setup(db: &TestDb) -> (Router, AppState) {
    let state = app_state(db.pool());
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
    }
    (router(state.clone()), state)
}

async fn login(app: &Router, username: &str) -> Value {
    let response = send(
        app,
        post_json(
            "/api/v1/auth/login",
            &json!({ "username": username, "password": PASSWORD }),
        ),
    )
    .await;
    assert_eq!(response.status, StatusCode::OK, "{}", response.json);
    response.json
}

#[tokio::test]
async fn login_returns_tokens_that_identify_the_admin() {
    let db = TestDb::new().await;
    let (app, _) = setup(&db).await;

    let response = send(
        &app,
        post_json(
            "/api/v1/auth/login",
            &json!({ "username": "OPS", "password": PASSWORD }),
        ),
    )
    .await;

    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(response.headers[header::CACHE_CONTROL], "no-store");
    assert_eq!(response.json["token_type"], "Bearer");
    assert_eq!(response.json["expires_in"], 15 * 60);
    assert_eq!(response.json["refresh_expires_in"], 7 * 24 * 60 * 60);

    let token = response.json["access_token"].as_str().unwrap();
    let me = send(&app, get_with_token("/api/v1/auth/me", token)).await;
    assert_eq!(me.status, StatusCode::OK);
    assert_eq!(me.json["role"], "admin");
    db.cleanup().await;
}

#[tokio::test]
async fn failed_logins_look_identical() {
    let db = TestDb::new().await;
    let (app, _) = setup(&db).await;

    let wrong_password = send(
        &app,
        post_json(
            "/api/v1/auth/login",
            &json!({ "username": "ops", "password": "nope" }),
        ),
    )
    .await;
    let unknown_user = send(
        &app,
        post_json(
            "/api/v1/auth/login",
            &json!({ "username": "ghost", "password": PASSWORD }),
        ),
    )
    .await;

    for response in [&wrong_password, &unknown_user] {
        assert_eq!(response.status, StatusCode::UNAUTHORIZED);
        assert_eq!(response.headers[header::WWW_AUTHENTICATE], "Bearer");
        assert_eq!(
            response.json,
            json!({ "error": { "code": "unauthorized", "message": "invalid credentials" } })
        );
    }
    db.cleanup().await;
}

#[tokio::test]
async fn protected_route_rejects_missing_or_bad_tokens() {
    let db = TestDb::new().await;
    let (app, _) = setup(&db).await;

    let no_header = Request::get("/api/v1/auth/me").body(Body::empty()).unwrap();
    for request in [
        no_header,
        get_with_token("/api/v1/auth/me", "forged.token.value"),
    ] {
        let response = send(&app, request).await;
        assert_eq!(response.status, StatusCode::UNAUTHORIZED);
        assert_eq!(response.json["error"]["code"], "unauthorized");
    }
    db.cleanup().await;
}

#[tokio::test]
async fn refresh_rotates_and_detects_reuse() {
    let db = TestDb::new().await;
    let (app, _) = setup(&db).await;
    let first = login(&app, "ops").await;
    let refresh =
        |token: &Value| post_json("/api/v1/auth/refresh", &json!({ "refresh_token": token }));

    let second = send(&app, refresh(&first["refresh_token"])).await;
    assert_eq!(second.status, StatusCode::OK);
    assert_ne!(second.json["refresh_token"], first["refresh_token"]);

    // Replaying the rotated token fails and kills the newer session too.
    let replay = send(&app, refresh(&first["refresh_token"])).await;
    assert_eq!(replay.status, StatusCode::UNAUTHORIZED);
    let newer = send(&app, refresh(&second.json["refresh_token"])).await;
    assert_eq!(newer.status, StatusCode::UNAUTHORIZED);
    db.cleanup().await;
}

#[tokio::test]
async fn logout_ends_the_session() {
    let db = TestDb::new().await;
    let (app, _) = setup(&db).await;
    let tokens = login(&app, "ops").await;
    let body = json!({ "refresh_token": tokens["refresh_token"] });

    let logout = send(&app, post_json("/api/v1/auth/logout", &body)).await;
    assert_eq!(logout.status, StatusCode::NO_CONTENT);

    let refresh = send(&app, post_json("/api/v1/auth/refresh", &body)).await;
    assert_eq!(refresh.status, StatusCode::UNAUTHORIZED);

    // Unknown tokens get the same 204: logout cannot be used as an oracle.
    let unknown = send(
        &app,
        post_json(
            "/api/v1/auth/logout",
            &json!({ "refresh_token": "made-up" }),
        ),
    )
    .await;
    assert_eq!(unknown.status, StatusCode::NO_CONTENT);
    db.cleanup().await;
}

#[tokio::test]
async fn malformed_requests_get_json_errors() {
    let db = TestDb::new().await;
    let (app, _) = setup(&db).await;

    let bad_json = Request::post("/api/v1/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from("{not json"))
        .unwrap();
    let response = send(&app, bad_json).await;
    assert_eq!(response.status, StatusCode::BAD_REQUEST);
    assert_eq!(response.json["error"]["code"], "invalid_request");

    let missing_field = send(
        &app,
        post_json("/api/v1/auth/login", &json!({ "username": "ops" })),
    )
    .await;
    assert_eq!(missing_field.status, StatusCode::UNPROCESSABLE_ENTITY);

    let wrong_type = Request::post("/api/v1/auth/login")
        .header(header::CONTENT_TYPE, "text/plain")
        .body(Body::from("username=ops"))
        .unwrap();
    let response = send(&app, wrong_type).await;
    assert_eq!(response.status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    assert_eq!(response.json["error"]["code"], "unsupported_media_type");
    db.cleanup().await;
}

#[tokio::test]
async fn oversized_bodies_are_rejected() {
    let db = TestDb::new().await;
    let (app, _) = setup(&db).await;
    let huge = json!({ "username": "ops", "password": "x".repeat(100 * 1024) });

    let response = send(&app, post_json("/api/v1/auth/login", &huge)).await;

    assert_eq!(response.status, StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(response.json["error"]["code"], "payload_too_large");
    db.cleanup().await;
}

#[tokio::test]
async fn require_admin_distinguishes_401_and_403() {
    let db = TestDb::new().await;
    let (app, state) = setup(&db).await;
    // A throwaway admin-only route, to exercise the extractor on its own.
    let guarded = Router::new()
        .route("/admin-only", get(|_: RequireAdmin| async { "ok" }))
        .with_state(state);

    let admin_token = login(&app, "ops").await["access_token"]
        .as_str()
        .unwrap()
        .to_owned();
    let viewer_token = login(&app, "watcher").await["access_token"]
        .as_str()
        .unwrap()
        .to_owned();

    let as_admin = send(&guarded, get_with_token("/admin-only", &admin_token)).await;
    assert_eq!(as_admin.status, StatusCode::OK);

    let as_viewer = send(&guarded, get_with_token("/admin-only", &viewer_token)).await;
    assert_eq!(as_viewer.status, StatusCode::FORBIDDEN);
    assert_eq!(as_viewer.json["error"]["code"], "forbidden");

    let anonymous = send(
        &guarded,
        Request::get("/admin-only").body(Body::empty()).unwrap(),
    )
    .await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
    db.cleanup().await;
}

#[tokio::test]
async fn repeated_failures_block_the_name_even_for_the_right_password() {
    let db = TestDb::new().await;
    let (app, _) = setup(&db).await;
    let attempt = |username: &'static str, password: &'static str| {
        post_json(
            "/api/v1/auth/login",
            &json!({ "username": username, "password": password }),
        )
    };

    for _ in 0..5 {
        let reply = send(&app, attempt("ops", "wrong password")).await;
        assert_eq!(reply.status, StatusCode::UNAUTHORIZED);
    }

    // Blocked now, even with the correct password.
    let blocked = send(&app, attempt("ops", PASSWORD)).await;
    assert_eq!(blocked.status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(blocked.json["error"]["code"], "too_many_requests");
    let retry: u64 = blocked.headers[header::RETRY_AFTER]
        .to_str()
        .unwrap()
        .parse()
        .unwrap();
    assert!(
        (850..=900).contains(&retry),
        "about 15 minutes, got {retry}"
    );

    // Other accounts are unaffected.
    assert_eq!(
        send(&app, attempt("watcher", PASSWORD)).await.status,
        StatusCode::OK
    );

    // A name that does not exist is throttled exactly the same way, so a
    // 429 does not reveal whether an account exists.
    for _ in 0..5 {
        send(&app, attempt("ghost", "whatever")).await;
    }
    assert_eq!(
        send(&app, attempt("ghost", "whatever")).await.status,
        StatusCode::TOO_MANY_REQUESTS
    );
    db.cleanup().await;
}

/// The `sac_refresh=<token>` pair from a `Set-Cookie` header.
fn cookie_pair(headers: &axum::http::HeaderMap) -> String {
    headers[header::SET_COOKIE]
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_owned()
}

fn with_cookie(uri: &str, cookie: &str, body: &Value) -> Request<Body> {
    Request::post(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::COOKIE, cookie)
        .body(Body::from(body.to_string()))
        .unwrap()
}

#[tokio::test]
async fn cookie_sessions_keep_the_refresh_token_away_from_scripts() {
    let db = TestDb::new().await;
    let (app, _) = setup(&db).await;

    let login = send(
        &app,
        post_json(
            "/api/v1/auth/login",
            &json!({ "username": "ops", "password": PASSWORD, "session": "cookie" }),
        ),
    )
    .await;
    assert_eq!(login.status, StatusCode::OK);
    assert!(
        login.json.get("refresh_token").is_none(),
        "not in the body: {}",
        login.json
    );
    assert!(login.json["access_token"].is_string());
    let set_cookie = login.headers[header::SET_COOKIE]
        .to_str()
        .unwrap()
        .to_owned();
    for attribute in [
        "HttpOnly",
        "Secure",
        "SameSite=Strict",
        "Path=/api/v1/auth",
        "Max-Age=604800",
    ] {
        assert!(
            set_cookie.contains(attribute),
            "missing {attribute}: {set_cookie}"
        );
    }
    let first = cookie_pair(&login.headers);

    // Refresh with the cookie and an empty JSON body: a new cookie comes back.
    let refreshed = send(
        &app,
        with_cookie("/api/v1/auth/refresh", &first, &json!({})),
    )
    .await;
    assert_eq!(refreshed.status, StatusCode::OK);
    assert!(refreshed.json.get("refresh_token").is_none());
    let second = cookie_pair(&refreshed.headers);
    assert_ne!(first, second, "rotated");

    // The old cookie is dead (and replaying it revokes the session).
    let replay = send(
        &app,
        with_cookie("/api/v1/auth/refresh", &first, &json!({})),
    )
    .await;
    assert_eq!(replay.status, StatusCode::UNAUTHORIZED);
    db.cleanup().await;
}

#[tokio::test]
async fn logout_clears_the_cookie() {
    let db = TestDb::new().await;
    let (app, _) = setup(&db).await;
    let login = send(
        &app,
        post_json(
            "/api/v1/auth/login",
            &json!({ "username": "ops", "password": PASSWORD, "session": "cookie" }),
        ),
    )
    .await;
    let cookie = cookie_pair(&login.headers);

    let logout = send(
        &app,
        with_cookie("/api/v1/auth/logout", &cookie, &json!({})),
    )
    .await;
    assert_eq!(logout.status, StatusCode::NO_CONTENT);
    let cleared = logout.headers[header::SET_COOKIE].to_str().unwrap();
    assert!(
        cleared.starts_with("sac_refresh=;") && cleared.contains("Max-Age=0"),
        "{cleared}"
    );

    let refresh = send(
        &app,
        with_cookie("/api/v1/auth/refresh", &cookie, &json!({})),
    )
    .await;
    assert_eq!(refresh.status, StatusCode::UNAUTHORIZED);
    db.cleanup().await;
}

#[tokio::test]
async fn refresh_without_any_token_or_with_a_form_post_is_rejected() {
    let db = TestDb::new().await;
    let (app, _) = setup(&db).await;
    let login = send(
        &app,
        post_json(
            "/api/v1/auth/login",
            &json!({ "username": "ops", "password": PASSWORD, "session": "cookie" }),
        ),
    )
    .await;
    let cookie = cookie_pair(&login.headers);

    let nothing = send(&app, post_json("/api/v1/auth/refresh", &json!({}))).await;
    assert_eq!(nothing.status, StatusCode::UNAUTHORIZED);

    // What a cross-site HTML form could send: not JSON, so it is refused
    // before the cookie is even looked at.
    let form = Request::post("/api/v1/auth/refresh")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .header(header::COOKIE, &cookie)
        .body(Body::from("x=1"))
        .unwrap();
    assert_eq!(
        send(&app, form).await.status,
        StatusCode::UNSUPPORTED_MEDIA_TYPE
    );
    db.cleanup().await;
}
