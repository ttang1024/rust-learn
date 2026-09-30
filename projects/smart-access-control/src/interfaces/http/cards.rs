//! `/cards`: simulated access credentials.

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    ApiError, ApiJson, ApiPath, ApiQuery, AppState, AuthenticatedAdmin, Page, RequireAdmin,
    page_request, subject,
};
use crate::{
    application::IssueCard,
    domain::{AccessCard, AuditAction, CardId, DomainError, Timestamp, UserId},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/cards", get(list).post(issue))
        .route("/cards/{id}", get(get_one).patch(update))
        .route("/cards/{id}/revoke", post(revoke))
}

#[derive(Debug, Serialize)]
pub struct CardResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub card_number: String,
    /// As stored.
    pub status: &'static str,
    /// As of now: an active card past its expiry reports `expired`.
    pub effective_status: &'static str,
    pub issued_at: Timestamp,
    pub expires_at: Option<Timestamp>,
}

impl CardResponse {
    fn new(card: &AccessCard, now: Timestamp) -> Self {
        Self {
            id: card.id().as_uuid(),
            user_id: card.user_id().as_uuid(),
            card_number: card.card_number().to_string(),
            status: card.status().as_str(),
            effective_status: card.effective_status_at(now).as_str(),
            issued_at: card.issued_at(),
            expires_at: card.expires_at(),
        }
    }
}

/// `?user_id=` lists one user's cards (unpaginated); otherwise all cards.
/// Pagination fields are spelled out rather than `#[serde(flatten)]`ed,
/// because flatten breaks number parsing in query strings.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CardsQuery {
    pub user_id: Option<Uuid>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IssueCardRequest {
    pub user_id: Uuid,
    pub card_number: String,
    /// RFC 3339; omit for a card that does not expire.
    pub expires_at: Option<Timestamp>,
}

/// `status`: `active` or `suspended`. Revoking has its own endpoint because
/// it cannot be undone.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateCardRequest {
    pub status: String,
}

async fn list(
    _: AuthenticatedAdmin,
    State(state): State<AppState>,
    ApiQuery(query): ApiQuery<CardsQuery>,
) -> Result<Json<Page<CardResponse>>, ApiError> {
    let page = page_request(query.limit, query.offset);
    let cards = match query.user_id {
        Some(user_id) => {
            state
                .cards
                .list_for_user(UserId::from_uuid(user_id))
                .await?
        }
        None => state.cards.list(page).await?,
    };
    let now = state.clock.now();
    let items = cards
        .iter()
        .map(|card| CardResponse::new(card, now))
        .collect();
    Ok(Json(Page::new(items, page)))
}

async fn get_one(
    _: AuthenticatedAdmin,
    State(state): State<AppState>,
    ApiPath(id): ApiPath<Uuid>,
) -> Result<Json<CardResponse>, ApiError> {
    let card = state.cards.get(CardId::from_uuid(id)).await?;
    Ok(Json(CardResponse::new(&card, state.clock.now())))
}

async fn issue(
    RequireAdmin(admin): RequireAdmin,
    State(state): State<AppState>,
    ApiJson(body): ApiJson<IssueCardRequest>,
) -> Result<(StatusCode, Json<CardResponse>), ApiError> {
    let card = state
        .cards
        .issue(IssueCard {
            user_id: UserId::from_uuid(body.user_id),
            card_number: body.card_number,
            expires_at: body.expires_at,
        })
        .await?;
    state
        .audit
        .record(
            admin.admin_id,
            AuditAction::CardIssued,
            &subject("card", card.id()),
        )
        .await;
    Ok((
        StatusCode::CREATED,
        Json(CardResponse::new(&card, state.clock.now())),
    ))
}

async fn update(
    RequireAdmin(admin): RequireAdmin,
    State(state): State<AppState>,
    ApiPath(id): ApiPath<Uuid>,
    ApiJson(body): ApiJson<UpdateCardRequest>,
) -> Result<Json<CardResponse>, ApiError> {
    let id = CardId::from_uuid(id);
    let (card, action) = match body.status.as_str() {
        "active" => (
            state.cards.reactivate(id).await?,
            AuditAction::CardReactivated,
        ),
        "suspended" => (state.cards.suspend(id).await?, AuditAction::CardSuspended),
        _ => {
            return Err(DomainError::Validation {
                field: "status",
                reason: "must be 'active' or 'suspended' (use POST /cards/{id}/revoke to revoke)",
            }
            .into());
        }
    };
    state
        .audit
        .record(admin.admin_id, action, &subject("card", id))
        .await;
    Ok(Json(CardResponse::new(&card, state.clock.now())))
}

async fn revoke(
    RequireAdmin(admin): RequireAdmin,
    State(state): State<AppState>,
    ApiPath(id): ApiPath<Uuid>,
) -> Result<Json<CardResponse>, ApiError> {
    let card = state.cards.revoke(CardId::from_uuid(id)).await?;
    state
        .audit
        .record(
            admin.admin_id,
            AuditAction::CardRevoked,
            &subject("card", card.id()),
        )
        .await;
    Ok(Json(CardResponse::new(&card, state.clock.now())))
}
