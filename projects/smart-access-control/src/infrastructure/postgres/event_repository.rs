use sqlx::PgPool;
use uuid::Uuid;

use super::{into_entities, into_entity, map_error};
use crate::{
    application::{
        AccessEventRepository, DecisionFilter, EventFilter, PageRequest, RepositoryResult,
    },
    domain::{
        AccessDecision, AccessEvent, CardId, CardNumber, DomainError, DoorId, EventId, Timestamp,
        UserId,
    },
};

#[derive(Debug, Clone)]
pub struct PgAccessEventRepository {
    pool: PgPool,
}

impl PgAccessEventRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct EventRow {
    id: Uuid,
    card_number: Option<String>,
    card_id: Option<Uuid>,
    user_id: Option<Uuid>,
    door_id: Option<Uuid>,
    decision: String,
    reason: Option<String>,
    occurred_at: Timestamp,
}

/// Storage shape of a decision: `('granted', NULL)` or `('denied', reason)`.
fn decision_columns(decision: AccessDecision) -> (&'static str, Option<&'static str>) {
    match decision {
        AccessDecision::Granted => ("granted", None),
        AccessDecision::Denied(reason) => ("denied", Some(reason.as_str())),
    }
}

impl TryFrom<EventRow> for AccessEvent {
    type Error = DomainError;

    fn try_from(row: EventRow) -> Result<Self, Self::Error> {
        let decision = match (row.decision.as_str(), row.reason.as_deref()) {
            ("granted", None) => AccessDecision::Granted,
            ("denied", Some(reason)) => AccessDecision::Denied(reason.parse()?),
            _ => {
                return Err(DomainError::Validation {
                    field: "decision",
                    reason: "decision and reason are inconsistent",
                });
            }
        };
        Ok(AccessEvent::restore(
            EventId::from_uuid(row.id),
            row.card_number
                .as_deref()
                .map(CardNumber::parse)
                .transpose()?,
            row.card_id.map(CardId::from_uuid),
            row.user_id.map(UserId::from_uuid),
            row.door_id.map(DoorId::from_uuid),
            decision,
            row.occurred_at,
        ))
    }
}

impl AccessEventRepository for PgAccessEventRepository {
    async fn append(&self, event: &AccessEvent) -> RepositoryResult<()> {
        let (decision, reason) = decision_columns(event.decision());
        sqlx::query(
            "INSERT INTO access_events
                 (id, card_number, card_id, user_id, door_id, decision, reason, occurred_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(event.id().as_uuid())
        .bind(event.card_number().map(CardNumber::as_str))
        .bind(event.card_id().map(|id| id.as_uuid()))
        .bind(event.user_id().map(|id| id.as_uuid()))
        .bind(event.door_id().map(|id| id.as_uuid()))
        .bind(decision)
        .bind(reason)
        .bind(event.occurred_at())
        .execute(&self.pool)
        .await
        .map_err(map_error)?;
        Ok(())
    }

    async fn find_by_id(&self, id: EventId) -> RepositoryResult<Option<AccessEvent>> {
        let row = sqlx::query_as::<_, EventRow>(
            "SELECT id, card_number, card_id, user_id, door_id, decision, reason, occurred_at
             FROM access_events WHERE id = $1",
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_error)?;
        into_entity(row)
    }

    /// One fixed, fully parameterized statement for every filter
    /// combination: `$n IS NULL OR column = $n` turns an unset filter into
    /// "match everything". Filter values never become part of the SQL text.
    ///
    /// Trade-off: a generic plan may not use the best index for every
    /// combination. Fine at this scale; `sqlx::QueryBuilder` could build a
    /// tailored (still parameterized) statement if it becomes a bottleneck.
    async fn list(
        &self,
        filter: &EventFilter,
        page: PageRequest,
    ) -> RepositoryResult<Vec<AccessEvent>> {
        let decision = filter.decision.map(|decision| match decision {
            DecisionFilter::Granted => "granted",
            DecisionFilter::Denied => "denied",
        });
        let rows = sqlx::query_as::<_, EventRow>(
            "SELECT id, card_number, card_id, user_id, door_id, decision, reason, occurred_at
             FROM access_events
             WHERE ($1::uuid IS NULL OR door_id = $1)
               AND ($2::uuid IS NULL OR user_id = $2)
               AND ($3::uuid IS NULL OR card_id = $3)
               AND ($4::text IS NULL OR decision = $4)
               AND ($5::text IS NULL OR reason = $5)
               AND ($6::timestamptz IS NULL OR occurred_at >= $6)
               AND ($7::timestamptz IS NULL OR occurred_at < $7)
             ORDER BY occurred_at DESC, id DESC
             LIMIT $8 OFFSET $9",
        )
        .bind(filter.door_id.map(|id| id.as_uuid()))
        .bind(filter.user_id.map(|id| id.as_uuid()))
        .bind(filter.card_id.map(|id| id.as_uuid()))
        .bind(decision)
        .bind(filter.reason.map(|reason| reason.as_str()))
        .bind(filter.from)
        .bind(filter.until)
        .bind(i64::from(page.limit()))
        .bind(i64::from(page.offset()))
        .fetch_all(&self.pool)
        .await
        .map_err(map_error)?;
        into_entities(rows)
    }
}
