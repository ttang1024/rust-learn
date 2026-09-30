//! Reading, filtering and publishing access events.

use super::{AccessEventRepository, ApplicationError, PageRequest};
use crate::domain::{
    AccessDecision, AccessEvent, CardId, DenialReason, DomainError, DoorId, EventId, Timestamp,
    UserId,
};

/// Pushes newly recorded events to live subscribers (e.g. dashboards).
///
/// Synchronous and non-blocking by contract: publishing must never delay an
/// access decision, however slow or numerous the subscribers are. Delivery
/// is best-effort; the event store stays the source of truth.
pub trait EventPublisher: Send + Sync {
    fn publish(&self, event: &AccessEvent);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecisionFilter {
    Granted,
    Denied,
}

/// Which events to return. Every `None` field matches everything; set fields
/// must all match (logical AND).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EventFilter {
    pub door_id: Option<DoorId>,
    pub user_id: Option<UserId>,
    pub card_id: Option<CardId>,
    pub decision: Option<DecisionFilter>,
    pub reason: Option<DenialReason>,
    /// Inclusive lower bound.
    pub from: Option<Timestamp>,
    /// Exclusive upper bound.
    pub until: Option<Timestamp>,
}

impl EventFilter {
    pub fn validate(&self) -> Result<(), DomainError> {
        if let (Some(from), Some(until)) = (self.from, self.until)
            && until <= from
        {
            return Err(DomainError::Validation {
                field: "until",
                reason: "must be after from",
            });
        }
        if self.reason.is_some() && self.decision == Some(DecisionFilter::Granted) {
            return Err(DomainError::Validation {
                field: "reason",
                reason: "only denied events have a reason",
            });
        }
        Ok(())
    }

    /// The in-memory version of the filter, used for live subscriptions.
    /// Must agree with the SQL in the PostgreSQL repository (both are tested
    /// against the same cases).
    pub fn matches(&self, event: &AccessEvent) -> bool {
        let decision = event.decision();
        let decision_ok = match self.decision {
            None => true,
            Some(DecisionFilter::Granted) => decision == AccessDecision::Granted,
            Some(DecisionFilter::Denied) => !decision.is_granted(),
        };
        decision_ok
            && self.door_id.is_none_or(|id| event.door_id() == Some(id))
            && self.user_id.is_none_or(|id| event.user_id() == Some(id))
            && self.card_id.is_none_or(|id| event.card_id() == Some(id))
            && self
                .reason
                .is_none_or(|reason| decision.reason() == Some(reason))
            && self.from.is_none_or(|from| event.occurred_at() >= from)
            && self.until.is_none_or(|until| event.occurred_at() < until)
    }
}

pub struct AccessEventService<E> {
    events: E,
}

impl<E: AccessEventRepository> AccessEventService<E> {
    pub fn new(events: E) -> Self {
        Self { events }
    }

    pub async fn get(&self, id: EventId) -> Result<AccessEvent, ApplicationError> {
        self.events
            .find_by_id(id)
            .await?
            .ok_or(ApplicationError::NotFound { entity: "event" })
    }

    /// Matching events, newest first.
    pub async fn list(
        &self,
        filter: &EventFilter,
        page: PageRequest,
    ) -> Result<Vec<AccessEvent>, ApplicationError> {
        filter.validate()?;
        Ok(self.events.list(filter, page).await?)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use chrono::{Duration, TimeZone, Utc};

    use super::*;
    use crate::domain::{AccessFacts, CardNumber};

    pub(crate) fn at(minute: i64) -> Timestamp {
        Utc.with_ymd_and_hms(2026, 9, 30, 10, 0, 0).unwrap() + Duration::minutes(minute)
    }

    /// An event at `door` for an unknown card (no card or user references).
    pub(crate) fn event(door: DoorId, decision: AccessDecision, minute: i64) -> AccessEvent {
        let number = CardNumber::parse("CARD-1").unwrap();
        let facts = AccessFacts {
            presented_card: Some(&number),
            requested_door: door,
            card: None,
            holder: None,
            door: None,
            holder_groups: &[],
            permissions: &[],
            schedules: &[],
        };
        AccessEvent::record(&facts, decision, at(minute))
    }

    #[test]
    fn empty_filter_matches_everything() {
        let e = event(DoorId::generate(), AccessDecision::Granted, 0);
        assert!(EventFilter::default().matches(&e));
    }

    #[test]
    fn decision_and_reason_filters() {
        let door = DoorId::generate();
        let granted = event(door, AccessDecision::Granted, 0);
        let offline = event(door, AccessDecision::Denied(DenialReason::DoorOffline), 0);

        let denied = EventFilter {
            decision: Some(DecisionFilter::Denied),
            ..EventFilter::default()
        };
        assert!(!denied.matches(&granted));
        assert!(denied.matches(&offline));

        let by_reason = EventFilter {
            reason: Some(DenialReason::UnknownCard),
            ..EventFilter::default()
        };
        assert!(!by_reason.matches(&offline));
    }

    #[test]
    fn time_range_is_half_open() {
        let e = event(DoorId::generate(), AccessDecision::Granted, 5);
        let range = |from, until| EventFilter {
            from: Some(at(from)),
            until: Some(at(until)),
            ..EventFilter::default()
        };
        assert!(range(5, 6).matches(&e), "from is inclusive");
        assert!(!range(4, 5).matches(&e), "until is exclusive");
    }

    #[test]
    fn id_filters_never_match_missing_references() {
        // The event references no user; a user filter must not match it.
        let e = event(DoorId::generate(), AccessDecision::Granted, 0);
        let filter = EventFilter {
            user_id: Some(UserId::generate()),
            ..EventFilter::default()
        };
        assert!(!filter.matches(&e));
    }

    #[test]
    fn validation() {
        let backwards = EventFilter {
            from: Some(at(5)),
            until: Some(at(5)),
            ..EventFilter::default()
        };
        assert!(backwards.validate().is_err());

        let contradictory = EventFilter {
            decision: Some(DecisionFilter::Granted),
            reason: Some(DenialReason::DoorOffline),
            ..EventFilter::default()
        };
        assert!(contradictory.validate().is_err());
        assert!(EventFilter::default().validate().is_ok());
    }
}
