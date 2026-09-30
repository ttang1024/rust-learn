//! `/doors`: physical or simulated access points.

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, patch},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    ApiError, ApiJson, ApiPath, ApiQuery, AppState, AuthenticatedAdmin, Page, PageParams,
    RequireAdmin, subject,
};
use crate::{
    application::{ApplicationError, CreateDoor, PageRequest, UpdateDoor},
    domain::{AuditAction, Door, DoorId, DoorStatus, Timestamp},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/doors", get(list).post(create))
        .route("/doors/{id}", get(get_one).patch(update))
        .route("/doors/{id}/status", patch(set_status))
}

#[derive(Debug, Serialize)]
pub struct DoorResponse {
    pub id: Uuid,
    pub name: String,
    pub location: String,
    pub controller_id: String,
    pub status: &'static str,
    pub created_at: Timestamp,
}

impl From<&Door> for DoorResponse {
    fn from(door: &Door) -> Self {
        Self {
            id: door.id().as_uuid(),
            name: door.name().to_string(),
            location: door.location().to_string(),
            controller_id: door.controller_id().to_string(),
            status: door.status().as_str(),
            created_at: door.created_at(),
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateDoorRequest {
    pub name: String,
    pub location: String,
    pub controller_id: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateDoorRequest {
    pub name: Option<String>,
    pub location: Option<String>,
    pub controller_id: Option<String>,
}

/// Target status: `online`, `offline` or `disabled`.
///
/// Until simulated controllers exist (Phase 6), this is also how a door is
/// brought online. `offline` on a disabled door re-enables it (it returns to
/// service offline, waiting for its controller).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DoorStatusRequest {
    pub status: String,
}

async fn list(
    _: AuthenticatedAdmin,
    State(state): State<AppState>,
    ApiQuery(params): ApiQuery<PageParams>,
) -> Result<Json<Page<DoorResponse>>, ApiError> {
    let page = PageRequest::from(params);
    let doors = state.doors.list(page).await?;
    Ok(Json(Page::new(
        doors.iter().map(DoorResponse::from).collect(),
        page,
    )))
}

async fn get_one(
    _: AuthenticatedAdmin,
    State(state): State<AppState>,
    ApiPath(id): ApiPath<Uuid>,
) -> Result<Json<DoorResponse>, ApiError> {
    let door = state.doors.get(DoorId::from_uuid(id)).await?;
    Ok(Json(DoorResponse::from(&door)))
}

async fn create(
    RequireAdmin(admin): RequireAdmin,
    State(state): State<AppState>,
    ApiJson(body): ApiJson<CreateDoorRequest>,
) -> Result<(StatusCode, Json<DoorResponse>), ApiError> {
    let door = state
        .doors
        .create(CreateDoor {
            name: body.name,
            location: body.location,
            controller_id: body.controller_id,
        })
        .await?;
    state
        .audit
        .record(
            admin.admin_id,
            AuditAction::DoorCreated,
            &subject("door", door.id()),
        )
        .await;
    Ok((StatusCode::CREATED, Json(DoorResponse::from(&door))))
}

async fn update(
    RequireAdmin(admin): RequireAdmin,
    State(state): State<AppState>,
    ApiPath(id): ApiPath<Uuid>,
    ApiJson(body): ApiJson<UpdateDoorRequest>,
) -> Result<Json<DoorResponse>, ApiError> {
    let door = state
        .doors
        .update(
            DoorId::from_uuid(id),
            UpdateDoor {
                name: body.name,
                location: body.location,
                controller_id: body.controller_id,
            },
        )
        .await?;
    state
        .audit
        .record(
            admin.admin_id,
            AuditAction::DoorUpdated,
            &subject("door", door.id()),
        )
        .await;
    Ok(Json(DoorResponse::from(&door)))
}

async fn set_status(
    RequireAdmin(admin): RequireAdmin,
    State(state): State<AppState>,
    ApiPath(id): ApiPath<Uuid>,
    ApiJson(body): ApiJson<DoorStatusRequest>,
) -> Result<Json<DoorResponse>, ApiError> {
    let id = DoorId::from_uuid(id);
    let target: DoorStatus = body.status.parse().map_err(ApplicationError::from)?;
    let door = match target {
        DoorStatus::Online => state.doors.report_online(id).await?,
        DoorStatus::Disabled => state.doors.disable(id).await?,
        DoorStatus::Offline => {
            if state.doors.get(id).await?.status() == DoorStatus::Disabled {
                state.doors.enable(id).await?
            } else {
                state.doors.report_offline(id).await?
            }
        }
    };
    let what = format!("{} -> {}", subject("door", id), door.status().as_str());
    state
        .audit
        .record(admin.admin_id, AuditAction::DoorStatusChanged, &what)
        .await;
    Ok(Json(DoorResponse::from(&door)))
}
