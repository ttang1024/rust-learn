//! `/device/*`: the API that (simulated) door controllers call.
//!
//! Authenticated with the controller's key in the `X-Controller-Key` header,
//! a credential separate from administrator tokens: an admin token does not
//! work here, and a controller key does not work anywhere else.
//!
//! Every successful call counts as a heartbeat. A controller that makes no
//! call for `CONTROLLER_TIMEOUT_SECONDS` is marked offline with its doors.

use axum::{
    Json, Router,
    extract::{FromRequestParts, State},
    http::{StatusCode, request::Parts},
    routing::post,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{ApiError, ApiJson, AppState, events::EventResponse};
use crate::{
    application::{AccessRequest, ApplicationError},
    domain::{Controller, DoorId},
};

pub const CONTROLLER_KEY_HEADER: &str = "x-controller-key";

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/device/heartbeat", post(heartbeat))
        .route("/device/access-requests", post(access_request))
        .route("/device/disconnect", post(disconnect))
}

/// The controller that owns the key in `X-Controller-Key`.
pub struct AuthenticatedController(pub Controller);

impl FromRequestParts<AppState> for AuthenticatedController {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, ApiError> {
        let invalid = || {
            ApiError::new(
                StatusCode::UNAUTHORIZED,
                "unauthorized",
                "invalid controller key",
            )
        };
        let key = parts
            .headers
            .get(CONTROLLER_KEY_HEADER)
            .and_then(|value| value.to_str().ok())
            .filter(|key| !key.is_empty())
            .ok_or_else(invalid)?;
        match state.controllers.authenticate(key).await {
            Ok(controller) => Ok(Self(controller)),
            Err(ApplicationError::Unauthorized) => Err(invalid()),
            Err(other) => Err(other.into()),
        }
    }
}

#[derive(Serialize)]
pub struct HeartbeatResponse {
    pub controller_id: String,
    pub doors_online: usize,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeviceAccessRequest {
    pub card_number: String,
    pub door_id: Uuid,
}

async fn heartbeat(
    AuthenticatedController(controller): AuthenticatedController,
    State(state): State<AppState>,
) -> Result<Json<HeartbeatResponse>, ApiError> {
    let report = state.controllers.heartbeat(&controller).await?;
    Ok(Json(HeartbeatResponse {
        controller_id: controller.id().to_string(),
        doors_online: report.doors_online,
    }))
}

/// A card was presented at one of this controller's doors. The response
/// carries the decision: open the door only for `"decision": "granted"`,
/// and treat any error as denied.
async fn access_request(
    AuthenticatedController(controller): AuthenticatedController,
    State(state): State<AppState>,
    ApiJson(body): ApiJson<DeviceAccessRequest>,
) -> Result<Json<EventResponse>, ApiError> {
    let door_id = DoorId::from_uuid(body.door_id);
    state
        .controllers
        .authorize_request(&controller, door_id)
        .await?;
    let event = state
        .access
        .decide(AccessRequest {
            card_number: body.card_number,
            door_id,
        })
        .await?;
    Ok(Json(EventResponse::from(&event)))
}

/// Clean shutdown of a controller: its doors go offline immediately instead
/// of after the heartbeat timeout.
async fn disconnect(
    AuthenticatedController(controller): AuthenticatedController,
    State(state): State<AppState>,
) -> Result<StatusCode, ApiError> {
    state.controllers.disconnect(&controller).await?;
    Ok(StatusCode::NO_CONTENT)
}
