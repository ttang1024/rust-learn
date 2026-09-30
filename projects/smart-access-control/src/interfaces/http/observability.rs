//! Request ids, HTTP metrics and the Prometheus endpoint.

use std::time::Instant;

use axum::{
    body::Body,
    extract::{MatchedPath, Request, State},
    http::{HeaderValue, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use tracing::Span;

use super::{ApiError, AppState};

const REQUEST_ID: &str = "x-request-id";

/// Removes an incoming `x-request-id` that is too long or contains unusual
/// characters, so a fresh one is generated instead. Ids set by a trusted
/// proxy are kept, which lets its logs and ours be correlated; anything
/// else a client sends ends up in logs, so it is kept boring.
pub async fn drop_invalid_request_id(mut request: Request) -> Request {
    let valid = request
        .headers()
        .get(REQUEST_ID)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|id| {
            !id.is_empty()
                && id.len() <= 64
                && id
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"-_.:".contains(&b))
        });
    if !valid {
        request.headers_mut().remove(REQUEST_ID);
    }
    request
}

/// The span every log line of a request is recorded in. Its `request_id`
/// appears on each line (and in JSON logs as a field).
pub fn request_span(request: &Request<Body>) -> Span {
    let request_id = request
        .headers()
        .get(REQUEST_ID)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("-");
    tracing::info_span!(
        "request",
        method = %request.method(),
        uri = %request.uri().path(),
        request_id = %request_id,
    )
}

/// Counts requests and measures latency per method, route pattern and status.
pub async fn track_http_metrics(request: Request, next: Next) -> Response {
    let started = Instant::now();
    let method = request.method().to_string();
    // The route pattern (`/api/v1/users/{id}`), never the raw path: raw
    // paths contain ids and would create unbounded label values.
    let path = request
        .extensions()
        .get::<MatchedPath>()
        .map(|path| path.as_str().to_owned())
        .unwrap_or_else(|| "unmatched".to_owned());

    let response = next.run(request).await;

    let status = response.status().as_u16().to_string();
    metrics::counter!(
        "http_requests_total",
        "method" => method.clone(),
        "path" => path.clone(),
        "status" => status,
    )
    .increment(1);
    metrics::histogram!(
        "http_request_duration_seconds",
        "method" => method,
        "path" => path,
    )
    .record(started.elapsed().as_secs_f64());
    response
}

/// `GET /metrics`: Prometheus text format, for scrapers presenting the
/// configured bearer token. Only mounted when `METRICS_TOKEN` is set.
pub async fn metrics(State(state): State<AppState>, request: Request) -> Response {
    let presented = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "));
    let authorised = match (&state.metrics_token, presented) {
        (Some(token), Some(presented)) => token.matches(presented),
        _ => false,
    };
    if !authorised {
        return ApiError::new(
            StatusCode::UNAUTHORIZED,
            "unauthorized",
            "invalid metrics token",
        )
        .into_response();
    }
    // Keeps the recorder's internal state bounded; cheap.
    state.metrics.run_upkeep();
    (
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/plain; version=0.0.4; charset=utf-8"),
        )],
        state.metrics.render(),
    )
        .into_response()
}
