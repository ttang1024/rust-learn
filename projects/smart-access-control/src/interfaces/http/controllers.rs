//! `/controllers`: registering (simulated) door controllers and their keys.

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

use super::{
    ApiError, ApiJson, ApiPath, ApiQuery, AppState, AuthenticatedAdmin, Page, PageParams,
    RequireAdmin, subject,
};
use crate::{
    application::{IssuedControllerKey, PageRequest},
    domain::{AuditAction, Controller, Timestamp},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/controllers", get(list).post(register))
        .route("/controllers/{id}", get(get_one))
        .route("/controllers/{id}/rotate-key", post(rotate_key))
}

#[derive(Debug, Serialize)]
pub struct ControllerResponse {
    pub id: String,
    pub status: &'static str,
    pub last_seen_at: Option<Timestamp>,
    pub created_at: Timestamp,
}

impl From<&Controller> for ControllerResponse {
    fn from(controller: &Controller) -> Self {
        Self {
            id: controller.id().to_string(),
            status: controller.status().as_str(),
            last_seen_at: controller.last_seen_at(),
            created_at: controller.created_at(),
        }
    }
}

/// Returned once, at registration or key rotation. The key cannot be
/// retrieved again; losing it means rotating it.
#[derive(Serialize)]
pub struct ControllerKeyResponse {
    pub controller: ControllerResponse,
    /// Send as the `X-Controller-Key` header on `/device/*` requests.
    pub key: String,
}

impl From<IssuedControllerKey> for ControllerKeyResponse {
    fn from(issued: IssuedControllerKey) -> Self {
        Self {
            controller: ControllerResponse::from(&issued.controller),
            key: issued.key,
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegisterControllerRequest {
    /// The id doors use in `controller_id`, e.g. `ctrl-001`.
    pub controller_id: String,
}

async fn list(
    _: AuthenticatedAdmin,
    State(state): State<AppState>,
    ApiQuery(params): ApiQuery<PageParams>,
) -> Result<Json<Page<ControllerResponse>>, ApiError> {
    let page = PageRequest::from(params);
    let controllers = state.controllers.list(page).await?;
    Ok(Json(Page::new(
        controllers.iter().map(ControllerResponse::from).collect(),
        page,
    )))
}

async fn get_one(
    _: AuthenticatedAdmin,
    State(state): State<AppState>,
    ApiPath(id): ApiPath<String>,
) -> Result<Json<ControllerResponse>, ApiError> {
    let controller = state.controllers.get(&id).await?;
    Ok(Json(ControllerResponse::from(&controller)))
}

async fn register(
    RequireAdmin(admin): RequireAdmin,
    State(state): State<AppState>,
    ApiJson(body): ApiJson<RegisterControllerRequest>,
) -> Result<(StatusCode, Json<ControllerKeyResponse>), ApiError> {
    let issued = state.controllers.register(&body.controller_id).await?;
    state
        .audit
        .record(
            admin.admin_id,
            AuditAction::ControllerRegistered,
            &subject("controller", issued.controller.id()),
        )
        .await;
    Ok((StatusCode::CREATED, Json(issued.into())))
}

async fn rotate_key(
    RequireAdmin(admin): RequireAdmin,
    State(state): State<AppState>,
    ApiPath(id): ApiPath<String>,
) -> Result<Json<ControllerKeyResponse>, ApiError> {
    let issued = state.controllers.rotate_key(&id).await?;
    state
        .audit
        .record(
            admin.admin_id,
            AuditAction::ControllerKeyRotated,
            &subject("controller", issued.controller.id()),
        )
        .await;
    Ok(Json(issued.into()))
}
