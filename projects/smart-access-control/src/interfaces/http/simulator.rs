//! `/simulator/*`: drive simulated door controllers (admin only).
//!
//! Mounted only when `SIMULATOR_ENABLED=true`. Simulated controllers use the
//! same controller protocol as a device would (key, heartbeats, requests),
//! through an in-process link instead of the network.

use std::{sync::Arc, time::Duration};

use axum::{
    Json, Router,
    extract::{FromRef, State},
    http::StatusCode,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    ApiError, ApiJson, ApiPath, AppState, AuthenticatedAdmin, RequireAdmin, events::EventResponse,
    subject,
};
use crate::{
    application::{AccessRequest, ApplicationError, HeartbeatReport},
    domain::{AccessEvent, AuditAction, DoorId, Timestamp},
    simulator::{ControllerLink, SimulatedStatus, Simulator, SimulatorError},
};

/// State for the simulator routes: the application plus the simulator.
#[derive(Clone)]
pub struct SimulatorState {
    pub app: AppState,
    pub simulator: Arc<Simulator>,
}

/// Lets `AuthenticatedAdmin` / `RequireAdmin` work on these routes too.
impl FromRef<SimulatorState> for AppState {
    fn from_ref(state: &SimulatorState) -> Self {
        state.app.clone()
    }
}

pub fn routes() -> Router<SimulatorState> {
    Router::new()
        .route("/simulator/controllers", get(list).post(start))
        .route("/simulator/controllers/{id}", get(get_one).delete(stop))
        .route("/simulator/controllers/{id}/swipe", post(swipe))
        .route("/simulator/controllers/{id}/outage", post(outage))
        .route("/simulator/controllers/{id}/disconnect", post(disconnect))
        .route("/simulator/controllers/{id}/reconnect", post(reconnect))
}

/// Reaches the backend through the application services directly, making
/// exactly the calls the `/device/*` HTTP handlers make, key check included.
pub struct InProcessLink {
    state: AppState,
    key: String,
}

impl ControllerLink for InProcessLink {
    async fn heartbeat(&self) -> Result<HeartbeatReport, ApplicationError> {
        let controller = self.state.controllers.authenticate(&self.key).await?;
        self.state.controllers.heartbeat(&controller).await
    }

    async fn request_access(
        &self,
        card_number: String,
        door_id: DoorId,
    ) -> Result<AccessEvent, ApplicationError> {
        let controller = self.state.controllers.authenticate(&self.key).await?;
        self.state
            .controllers
            .authorize_request(&controller, door_id)
            .await?;
        self.state
            .access
            .decide(AccessRequest {
                card_number,
                door_id,
            })
            .await
    }

    async fn disconnect(&self) -> Result<(), ApplicationError> {
        let controller = self.state.controllers.authenticate(&self.key).await?;
        self.state.controllers.disconnect(&controller).await
    }
}

#[derive(Debug, Serialize)]
pub struct SimulatedControllerResponse {
    pub controller_id: String,
    /// Always `true`: marks this as a simulation, not a device.
    pub simulated: bool,
    /// `up` or `down` (simulated network).
    pub network: &'static str,
    pub heartbeats_sent: u64,
    pub last_heartbeat_at: Option<Timestamp>,
    pub last_error: Option<String>,
}

impl SimulatedControllerResponse {
    fn new(controller_id: String, status: SimulatedStatus) -> Self {
        Self {
            controller_id,
            simulated: true,
            network: if status.network_up { "up" } else { "down" },
            heartbeats_sent: status.heartbeats_sent,
            last_heartbeat_at: status.last_heartbeat_at,
            last_error: status.last_error,
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StartRequest {
    pub controller_id: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SwipeRequest {
    pub card_number: String,
    pub door_id: Uuid,
}

impl From<SimulatorError> for ApiError {
    fn from(err: SimulatorError) -> Self {
        match err {
            SimulatorError::NotRunning(_) => {
                Self::new(StatusCode::NOT_FOUND, "not_found", err.to_string())
            }
            SimulatorError::AlreadyRunning(_) => {
                Self::new(StatusCode::CONFLICT, "conflict", err.to_string())
            }
            SimulatorError::Offline(_) => Self::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "controller_offline",
                err.to_string(),
            ),
            SimulatorError::Backend(err) => err.into(),
        }
    }
}

fn describe(state: &SimulatorState, id: &str) -> Result<SimulatedControllerResponse, ApiError> {
    let status = state.simulator.status(id)?;
    Ok(SimulatedControllerResponse::new(id.to_owned(), status))
}

/// Starts simulating a controller. Registers it if needed; if it is already
/// registered, its key is rotated, because the existing key is stored only
/// as a hash and cannot be recovered for the simulator to use.
async fn start(
    RequireAdmin(admin): RequireAdmin,
    State(state): State<SimulatorState>,
    ApiJson(body): ApiJson<StartRequest>,
) -> Result<(StatusCode, Json<SimulatedControllerResponse>), ApiError> {
    let id = body.controller_id;
    if state.simulator.is_running(&id) {
        return Err(SimulatorError::AlreadyRunning(id).into());
    }
    let controllers = &state.app.controllers;
    let (issued, action) = match controllers.register(&id).await {
        Ok(issued) => (issued, AuditAction::ControllerRegistered),
        Err(ApplicationError::Conflict { .. }) => (
            controllers.rotate_key(&id).await?,
            AuditAction::ControllerKeyRotated,
        ),
        Err(err) => return Err(err.into()),
    };
    let id = issued.controller.id().to_string();
    state
        .app
        .audit
        .record(admin.admin_id, action, &subject("controller", &id))
        .await;

    // Same cadence as the liveness monitor: three heartbeats per timeout.
    let timeout = controllers
        .timeout()
        .to_std()
        .unwrap_or(Duration::from_secs(30));
    let heartbeat_every = (timeout / 3).max(Duration::from_secs(1));
    let clock = Arc::clone(&state.app.clock);
    let link = InProcessLink {
        state: state.app.clone(),
        key: issued.key,
    };
    state
        .simulator
        .start(&id, link, heartbeat_every, move || clock.now())?;
    tracing::info!(controller_id = %id, "simulated controller started");
    Ok((StatusCode::CREATED, Json(describe(&state, &id)?)))
}

async fn list(
    _: AuthenticatedAdmin,
    State(state): State<SimulatorState>,
) -> Json<Vec<SimulatedControllerResponse>> {
    Json(
        state
            .simulator
            .list()
            .into_iter()
            .map(|(id, status)| SimulatedControllerResponse::new(id, status))
            .collect(),
    )
}

async fn get_one(
    _: AuthenticatedAdmin,
    State(state): State<SimulatorState>,
    ApiPath(id): ApiPath<String>,
) -> Result<Json<SimulatedControllerResponse>, ApiError> {
    Ok(Json(describe(&state, &id)?))
}

/// Presents a card at one of the simulated controller's doors.
async fn swipe(
    _: RequireAdmin,
    State(state): State<SimulatorState>,
    ApiPath(id): ApiPath<String>,
    ApiJson(body): ApiJson<SwipeRequest>,
) -> Result<Json<EventResponse>, ApiError> {
    let event = state
        .simulator
        .swipe(&id, body.card_number, DoorId::from_uuid(body.door_id))
        .await?;
    Ok(Json(EventResponse::from(&event)))
}

/// Cuts the simulated network without warning; the backend notices only
/// when heartbeats stop arriving.
async fn outage(
    _: RequireAdmin,
    State(state): State<SimulatorState>,
    ApiPath(id): ApiPath<String>,
) -> Result<Json<SimulatedControllerResponse>, ApiError> {
    state.simulator.outage(&id).await?;
    Ok(Json(describe(&state, &id)?))
}

async fn disconnect(
    _: RequireAdmin,
    State(state): State<SimulatorState>,
    ApiPath(id): ApiPath<String>,
) -> Result<Json<SimulatedControllerResponse>, ApiError> {
    state.simulator.disconnect(&id).await?;
    Ok(Json(describe(&state, &id)?))
}

async fn reconnect(
    _: RequireAdmin,
    State(state): State<SimulatorState>,
    ApiPath(id): ApiPath<String>,
) -> Result<Json<SimulatedControllerResponse>, ApiError> {
    state.simulator.reconnect(&id).await?;
    Ok(Json(describe(&state, &id)?))
}

async fn stop(
    _: RequireAdmin,
    State(state): State<SimulatorState>,
    ApiPath(id): ApiPath<String>,
) -> Result<StatusCode, ApiError> {
    state.simulator.stop(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}
