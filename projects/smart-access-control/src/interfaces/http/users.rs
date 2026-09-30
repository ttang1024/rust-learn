//! `/users`: people who hold access cards.

use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    ApiError, ApiJson, ApiPath, ApiQuery, AppState, AuthenticatedAdmin, Page, PageParams,
    RequireAdmin, subject,
};
use crate::{
    application::{ApplicationError, PageRequest, RegisterUser, UpdateUser},
    domain::{AuditAction, Timestamp, User, UserId, UserStatus},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/users", get(list).post(create))
        .route("/users/{id}", get(get_one).patch(update).delete(archive))
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub status: &'static str,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl From<&User> for UserResponse {
    fn from(user: &User) -> Self {
        Self {
            id: user.id().as_uuid(),
            name: user.name().to_string(),
            email: user.email().to_string(),
            status: user.status().as_str(),
            created_at: user.created_at(),
            updated_at: user.updated_at(),
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
}

/// All fields optional. `status`: `active`, `suspended` or `archived`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateUserRequest {
    pub name: Option<String>,
    pub email: Option<String>,
    pub status: Option<String>,
}

async fn list(
    _: AuthenticatedAdmin,
    State(state): State<AppState>,
    ApiQuery(params): ApiQuery<PageParams>,
) -> Result<Json<Page<UserResponse>>, ApiError> {
    let page = PageRequest::from(params);
    let users = state.users.list(page).await?;
    Ok(Json(Page::of(&users, page)))
}

async fn get_one(
    _: AuthenticatedAdmin,
    State(state): State<AppState>,
    ApiPath(id): ApiPath<Uuid>,
) -> Result<Json<UserResponse>, ApiError> {
    let user = state.users.get(UserId::from_uuid(id)).await?;
    Ok(Json(UserResponse::from(&user)))
}

async fn create(
    RequireAdmin(admin): RequireAdmin,
    State(state): State<AppState>,
    ApiJson(body): ApiJson<CreateUserRequest>,
) -> Result<(StatusCode, Json<UserResponse>), ApiError> {
    let user = state
        .users
        .register(RegisterUser {
            name: body.name,
            email: body.email,
        })
        .await?;
    state
        .record_audit(
            admin,
            AuditAction::UserRegistered,
            &subject("user", user.id()),
        )
        .await;
    Ok((StatusCode::CREATED, Json(UserResponse::from(&user))))
}

/// Applies profile changes, then a status change. The status is validated
/// before anything is modified.
async fn update(
    RequireAdmin(admin): RequireAdmin,
    State(state): State<AppState>,
    ApiPath(id): ApiPath<Uuid>,
    ApiJson(body): ApiJson<UpdateUserRequest>,
) -> Result<Json<UserResponse>, ApiError> {
    let id = UserId::from_uuid(id);
    let status = body
        .status
        .as_deref()
        .map(str::parse::<UserStatus>)
        .transpose()
        .map_err(ApplicationError::from)?;
    let who = subject("user", id);

    let mut user = if body.name.is_some() || body.email.is_some() {
        let user = state
            .users
            .update(
                id,
                UpdateUser {
                    name: body.name,
                    email: body.email,
                },
            )
            .await?;
        state
            .record_audit(admin, AuditAction::UserUpdated, &who)
            .await;
        user
    } else {
        state.users.get(id).await?
    };

    // Setting the current status again is a no-op, which keeps PATCH idempotent.
    if let Some(status) = status
        && status != user.status()
    {
        let action = match status {
            UserStatus::Active => {
                user = state.users.reactivate(id).await?;
                AuditAction::UserReactivated
            }
            UserStatus::Suspended => {
                user = state.users.suspend(id).await?;
                AuditAction::UserSuspended
            }
            UserStatus::Archived => {
                user = state.users.archive(id).await?;
                AuditAction::UserArchived
            }
        };
        state.record_audit(admin, action, &who).await;
    }
    Ok(Json(UserResponse::from(&user)))
}

/// `DELETE` archives (soft-deletes): access history must stay intact.
async fn archive(
    RequireAdmin(admin): RequireAdmin,
    State(state): State<AppState>,
    ApiPath(id): ApiPath<Uuid>,
) -> Result<StatusCode, ApiError> {
    let user = state.users.archive(UserId::from_uuid(id)).await?;
    state
        .record_audit(
            admin,
            AuditAction::UserArchived,
            &subject("user", user.id()),
        )
        .await;
    Ok(StatusCode::NO_CONTENT)
}
