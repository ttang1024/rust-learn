//! `/permissions`: which access group may use which door, and when.

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{delete, get},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    ApiError, ApiJson, ApiPath, ApiQuery, AppState, AuthenticatedAdmin, Page, PageParams,
    RequireAdmin, subject,
};
use crate::{
    application::{GrantPermission, PageRequest},
    domain::{
        AccessGroupId, AccessPermission, AuditAction, DoorId, PermissionId, ScheduleId, Timestamp,
    },
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/permissions", get(list).post(grant))
        .route("/permissions/{id}", delete(revoke))
}

#[derive(Debug, Serialize)]
pub struct PermissionResponse {
    pub id: Uuid,
    pub group_id: Uuid,
    pub door_id: Uuid,
    /// `null` = at any time.
    pub schedule_id: Option<Uuid>,
    pub created_at: Timestamp,
}

impl From<&AccessPermission> for PermissionResponse {
    fn from(permission: &AccessPermission) -> Self {
        Self {
            id: permission.id().as_uuid(),
            group_id: permission.group_id().as_uuid(),
            door_id: permission.door_id().as_uuid(),
            schedule_id: permission.schedule_id().map(|id| id.as_uuid()),
            created_at: permission.created_at(),
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GrantPermissionRequest {
    pub group_id: Uuid,
    pub door_id: Uuid,
    pub schedule_id: Option<Uuid>,
}

async fn list(
    _: AuthenticatedAdmin,
    State(state): State<AppState>,
    ApiQuery(params): ApiQuery<PageParams>,
) -> Result<Json<Page<PermissionResponse>>, ApiError> {
    let page = PageRequest::from(params);
    let permissions = state.permissions.list(page).await?;
    Ok(Json(Page::new(
        permissions.iter().map(PermissionResponse::from).collect(),
        page,
    )))
}

async fn grant(
    RequireAdmin(admin): RequireAdmin,
    State(state): State<AppState>,
    ApiJson(body): ApiJson<GrantPermissionRequest>,
) -> Result<(StatusCode, Json<PermissionResponse>), ApiError> {
    let permission = state
        .permissions
        .grant(GrantPermission {
            group_id: AccessGroupId::from_uuid(body.group_id),
            door_id: DoorId::from_uuid(body.door_id),
            schedule_id: body.schedule_id.map(ScheduleId::from_uuid),
        })
        .await?;
    state
        .audit
        .record(
            admin.admin_id,
            AuditAction::PermissionGranted,
            &subject("permission", permission.id()),
        )
        .await;
    Ok((
        StatusCode::CREATED,
        Json(PermissionResponse::from(&permission)),
    ))
}

async fn revoke(
    RequireAdmin(admin): RequireAdmin,
    State(state): State<AppState>,
    ApiPath(id): ApiPath<Uuid>,
) -> Result<StatusCode, ApiError> {
    let id = PermissionId::from_uuid(id);
    state.permissions.revoke(id).await?;
    state
        .audit
        .record(
            admin.admin_id,
            AuditAction::PermissionRevoked,
            &subject("permission", id),
        )
        .await;
    Ok(StatusCode::NO_CONTENT)
}
