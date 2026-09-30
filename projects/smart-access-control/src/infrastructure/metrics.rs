//! Prometheus metrics.
//!
//! Code everywhere records through the `metrics` facade macros
//! (`counter!`, `gauge!`, `histogram!`), the way it logs through `tracing`:
//! it never knows where the numbers go. This module installs the one global
//! recorder, which keeps them in memory and renders them for `GET /metrics`.
//!
//! Label values must come from small, fixed sets (a route pattern, a denial
//! reason), never from user input such as ids or raw URLs: every distinct
//! label combination is a separate time series held in memory.

use std::sync::OnceLock;

use metrics::{Unit, describe_counter, describe_gauge, describe_histogram};
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle};

/// Request latency buckets in seconds (1 ms … 10 s).
const LATENCY_BUCKETS: [f64; 11] = [
    0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 10.0,
];

static HANDLE: OnceLock<PrometheusHandle> = OnceLock::new();

/// Installs the global recorder (once per process) and returns a handle to
/// render its contents. Safe to call repeatedly, e.g. from many tests.
pub fn install() -> PrometheusHandle {
    HANDLE
        .get_or_init(|| {
            let recorder = PrometheusBuilder::new()
                .set_buckets_for_metric(
                    Matcher::Full("http_request_duration_seconds".into()),
                    &LATENCY_BUCKETS,
                )
                .expect("latency buckets are non-empty")
                .build_recorder();
            let handle = recorder.handle();
            // Fails only if another recorder was installed first; then ours
            // simply stays unused, which is harmless.
            let _ = metrics::set_global_recorder(recorder);
            describe();
            handle
        })
        .clone()
}

/// Help texts shown in the Prometheus output.
fn describe() {
    describe_counter!(
        "http_requests_total",
        "HTTP requests by method, route and status"
    );
    describe_histogram!(
        "http_request_duration_seconds",
        Unit::Seconds,
        "HTTP request latency by method and route"
    );
    describe_counter!(
        "access_decisions_total",
        "Access decisions by outcome and denial reason"
    );
    describe_counter!(
        "auth_login_attempts_total",
        "Administrator login attempts by outcome (success, failure, throttled)"
    );
    describe_counter!(
        "auth_refresh_token_reuse_total",
        "Rotated refresh tokens presented again (sessions revoked)"
    );
    describe_gauge!("websocket_connections", "Open live-event WebSocket streams");
    describe_counter!(
        "controllers_expired_total",
        "Controllers marked offline for missing heartbeats"
    );
}
