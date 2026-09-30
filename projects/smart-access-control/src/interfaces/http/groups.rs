//! `/access-groups`: groups of users that share door permissions.

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
    RequireAdmin, double_option, subject,
};
use crate::{
    application::{CreateGroup, PageRequest, UpdateGroup},
    domain::{AccessGroup, AccessGroupId, AuditAction, UserId},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/access-groups", get(list).post(create))
        .route(
            "/access-groups/{id}",
            get(get_one).patch(update).delete(remove),
        )
        .route(
            "/access-groups/{id}/members",
            get(list_members).post(add_member),
        )
        .route(
            "/access-groups/{id}/members/{user_id}",
            delete(remove_member),
        )
}

#[derive(Debug, Serialize)]
pub struct GroupResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
}

impl From<&AccessGroup> for GroupResponse {
    fn from(group: &AccessGroup) -> Self {
        Self {
            id: group.id().as_uuid(),
            name: group.name().to_string(),
            description: group.description().map(ToString::to_string),
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateGroupRequest {
    pub name: String,
    pub description: Option<String>,
}

/// `"description": null` clears it; leaving it out keeps it.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateGroupRequest {
    pub name: Option<String>,
    #[serde(default, deserialize_with = "double_option")]
    pub description: Option<Option<String>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AddMemberRequest {
    pub user_id: Uuid,
}

async fn list(
    _: AuthenticatedAdmin,
    State(state): State<AppState>,
    ApiQuery(params): ApiQuery<PageParams>,
) -> Result<Json<Page<GroupResponse>>, ApiError> {
    let page = PageRequest::from(params);
    let groups = state.groups.list(page).await?;
    Ok(Json(Page::new(
        groups.iter().map(GroupResponse::from).collect(),
        page,
    )))
}

async fn get_one(
    _: AuthenticatedAdmin,
    State(state): State<AppState>,
    ApiPath(id): ApiPath<Uuid>,
) -> Result<Json<GroupResponse>, ApiError> {
    let group = state.groups.get(AccessGroupId::from_uuid(id)).await?;
    Ok(Json(GroupResponse::from(&group)))
}

async fn create(
    RequireAdmin(admin): RequireAdmin,
    State(state): State<AppState>,
    ApiJson(body): ApiJson<CreateGroupRequest>,
) -> Result<(StatusCode, Json<GroupResponse>), ApiError> {
    let group = state
        .groups
        .create(CreateGroup {
            name: body.name,
            description: body.description,
        })
        .await?;
    state
        .audit
        .record(
            admin.admin_id,
            AuditAction::AccessGroupCreated,
            &subject("access_group", group.id()),
        )
        .await;
    Ok((StatusCode::CREATED, Json(GroupResponse::from(&group))))
}

async fn update(
    RequireAdmin(admin): RequireAdmin,
    State(state): State<AppState>,
    ApiPath(id): ApiPath<Uuid>,
    ApiJson(body): ApiJson<UpdateGroupRequest>,
) -> Result<Json<GroupResponse>, ApiError> {
    let group = state
        .groups
        .update(
            AccessGroupId::from_uuid(id),
            UpdateGroup {
                name: body.name,
                description: body.description,
            },
        )
        .await?;
    state
        .audit
        .record(
            admin.admin_id,
            AuditAction::AccessGroupUpdated,
            &subject("access_group", group.id()),
        )
        .await;
    Ok(Json(GroupResponse::from(&group)))
}

/// Deleting a group also removes its memberships and permissions.
async fn remove(
    RequireAdmin(admin): RequireAdmin,
    State(state): State<AppState>,
    ApiPath(id): ApiPath<Uuid>,
) -> Result<StatusCode, ApiError> {
    let id = AccessGroupId::from_uuid(id);
    state.groups.delete(id).await?;
    state
        .audit
        .record(
            admin.admin_id,
            AuditAction::AccessGroupDeleted,
            &subject("access_group", id),
        )
        .await;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Serialize)]
pub struct MembersResponse {
    pub user_ids: Vec<Uuid>,
}

async fn list_members(
    _: AuthenticatedAdmin,
    State(state): State<AppState>,
    ApiPath(id): ApiPath<Uuid>,
) -> Result<Json<MembersResponse>, ApiError> {
    let members = state.groups.members(AccessGroupId::from_uuid(id)).await?;
    Ok(Json(MembersResponse {
        user_ids: members.iter().map(|id| id.as_uuid()).collect(),
    }))
}

async fn add_member(
    RequireAdmin(admin): RequireAdmin,
    State(state): State<AppState>,
    ApiPath(id): ApiPath<Uuid>,
    ApiJson(body): ApiJson<AddMemberRequest>,
) -> Result<StatusCode, ApiError> {
    let (group_id, user_id) = (
        AccessGroupId::from_uuid(id),
        UserId::from_uuid(body.user_id),
    );
    state.groups.add_member(group_id, user_id).await?;
    let what = format!(
        "{} + {}",
        subject("access_group", group_id),
        subject("user", user_id)
    );
    state
        .audit
        .record(admin.admin_id, AuditAction::GroupMemberAdded, &what)
        .await;
    Ok(StatusCode::NO_CONTENT)
}

async fn remove_member(
    RequireAdmin(admin): RequireAdmin,
    State(state): State<AppState>,
    ApiPath((id, user_id)): ApiPath<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    let (group_id, user_id) = (AccessGroupId::from_uuid(id), UserId::from_uuid(user_id));
    state.groups.remove_member(group_id, user_id).await?;
    let what = format!(
        "{} - {}",
        subject("access_group", group_id),
        subject("user", user_id)
    );
    state
        .audit
        .record(admin.admin_id, AuditAction::GroupMemberRemoved, &what)
        .await;
    Ok(StatusCode::NO_CONTENT)
}
