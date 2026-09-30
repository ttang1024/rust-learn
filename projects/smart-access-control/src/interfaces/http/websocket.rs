//! `GET /api/v1/events/stream`: live access events over a WebSocket.
//!
//! # Protocol
//!
//! 1. Connect. Within 5 seconds send
//!    `{"type": "subscribe", "token": "<access token>", "filter": {...}}`.
//!    `filter` is optional and takes the same fields as `GET /events`
//!    (`door_id`, `user_id`, `card_id`, `decision`, `reason`, `from`, `until`).
//! 2. The server answers `{"type": "subscribed", "expires_at": "..."}`.
//! 3. Then, as they happen: `{"type": "access_event", "event": {...}}`, and
//!    `{"type": "lagged", "missed": n}` if this client fell too far behind.
//!
//! The token goes in the first message, not the URL: browsers cannot set an
//! `Authorization` header on WebSockets, and URLs end up in access logs.
//!
//! Close codes: `4400` malformed message, `4401` invalid token, `4408` no
//! subscribe message in time, `4409` access token expired (reconnect with a
//! fresh one), `1001` server shutting down.
//!
//! Delivery is best-effort; see `infrastructure::event_hub` for the exact
//! guarantees. Missed events can be fetched from `GET /events?from=...`.

use std::{sync::Arc, time::Duration};

use axum::{
    Router,
    extract::{
        State, WebSocketUpgrade,
        ws::{CloseFrame, Message, WebSocket, close_code},
    },
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast::error::RecvError;

use super::{
    ApiError, AppState,
    events::{EventResponse, FilterParams},
};
use crate::{
    application::{AccessClaims, EventFilter},
    domain::Timestamp,
};

/// Application-specific close codes (the 4000–4999 range is reserved for apps).
pub mod close_codes {
    pub const BAD_MESSAGE: u16 = 4400;
    pub const UNAUTHORIZED: u16 = 4401;
    pub const AUTH_TIMEOUT: u16 = 4408;
    pub const TOKEN_EXPIRED: u16 = 4409;
}

const AUTH_TIMEOUT: Duration = Duration::from_secs(5);
/// A client that cannot take one message within this time is disconnected,
/// so a stalled TCP connection cannot pin server resources forever.
const SEND_TIMEOUT: Duration = Duration::from_secs(10);
/// Clients only ever send one small subscribe message.
const MAX_CLIENT_MESSAGE_BYTES: usize = 16 * 1024;

pub fn routes() -> Router<AppState> {
    Router::new().route("/events/stream", get(stream))
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum ClientMessage {
    Subscribe {
        token: String,
        #[serde(default)]
        filter: FilterParams,
    },
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ServerMessage {
    Subscribed { expires_at: Timestamp },
    AccessEvent { event: EventResponse },
    Lagged { missed: u64 },
}

/// Refuses the upgrade with `503` when `WS_MAX_CONNECTIONS` streams are
/// already open. The permit is taken before upgrading and released when the
/// connection's task ends, however it ends.
async fn stream(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    let Ok(permit) = Arc::clone(&state.ws_permits).try_acquire_owned() else {
        tracing::warn!("event stream refused: connection limit reached");
        return ApiError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "too_many_connections",
            "too many live event connections; try again later",
        )
        .into_response();
    };
    ws.max_message_size(MAX_CLIENT_MESSAGE_BYTES)
        .on_upgrade(move |socket| async move {
            let open = metrics::gauge!("websocket_connections");
            open.increment(1.0);
            connection(socket, state).await;
            open.decrement(1.0);
            drop(permit);
        })
}

/// One task per connected client. It owns its socket and its broadcast
/// receiver, so a slow or stuck client only ever delays itself.
async fn connection(mut socket: WebSocket, state: AppState) {
    let (claims, filter) = match authenticate(&mut socket, &state).await {
        Ok(subscription) => subscription,
        Err((code, reason)) => return close(&mut socket, code, reason).await,
    };

    // Subscribe before acknowledging, so no event published in between is lost.
    let mut events = state.event_hub.subscribe();
    let subscribed = ServerMessage::Subscribed {
        expires_at: claims.expires_at,
    };
    if !send(&mut socket, &subscribed).await {
        return;
    }
    tracing::debug!(admin_id = %claims.admin_id, "event stream opened");

    // The stream lives no longer than the token that opened it.
    let remaining = (claims.expires_at - state.clock.now())
        .to_std()
        .unwrap_or_default();
    let token_expiry = tokio::time::sleep(remaining);
    // `select!` polls this timer many times across loop iterations; pinning
    // it in place lets us keep polling the *same* timer by `&mut` reference.
    tokio::pin!(token_expiry);

    loop {
        tokio::select! {
            received = events.recv() => {
                let message = match received {
                    Ok(event) if filter.matches(&event) => ServerMessage::AccessEvent {
                        event: EventResponse::from(event.as_ref()),
                    },
                    Ok(_) => continue,
                    Err(RecvError::Lagged(missed)) => ServerMessage::Lagged { missed },
                    Err(RecvError::Closed) => break,
                };
                if !send(&mut socket, &message).await {
                    break; // gone, or too slow to accept a message
                }
            }
            incoming = socket.recv() => match incoming {
                // The client is closing. tungstenite has queued our reply
                // but only writes it on the next read or write, so read
                // once more (bounded, in case the client never hangs up).
                Some(Ok(Message::Close(_))) => {
                    let _ = tokio::time::timeout(SEND_TIMEOUT, socket.recv()).await;
                    break;
                }
                // Pings are answered automatically; anything else is ignored.
                Some(Err(_)) | None => break,
                Some(Ok(_)) => {}
            },
            () = &mut token_expiry => {
                close(&mut socket, close_codes::TOKEN_EXPIRED, "access token expired").await;
                break;
            }
            () = state.shutdown_requested() => {
                close(&mut socket, close_code::AWAY, "server shutting down").await;
                break;
            }
        }
    }
    tracing::debug!(admin_id = %claims.admin_id, "event stream closed");
}

/// Waits for the subscribe message and checks its token and filter.
async fn authenticate(
    socket: &mut WebSocket,
    state: &AppState,
) -> Result<(AccessClaims, EventFilter), (u16, &'static str)> {
    let text = tokio::time::timeout(AUTH_TIMEOUT, next_text(socket))
        .await
        .map_err(|_| (close_codes::AUTH_TIMEOUT, "no subscribe message received"))?
        .ok_or((close_codes::BAD_MESSAGE, "expected a subscribe message"))?;

    let ClientMessage::Subscribe { token, filter } = serde_json::from_str(&text)
        .map_err(|_| (close_codes::BAD_MESSAGE, "invalid subscribe message"))?;
    let claims = state
        .auth
        .authenticate(&token)
        .ok_or((close_codes::UNAUTHORIZED, "invalid credentials"))?;
    let filter = filter
        .into_filter()
        .map_err(|_| (close_codes::BAD_MESSAGE, "invalid filter"))?;
    Ok((claims, filter))
}

/// The next text message, skipping pings. `None` if the client sent
/// something else or went away.
async fn next_text(socket: &mut WebSocket) -> Option<String> {
    loop {
        match socket.recv().await? {
            Ok(Message::Text(text)) => return Some(text.to_string()),
            Ok(Message::Ping(_) | Message::Pong(_)) => continue,
            _ => return None,
        }
    }
}

/// Sends one message; `false` if the client is gone or did not accept it in time.
async fn send(socket: &mut WebSocket, message: &ServerMessage) -> bool {
    let Ok(text) = serde_json::to_string(message) else {
        return false;
    };
    matches!(
        tokio::time::timeout(SEND_TIMEOUT, socket.send(Message::Text(text.into()))).await,
        Ok(Ok(()))
    )
}

async fn close(socket: &mut WebSocket, code: u16, reason: &'static str) {
    let frame = CloseFrame {
        code,
        reason: reason.into(),
    };
    let _ = tokio::time::timeout(SEND_TIMEOUT, socket.send(Message::Close(Some(frame)))).await;
}
