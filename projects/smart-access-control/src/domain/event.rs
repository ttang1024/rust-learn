//! The append-only audit record of one access attempt.

use super::{AccessDecision, AccessFacts, CardId, CardNumber, DoorId, EventId, Timestamp, UserId};

/// One access attempt and its outcome. There are no mutating methods:
/// events are append-only history.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessEvent {
    id: EventId,
    /// The number that was presented, if it was well-formed. Kept even for
    /// unknown cards, which is exactly when an audit trail matters most.
    card_number: Option<CardNumber>,
    card_id: Option<CardId>,
    user_id: Option<UserId>,
    door_id: Option<DoorId>,
    decision: AccessDecision,
    occurred_at: Timestamp,
}

impl AccessEvent {
    /// Builds the record for a decision. It references only entities that
    /// exist *and* match the request (the same checks the engine uses), so
    /// it never points at a phantom or unrelated row.
    pub fn record(
        facts: &AccessFacts<'_>,
        decision: AccessDecision,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            id: EventId::generate(),
            card_number: facts.presented_card.cloned(),
            card_id: facts.matching_card().map(|card| card.id()),
            user_id: facts.matching_holder().map(|user| user.id()),
            door_id: facts.matching_door().map(|door| door.id()),
            decision,
            occurred_at,
        }
    }

    /// Rebuilds an event from stored values. For repositories only.
    pub fn restore(
        id: EventId,
        card_number: Option<CardNumber>,
        card_id: Option<CardId>,
        user_id: Option<UserId>,
        door_id: Option<DoorId>,
        decision: AccessDecision,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            id,
            card_number,
            card_id,
            user_id,
            door_id,
            decision,
            occurred_at,
        }
    }

    pub fn id(&self) -> EventId {
        self.id
    }

    pub fn card_number(&self) -> Option<&CardNumber> {
        self.card_number.as_ref()
    }

    pub fn card_id(&self) -> Option<CardId> {
        self.card_id
    }

    pub fn user_id(&self) -> Option<UserId> {
        self.user_id
    }

    pub fn door_id(&self) -> Option<DoorId> {
        self.door_id
    }

    pub fn decision(&self) -> AccessDecision {
        self.decision
    }

    pub fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::*;
    use crate::domain::{
        AccessCard, ControllerId, DenialReason, Door, DoorName, Email, Location, User, UserName,
    };

    #[test]
    fn records_what_exists_and_what_was_presented() {
        let now = Utc.with_ymd_and_hms(2026, 9, 30, 10, 0, 0).unwrap();
        let user = User::register(
            UserName::parse("Alice").unwrap(),
            Email::parse("alice@example.com").unwrap(),
            now,
        );
        let number = CardNumber::parse("CARD-1").unwrap();
        let card = AccessCard::issue(&user, number.clone(), now, None).unwrap();
        let door = Door::create(
            DoorName::parse("Main").unwrap(),
            Location::parse("A").unwrap(),
            ControllerId::parse("c1").unwrap(),
            now,
        );
        let full = AccessFacts {
            presented_card: Some(&number),
            requested_door: door.id(),
            card: Some(&card),
            holder: Some(&user),
            door: Some(&door),
            holder_groups: &[],
            permissions: &[],
            schedules: &[],
        };

        let event = AccessEvent::record(&full, AccessDecision::Granted, now);
        assert_eq!(event.card_id(), Some(card.id()));
        assert_eq!(event.user_id(), Some(user.id()));
        assert_eq!(event.door_id(), Some(door.id()));
        assert_eq!(event.occurred_at(), now);

        // Unknown card: the presented number is kept, no phantom IDs.
        let unknown = AccessFacts {
            card: None,
            holder: None,
            ..full
        };
        let decision = AccessDecision::Denied(DenialReason::UnknownCard);
        let event = AccessEvent::record(&unknown, decision, now);
        assert_eq!(event.card_number(), Some(&number));
        assert_eq!(event.card_id(), None);
        assert_eq!(event.user_id(), None);
        assert_eq!(event.decision().reason(), Some(DenialReason::UnknownCard));

        // A loaded card or door that does not match the request is never referenced.
        let other_number = CardNumber::parse("CARD-2").unwrap();
        let mismatched = AccessFacts {
            presented_card: Some(&other_number),
            requested_door: crate::domain::DoorId::generate(),
            ..full
        };
        let event = AccessEvent::record(&mismatched, decision, now);
        assert_eq!(event.card_id(), None);
        assert_eq!(event.user_id(), None);
        assert_eq!(event.door_id(), None);
    }
}
