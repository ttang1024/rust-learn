//! The live event stream, over a real TCP socket with a real WebSocket client.

mod common;

use std::{net::SocketAddr, sync::Arc, time::Duration};

use common::{TestDb, auth_config};
use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use smart_access_control::{
    application::{AccessRequest, CreateAdministrator, CreateDoor},
    config::AuthConfig,
    domain::{DoorId, Role},
    infrastructure::clock::SystemClock,
    interfaces::http::{AppOptions, AppState, close_codes, router},
};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async,
    tungstenite::{Message, protocol::frame::coding::CloseCode},
};

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

const PASSWORD: &str = "correct horse battery staple";

struct Server {
    addr: SocketAddr,
    state: AppState,
}

/// Starts the real router on a random local port.
async fn server(db: &TestDb, auth: AuthConfig) -> Server {
    server_with_limit(db, auth, 256).await
}

async fn server_with_limit(db: &TestDb, auth: AuthConfig, max_streams: usize) -> Server {
    let state = AppState::new(
        db.pool(),
        &auth,
        AppOptions {
            controller_timeout: chrono::Duration::seconds(30),
            max_ws_connections: max_streams,
            ..AppOptions::default()
        },
        Arc::new(SystemClock),
    )
    .unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = router(state.clone());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    Server { addr, state }
}

impl Server {
    async fn token(&self, username: &str, role: Role) -> String {
        self.state
            .auth
            .create_administrator(CreateAdministrator {
                username: username.into(),
                password: PASSWORD.into(),
                role,
            })
            .await
            .unwrap();
        self.state
            .auth
            .login(username, PASSWORD)
            .await
            .unwrap()
            .access_token
    }

    async fn connect(&self) -> Socket {
        let url = format!("ws://{}/api/v1/events/stream", self.addr);
        connect_async(url).await.unwrap().0
    }

    /// Connects and subscribes; returns once the server confirmed.
    async fn subscribe(&self, token: &str, filter: Value) -> Socket {
        let mut socket = self.connect().await;
        send(
            &mut socket,
            json!({ "type": "subscribe", "token": token, "filter": filter }),
        )
        .await;
        let reply = next_json(&mut socket).await;
        assert_eq!(reply["type"], "subscribed", "{reply}");
        socket
    }

    async fn door(&self, name: &str) -> DoorId {
        self.state
            .doors
            .create(CreateDoor {
                name: name.into(),
                location: "Lab".into(),
                controller_id: "ctrl-1".into(),
            })
            .await
            .unwrap()
            .id()
    }

    /// Produces one (denied) access event at `door`.
    async fn attempt(&self, door: DoorId, card: &str) {
        self.state
            .access
            .decide(AccessRequest {
                card_number: card.into(),
                door_id: door,
            })
            .await
            .unwrap();
    }
}

async fn send(socket: &mut Socket, message: Value) {
    socket
        .send(Message::text(message.to_string()))
        .await
        .unwrap();
}

/// The next JSON message, failing the test after 5 seconds.
async fn next_json(socket: &mut Socket) -> Value {
    loop {
        let message = tokio::time::timeout(Duration::from_secs(5), socket.next())
            .await
            .expect("timed out waiting for a message")
            .expect("stream ended")
            .expect("websocket error");
        match message {
            Message::Text(text) => return serde_json::from_str(&text).unwrap(),
            Message::Ping(_) | Message::Pong(_) => continue,
            other => panic!("expected text, got {other:?}"),
        }
    }
}

/// Waits for the server to close the connection and returns the close code.
async fn close_code(socket: &mut Socket, within: Duration) -> u16 {
    loop {
        let message = tokio::time::timeout(within, socket.next())
            .await
            .expect("timed out waiting for close")
            .expect("stream ended without a close frame")
            .expect("websocket error");
        if let Message::Close(frame) = message {
            return frame.map(|f| u16::from(f.code)).unwrap_or_default();
        }
    }
}

#[tokio::test]
async fn subscribers_receive_new_events() {
    let db = TestDb::new().await;
    let server = server(&db, auth_config()).await;
    let admin = server.token("ops", Role::Admin).await;
    let viewer = server.token("watcher", Role::Viewer).await;
    let door = server.door("Main").await;

    // Two dashboards, one per role: monitoring is a read operation.
    let mut a = server.subscribe(&admin, json!({})).await;
    let mut b = server.subscribe(&viewer, json!({})).await;

    server.attempt(door, "CARD-1").await;

    for socket in [&mut a, &mut b] {
        let message = next_json(socket).await;
        assert_eq!(message["type"], "access_event");
        assert_eq!(message["event"]["card_number"], "CARD-1");
        assert_eq!(message["event"]["door_id"], door.to_string());
        assert_eq!(message["event"]["decision"], "denied");
    }
    db.cleanup().await;
}

#[tokio::test]
async fn filters_apply_to_live_events() {
    let db = TestDb::new().await;
    let server = server(&db, auth_config()).await;
    let token = server.token("ops", Role::Admin).await;
    let (watched, other) = (server.door("Watched").await, server.door("Other").await);

    let mut socket = server
        .subscribe(
            &token,
            json!({ "door_id": watched.to_string(), "decision": "denied" }),
        )
        .await;

    server.attempt(other, "CARD-OTHER").await;
    server.attempt(watched, "CARD-WATCHED").await;

    // The first message is the watched door's: the other one was filtered out.
    let message = next_json(&mut socket).await;
    assert_eq!(message["event"]["card_number"], "CARD-WATCHED");
    db.cleanup().await;
}

#[tokio::test]
async fn invalid_token_is_rejected() {
    let db = TestDb::new().await;
    let server = server(&db, auth_config()).await;

    let mut socket = server.connect().await;
    send(
        &mut socket,
        json!({ "type": "subscribe", "token": "forged" }),
    )
    .await;

    let code = close_code(&mut socket, Duration::from_secs(5)).await;
    assert_eq!(code, close_codes::UNAUTHORIZED);
    db.cleanup().await;
}

#[tokio::test]
async fn malformed_messages_and_filters_are_rejected() {
    let db = TestDb::new().await;
    let server = server(&db, auth_config()).await;
    let token = server.token("ops", Role::Admin).await;

    for message in [
        json!({ "type": "hello" }),
        json!({ "type": "subscribe", "token": token, "filter": { "decision": "maybe" } }),
        json!({ "type": "subscribe", "token": token, "extra": 1 }),
    ] {
        let mut socket = server.connect().await;
        send(&mut socket, message.clone()).await;
        let code = close_code(&mut socket, Duration::from_secs(5)).await;
        assert_eq!(code, close_codes::BAD_MESSAGE, "{message}");
    }
    db.cleanup().await;
}

#[tokio::test]
async fn silent_clients_are_disconnected() {
    let db = TestDb::new().await;
    let server = server(&db, auth_config()).await;

    let mut socket = server.connect().await;

    // No subscribe message: the server gives up after 5 seconds.
    let code = close_code(&mut socket, Duration::from_secs(8)).await;
    assert_eq!(code, close_codes::AUTH_TIMEOUT);
    db.cleanup().await;
}

#[tokio::test]
async fn stream_ends_when_the_access_token_expires() {
    let db = TestDb::new().await;
    let short_lived = AuthConfig {
        access_token_ttl: chrono::Duration::seconds(2),
        ..auth_config()
    };
    let server = server(&db, short_lived).await;
    let token = server.token("ops", Role::Admin).await;

    let mut socket = server.subscribe(&token, json!({})).await;

    let code = close_code(&mut socket, Duration::from_secs(5)).await;
    assert_eq!(code, close_codes::TOKEN_EXPIRED);
    db.cleanup().await;
}

#[tokio::test]
async fn shutdown_closes_streams_with_going_away() {
    let db = TestDb::new().await;
    let server = server(&db, auth_config()).await;
    let token = server.token("ops", Role::Admin).await;
    let mut socket = server.subscribe(&token, json!({})).await;

    server.state.begin_shutdown();

    let code = close_code(&mut socket, Duration::from_secs(5)).await;
    assert_eq!(code, u16::from(CloseCode::Away));
    db.cleanup().await;
}

/// Regression: the server dropped the socket on the client's close frame
/// without sending its reply, so browsers reported an abnormal closure
/// (1006) every time the dashboard closed the stream.
#[tokio::test]
async fn client_initiated_close_completes_the_handshake() {
    let db = TestDb::new().await;
    let server = server(&db, auth_config()).await;
    let token = server.token("ops", Role::Admin).await;

    let mut socket = server.subscribe(&token, json!({})).await;
    socket.send(Message::Close(None)).await.unwrap();

    // The server echoes the close frame, then the stream ends cleanly.
    let reply = tokio::time::timeout(Duration::from_secs(5), socket.next())
        .await
        .expect("timed out waiting for the close reply");
    assert!(
        matches!(reply, Some(Ok(Message::Close(_)))),
        "expected a close frame, got {reply:?}"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn disconnected_clients_release_their_subscription() {
    let db = TestDb::new().await;
    let server = server(&db, auth_config()).await;
    let token = server.token("ops", Role::Admin).await;
    let door = server.door("Main").await;

    let socket = server.subscribe(&token, json!({})).await;
    assert_eq!(server.state.event_hub.subscriber_count(), 1);

    // The client vanishes without a close handshake.
    drop(socket);
    // The server notices on its next read or write; an event forces a write.
    let mut released = false;
    for _ in 0..50 {
        server.attempt(door, "CARD-1").await;
        if server.state.event_hub.subscriber_count() == 0 {
            released = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert!(released, "subscription was not released");
    db.cleanup().await;
}

#[tokio::test]
async fn connections_beyond_the_limit_are_refused_until_one_closes() {
    let db = TestDb::new().await;
    let server = server_with_limit(&db, auth_config(), 1).await;
    let token = server.token("ops", Role::Admin).await;
    let url = format!("ws://{}/api/v1/events/stream", server.addr);

    let first = server.subscribe(&token, json!({})).await;

    // The second upgrade is refused with 503 before any WebSocket exists.
    match connect_async(&url).await {
        Err(tokio_tungstenite::tungstenite::Error::Http(response)) => {
            assert_eq!(response.status(), 503);
        }
        other => panic!(
            "expected an HTTP 503 refusal, got {:?}",
            other.map(|_| "a connection")
        ),
    }

    // Closing the first stream frees its slot.
    drop(first);
    let mut reconnected = false;
    for _ in 0..50 {
        if connect_async(&url).await.is_ok() {
            reconnected = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert!(reconnected, "the slot was not released");
    db.cleanup().await;
}
