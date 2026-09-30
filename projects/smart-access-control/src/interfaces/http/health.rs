//! Liveness and readiness endpoints.

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::Serialize;

use super::AppState;
use crate::infrastructure::postgres;

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub version: &'static str,
}

#[derive(Debug, Serialize)]
pub struct ReadinessResponse {
    pub status: &'static str,
}

/// `GET /api/v1/health`: the process is up. Never touches dependencies, so an
/// orchestrator will not restart the app just because the database blipped.
pub async fn live() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
    })
}

/// `GET /api/v1/health/ready`: the app can serve traffic (database reachable).
///
/// The failure body is deliberately generic; the cause is logged, not returned.
pub async fn ready(State(state): State<AppState>) -> impl IntoResponse {
    match postgres::ping(&state.db).await {
        Ok(()) => (StatusCode::OK, Json(ReadinessResponse { status: "ready" })),
        Err(err) => {
            tracing::warn!(error = %err, "readiness check failed: database unreachable");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ReadinessResponse {
                    status: "unavailable",
                }),
            )
        }
    }
}
