//! `/events` (access attempts), `/access/decisions` (simulated requests)
//! and `/audit-log` (administrative actions).

use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    ApiError, ApiJson, ApiPath, ApiQuery, AppState, AuthenticatedAdmin, Page, PageParams,
    RequireAdmin, page_request,
};
use crate::{
    application::{AccessRequest, DecisionFilter, EventFilter, PageRequest},
    domain::{
        AccessEvent, AuditEntry, CardId, DenialReason, DomainError, DoorId, EventId, Timestamp,
        UserId,
    },
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/events", get(list))
        .route("/events/recent", get(recent))
        .route("/events/{id}", get(get_one))
        .route("/access/decisions", post(decide))
        .route("/audit-log", get(audit_log))
}

#[derive(Debug, Serialize)]
pub struct EventResponse {
    pub id: Uuid,
    /// `granted` or `denied`.
    pub decision: &'static str,
    /// Denial reason, `null` when granted.
    pub reason: Option<&'static str>,
    pub card_number: Option<String>,
    pub card_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub door_id: Option<Uuid>,
    pub occurred_at: Timestamp,
}

impl From<&AccessEvent> for EventResponse {
    fn from(event: &AccessEvent) -> Self {
        let decision = event.decision();
        Self {
            id: event.id().as_uuid(),
            decision: if decision.is_granted() {
                "granted"
            } else {
                "denied"
            },
            reason: decision.reason().map(|reason| reason.as_str()),
            card_number: event.card_number().map(ToString::to_string),
            card_id: event.card_id().map(|id| id.as_uuid()),
            user_id: event.user_id().map(|id| id.as_uuid()),
            door_id: event.door_id().map(|id| id.as_uuid()),
            occurred_at: event.occurred_at(),
        }
    }
}

/// Event filter fields, shared by `GET /events` and WebSocket subscriptions.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FilterParams {
    pub door_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub card_id: Option<Uuid>,
    /// `granted` or `denied`.
    pub decision: Option<String>,
    /// A denial reason such as `door_offline`.
    pub reason: Option<String>,
    /// RFC 3339, inclusive.
    pub from: Option<Timestamp>,
    /// RFC 3339, exclusive.
    pub until: Option<Timestamp>,
}

impl FilterParams {
    pub fn into_filter(self) -> Result<EventFilter, DomainError> {
        let decision = match self.decision.as_deref() {
            None => None,
            Some("granted") => Some(DecisionFilter::Granted),
            Some("denied") => Some(DecisionFilter::Denied),
            Some(_) => {
                return Err(DomainError::Validation {
                    field: "decision",
                    reason: "must be 'granted' or 'denied'",
                });
            }
        };
        let filter = EventFilter {
            door_id: self.door_id.map(DoorId::from_uuid),
            user_id: self.user_id.map(UserId::from_uuid),
            card_id: self.card_id.map(CardId::from_uuid),
            decision,
            reason: self
                .reason
                .as_deref()
                .map(str::parse::<DenialReason>)
                .transpose()?,
            from: self.from,
            until: self.until,
        };
        filter.validate()?;
        Ok(filter)
    }
}

/// `GET /events` query: filter fields plus pagination. Spelled out rather
/// than `#[serde(flatten)]`ed, which breaks number parsing in query strings.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventsQuery {
    pub door_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub card_id: Option<Uuid>,
    pub decision: Option<String>,
    pub reason: Option<String>,
    pub from: Option<Timestamp>,
    pub until: Option<Timestamp>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecentQuery {
    /// Default 20, at most 100.
    pub limit: Option<u32>,
}

/// A simulated controller's request. No timestamp: the server's clock decides.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionRequest {
    pub card_number: String,
    pub door_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct AuditEntryResponse {
    pub id: Uuid,
    pub administrator_id: Option<Uuid>,
    pub action: &'static str,
    pub subject: Option<String>,
    pub occurred_at: Timestamp,
}

impl From<&AuditEntry> for AuditEntryResponse {
    fn from(entry: &AuditEntry) -> Self {
        Self {
            id: entry.id().as_uuid(),
            administrator_id: entry.actor().map(|id| id.as_uuid()),
            action: entry.action().as_str(),
            subject: entry.subject().map(ToString::to_string),
            occurred_at: entry.occurred_at(),
        }
    }
}

/// Matching events, newest first.
async fn list(
    _: AuthenticatedAdmin,
    State(state): State<AppState>,
    ApiQuery(query): ApiQuery<EventsQuery>,
) -> Result<Json<Page<EventResponse>>, ApiError> {
    let page = page_request(query.limit, query.offset);
    let filter = FilterParams {
        door_id: query.door_id,
        user_id: query.user_id,
        card_id: query.card_id,
        decision: query.decision,
        reason: query.reason,
        from: query.from,
        until: query.until,
    }
    .into_filter()?;
    let events = state.events.list(&filter, page).await?;
    Ok(Json(Page::new(
        events.iter().map(EventResponse::from).collect(),
        page,
    )))
}

async fn recent(
    _: AuthenticatedAdmin,
    State(state): State<AppState>,
    ApiQuery(query): ApiQuery<RecentQuery>,
) -> Result<Json<Vec<EventResponse>>, ApiError> {
    let page = PageRequest::new(query.limit.unwrap_or(20), 0);
    let events = state.events.list(&EventFilter::default(), page).await?;
    Ok(Json(events.iter().map(EventResponse::from).collect()))
}

async fn get_one(
    _: AuthenticatedAdmin,
    State(state): State<AppState>,
    ApiPath(id): ApiPath<Uuid>,
) -> Result<Json<EventResponse>, ApiError> {
    let event = state.events.get(EventId::from_uuid(id)).await?;
    Ok(Json(EventResponse::from(&event)))
}

/// `POST /access/decisions`: evaluates a simulated access request and
/// records it. Both outcomes are `200 OK`; the decision is in the body.
///
/// Requires an admin token for now; simulated door controllers get their
/// own credentials in Phase 6.
async fn decide(
    _: RequireAdmin,
    State(state): State<AppState>,
    ApiJson(body): ApiJson<DecisionRequest>,
) -> Result<Json<EventResponse>, ApiError> {
    let event = state
        .access
        .decide(AccessRequest {
            card_number: body.card_number,
            door_id: DoorId::from_uuid(body.door_id),
        })
        .await?;
    Ok(Json(EventResponse::from(&event)))
}

async fn audit_log(
    _: AuthenticatedAdmin,
    State(state): State<AppState>,
    ApiQuery(params): ApiQuery<PageParams>,
) -> Result<Json<Page<AuditEntryResponse>>, ApiError> {
    let page = PageRequest::from(params);
    let entries = state.audit.list_recent(page).await?;
    Ok(Json(Page::new(
        entries.iter().map(AuditEntryResponse::from).collect(),
        page,
    )))
}
