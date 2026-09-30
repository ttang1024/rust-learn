//! The access decision use case: load facts, decide, record.

use std::sync::Arc;

use super::{AccessDataSource, AccessEventRepository, ApplicationError, Clock, EventPublisher};
use crate::domain::{
    self, AccessCard, AccessEvent, AccessFacts, AccessGroupId, AccessPermission, AccessSchedule,
    CardNumber, Door, DoorId, User,
};

/// A simulated controller asking "may this card open this door?".
///
/// There is intentionally no timestamp field: the decision time always comes
/// from the server's clock. A device-supplied time could be forged to land
/// inside a schedule window.
#[derive(Debug, Clone)]
pub struct AccessRequest {
    pub card_number: String,
    pub door_id: DoorId,
}

/// Owned copies of the loaded facts for one request.
///
/// The engine works on borrowed [`AccessFacts`]; this struct owns the data
/// those borrows point into. [`AccessSnapshot::facts`] lends it out, and the
/// borrow checker guarantees the snapshot lives at least as long as the facts.
#[derive(Debug, Clone, Default)]
pub struct AccessSnapshot {
    pub card: Option<AccessCard>,
    pub holder: Option<User>,
    pub door: Option<Door>,
    pub holder_groups: Vec<AccessGroupId>,
    pub permissions: Vec<AccessPermission>,
    pub schedules: Vec<AccessSchedule>,
}

impl AccessSnapshot {
    /// Combines the loaded data with the request it was loaded for.
    ///
    /// The explicit `'a` says: the result borrows from *both* `self` and
    /// `presented_card`, so it must not outlive either of them. With two
    /// input references the compiler cannot guess this on its own.
    pub fn facts<'a>(
        &'a self,
        presented_card: Option<&'a CardNumber>,
        requested_door: DoorId,
    ) -> AccessFacts<'a> {
        AccessFacts {
            presented_card,
            requested_door,
            // `Option<T>` -> `Option<&T>` without moving out of `self`.
            card: self.card.as_ref(),
            holder: self.holder.as_ref(),
            door: self.door.as_ref(),
            holder_groups: &self.holder_groups,
            permissions: &self.permissions,
            schedules: &self.schedules,
        }
    }
}

pub struct AccessDecisionService<A, E> {
    source: A,
    events: E,
    publisher: Arc<dyn EventPublisher>,
    clock: Arc<dyn Clock>,
}

impl<A: AccessDataSource, E: AccessEventRepository> AccessDecisionService<A, E> {
    pub fn new(
        source: A,
        events: E,
        publisher: Arc<dyn EventPublisher>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            source,
            events,
            publisher,
            clock,
        }
    }

    /// Decides an access request, appends it to the audit log and publishes
    /// it to live subscribers.
    ///
    /// Publishing happens only after the event is stored, so every event a
    /// dashboard sees is also in the database.
    ///
    /// Returns the recorded event, which carries the decision. Any `Err`
    /// (facts could not be loaded, or the event could not be recorded) must
    /// be treated as **denied** by the caller: access is never granted
    /// without an audit record.
    pub async fn decide(&self, request: AccessRequest) -> Result<AccessEvent, ApplicationError> {
        let now = self.clock.now();

        // A malformed number cannot match any card; skip the lookup but still
        // decide (UnknownCard) and record the attempt.
        let presented = CardNumber::parse(&request.card_number).ok();
        let snapshot = self
            .source
            .load(presented.as_ref(), request.door_id)
            .await?;

        let facts = snapshot.facts(presented.as_ref(), request.door_id);
        let decision = domain::decide(&facts, now);
        let event = AccessEvent::record(&facts, decision, now);
        self.events.append(&event).await?;
        self.publisher.publish(&event);

        metrics::counter!(
            "access_decisions_total",
            "decision" => if decision.is_granted() { "granted" } else { "denied" },
            "reason" => decision.reason().map_or("none", |reason| reason.as_str()),
        )
        .increment(1);

        // Card numbers are credentials: log IDs and the outcome, never the number.
        tracing::info!(
            event_id = %event.id(),
            door_id = %request.door_id,
            granted = decision.is_granted(),
            reason = decision.reason().map(|reason| reason.as_str()),
            "access decision"
        );
        Ok(event)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, TimeZone, Utc};

    use super::*;
    use crate::{
        application::{
            EventFilter, PageRequest,
            fakes::{
                FailingEvents, FixedClock, InMemoryEvents, RecordingPublisher, StaticAccessData,
            },
        },
        domain::{
            AccessDecision, ControllerId, DenialReason, DoorName, Email, Location, Timestamp,
            UserName,
        },
    };

    fn now() -> Timestamp {
        Utc.with_ymd_and_hms(2026, 9, 30, 10, 0, 0).unwrap()
    }

    fn publisher() -> Arc<dyn EventPublisher> {
        Arc::new(RecordingPublisher::default())
    }

    /// Facts under which access is granted.
    fn granted_snapshot() -> AccessSnapshot {
        let earlier = now() - Duration::days(1);
        let user = User::register(
            UserName::parse("Alice").unwrap(),
            Email::parse("alice@example.com").unwrap(),
            earlier,
        );
        let card =
            AccessCard::issue(&user, CardNumber::parse("CARD-1").unwrap(), earlier, None).unwrap();
        let mut door = Door::create(
            DoorName::parse("Main").unwrap(),
            Location::parse("A").unwrap(),
            ControllerId::parse("c1").unwrap(),
            earlier,
        );
        door.mark_online().unwrap();
        let group = AccessGroupId::generate();
        let permission = AccessPermission::grant(group, door.id(), None, earlier);
        AccessSnapshot {
            card: Some(card),
            holder: Some(user),
            door: Some(door),
            holder_groups: vec![group],
            permissions: vec![permission],
            schedules: vec![],
        }
    }

    fn request(snapshot: &AccessSnapshot, card_number: &str) -> AccessRequest {
        AccessRequest {
            card_number: card_number.into(),
            door_id: snapshot.door.as_ref().unwrap().id(),
        }
    }

    #[tokio::test]
    async fn grants_and_records_the_event() {
        let snapshot = granted_snapshot();
        let events = InMemoryEvents::default();
        let published = RecordingPublisher::default();
        let service = AccessDecisionService::new(
            StaticAccessData::new(snapshot.clone()),
            events.clone(),
            Arc::new(published.clone()),
            FixedClock::at(now()),
        );

        let event = service.decide(request(&snapshot, "card-1")).await.unwrap();
        // Published exactly once, and only the stored event.
        assert_eq!(published.events(), vec![event.clone()]);

        assert_eq!(event.decision(), AccessDecision::Granted);
        assert_eq!(event.occurred_at(), now());
        assert_eq!(event.card_id(), snapshot.card.as_ref().map(AccessCard::id));
        assert_eq!(
            events
                .list(&EventFilter::default(), PageRequest::default())
                .await
                .unwrap(),
            vec![event]
        );
    }

    #[tokio::test]
    async fn denials_are_recorded_too() {
        let mut snapshot = granted_snapshot();
        snapshot.permissions.clear();
        let events = InMemoryEvents::default();
        let service = AccessDecisionService::new(
            StaticAccessData::new(snapshot.clone()),
            events.clone(),
            publisher(),
            FixedClock::at(now()),
        );

        let event = service.decide(request(&snapshot, "CARD-1")).await.unwrap();

        assert_eq!(
            event.decision(),
            AccessDecision::Denied(DenialReason::PermissionDenied)
        );
        assert_eq!(
            events
                .list(&EventFilter::default(), PageRequest::default())
                .await
                .unwrap()
                .len(),
            1
        );
    }

    /// Regression: the fake returns a full snapshot even without a card
    /// number, like a buggy loader would. This once granted access.
    #[tokio::test]
    async fn malformed_card_number_is_an_unknown_card() {
        let snapshot = granted_snapshot();
        let source = StaticAccessData::new(snapshot.clone());
        let service = AccessDecisionService::new(
            source.clone(),
            InMemoryEvents::default(),
            publisher(),
            FixedClock::at(now()),
        );

        let event = service
            .decide(request(&snapshot, "not a card!"))
            .await
            .unwrap();

        assert_eq!(
            event.decision(),
            AccessDecision::Denied(DenialReason::UnknownCard)
        );
        assert_eq!(event.card_number(), None);
        // The data source was asked for the door only, without a card number.
        assert_eq!(source.last_card_number(), Some(None));
    }

    #[tokio::test]
    async fn failure_to_record_is_an_error_not_a_grant() {
        let snapshot = granted_snapshot();
        let published = RecordingPublisher::default();
        let service = AccessDecisionService::new(
            StaticAccessData::new(snapshot.clone()),
            FailingEvents,
            Arc::new(published.clone()),
            FixedClock::at(now()),
        );

        let err = service
            .decide(request(&snapshot, "CARD-1"))
            .await
            .unwrap_err();

        assert!(matches!(err, ApplicationError::Internal(_)), "{err:?}");
        // An event that was not stored is never shown to dashboards.
        assert!(published.events().is_empty());
    }

    #[tokio::test]
    async fn uses_the_server_clock() {
        let snapshot = granted_snapshot();
        let clock = FixedClock::at(now());
        let service = AccessDecisionService::new(
            StaticAccessData::new(snapshot.clone()),
            InMemoryEvents::default(),
            publisher(),
            clock.clone(),
        );

        clock.advance(Duration::minutes(3));
        let event = service.decide(request(&snapshot, "CARD-1")).await.unwrap();

        assert_eq!(event.occurred_at(), now() + Duration::minutes(3));
    }
}
