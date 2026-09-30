//! The access decision engine: may this card open this door right now?
//!
//! [`decide`] is a pure function. It reads a snapshot of already-loaded
//! facts and an explicit time, and performs no I/O, so the same input always
//! gives the same decision. Loading the facts is the application layer's job.
//!
//! Every missing fact or failed check denies access (deny by default).

use std::{fmt, str::FromStr};

use super::{
    AccessCard, AccessGroupId, AccessPermission, AccessSchedule, CardNumber, CardStatus,
    DomainError, Door, DoorId, DoorStatus, Timestamp, User, UserStatus,
};

/// Why access was denied. Stored in the audit log, so names are stable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DenialReason {
    UnknownCard,
    CardRevoked,
    CardSuspended,
    CardExpired,
    UserSuspended,
    UserArchived,
    UnknownDoor,
    DoorDisabled,
    DoorOffline,
    PermissionDenied,
    OutsideSchedule,
}

impl DenialReason {
    pub const ALL: [Self; 11] = [
        Self::UnknownCard,
        Self::CardRevoked,
        Self::CardSuspended,
        Self::CardExpired,
        Self::UserSuspended,
        Self::UserArchived,
        Self::UnknownDoor,
        Self::DoorDisabled,
        Self::DoorOffline,
        Self::PermissionDenied,
        Self::OutsideSchedule,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnknownCard => "unknown_card",
            Self::CardRevoked => "card_revoked",
            Self::CardSuspended => "card_suspended",
            Self::CardExpired => "card_expired",
            Self::UserSuspended => "user_suspended",
            Self::UserArchived => "user_archived",
            Self::UnknownDoor => "unknown_door",
            Self::DoorDisabled => "door_disabled",
            Self::DoorOffline => "door_offline",
            Self::PermissionDenied => "permission_denied",
            Self::OutsideSchedule => "outside_schedule",
        }
    }
}

impl FromStr for DenialReason {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|reason| reason.as_str() == s)
            .ok_or(DomainError::Validation {
                field: "reason",
                reason: "unknown denial reason",
            })
    }
}

impl fmt::Display for DenialReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AccessDecision {
    Granted,
    Denied(DenialReason),
}

impl AccessDecision {
    pub fn is_granted(self) -> bool {
        self == Self::Granted
    }

    pub fn reason(self) -> Option<DenialReason> {
        match self {
            Self::Granted => None,
            Self::Denied(reason) => Some(reason),
        }
    }
}

/// Everything the engine needs to know about one access attempt.
///
/// # Lifetimes
///
/// The struct holds references (`&'a User`, `&'a [AccessPermission]`, ...)
/// instead of owned values, so building it copies nothing: it only *borrows*
/// data that the caller owns. The lifetime parameter `'a` tells the compiler
/// "these references are valid for at least `'a`", which guarantees an
/// `AccessFacts` can never outlive the data it points to. Because every
/// reference is shared (`&`, not `&mut`), the engine also cannot modify
/// anything: purity is enforced by the type system.
///
/// # Trust
///
/// The request itself (`presented_card`, `requested_door`) comes from the
/// caller; everything else comes from a loader. The engine never assumes the
/// loader got it right: a loaded card, holder or door is only used if it
/// matches the request (see [`AccessFacts::matching_card`] and friends).
#[derive(Debug, Clone, Copy)]
pub struct AccessFacts<'a> {
    /// The card number presented at the door, or `None` if it was malformed.
    pub presented_card: Option<&'a CardNumber>,
    /// The door the request was made at.
    pub requested_door: DoorId,
    /// The card with the presented number, if one exists.
    pub card: Option<&'a AccessCard>,
    /// The card's holder, if found.
    pub holder: Option<&'a User>,
    /// The requested door, if it exists.
    pub door: Option<&'a Door>,
    /// Groups the holder belongs to.
    pub holder_groups: &'a [AccessGroupId],
    /// Candidate permissions. The engine re-checks door and group itself, so
    /// passing extra permissions can never grant access by mistake.
    pub permissions: &'a [AccessPermission],
    /// Schedules referenced by the permissions. A referenced schedule that
    /// is missing here counts as "not allowed".
    pub schedules: &'a [AccessSchedule],
}

impl<'a> AccessFacts<'a> {
    /// The loaded card, but only if it carries the presented number.
    ///
    /// Returns `&'a AccessCard`, not a reference tied to `&self`: the card
    /// lives as long as the underlying data, not as long as this struct.
    pub fn matching_card(&self) -> Option<&'a AccessCard> {
        let presented = self.presented_card?;
        self.card.filter(|card| card.card_number() == presented)
    }

    /// The loaded holder, but only if they own the matching card.
    pub fn matching_holder(&self) -> Option<&'a User> {
        let card = self.matching_card()?;
        self.holder.filter(|user| user.id() == card.user_id())
    }

    /// The loaded door, but only if it is the requested one.
    pub fn matching_door(&self) -> Option<&'a Door> {
        self.door.filter(|door| door.id() == self.requested_door)
    }
}

/// Decides one access attempt at time `now`.
pub fn decide(facts: &AccessFacts<'_>, now: Timestamp) -> AccessDecision {
    match evaluate(facts, now) {
        Ok(()) => AccessDecision::Granted,
        Err(reason) => AccessDecision::Denied(reason),
    }
}

/// The checks, in the order required by the specification. The first
/// failure wins; `?` returns early with its reason.
///
/// The status `match`es have no `_` arm on purpose: adding a new status to
/// an enum stops this from compiling until the engine decides how to treat it.
fn evaluate(facts: &AccessFacts<'_>, now: Timestamp) -> Result<(), DenialReason> {
    // 1–2. The card must exist (and be the one that was presented).
    let card = facts.matching_card().ok_or(DenialReason::UnknownCard)?;

    // 3. The card must be active.
    match card.status() {
        CardStatus::Active => {}
        CardStatus::Suspended => return Err(DenialReason::CardSuspended),
        CardStatus::Revoked => return Err(DenialReason::CardRevoked),
        CardStatus::Expired => return Err(DenialReason::CardExpired),
    }

    // 4. The card must not have passed its expiry.
    if card.is_expired_at(now) {
        return Err(DenialReason::CardExpired);
    }

    // 5. The holder must exist, really own this card, and be active.
    let holder = facts.matching_holder().ok_or(DenialReason::UnknownCard)?;
    match holder.status() {
        UserStatus::Active => {}
        UserStatus::Suspended => return Err(DenialReason::UserSuspended),
        UserStatus::Archived => return Err(DenialReason::UserArchived),
    }

    // 6. The door must exist (and be the requested one), be enabled and online.
    let door = facts.matching_door().ok_or(DenialReason::UnknownDoor)?;
    match door.status() {
        DoorStatus::Online => {}
        DoorStatus::Offline => return Err(DenialReason::DoorOffline),
        DoorStatus::Disabled => return Err(DenialReason::DoorDisabled),
    }

    // 7. At least one of the holder's groups must have a permission for this door.
    let mut applicable = facts
        .permissions
        .iter()
        .filter(|permission| permission.applies_to(door.id(), facts.holder_groups))
        .peekable();
    if applicable.peek().is_none() {
        return Err(DenialReason::PermissionDenied);
    }

    // 8. At least one applicable permission must allow access now.
    let in_schedule = applicable.any(|permission| match permission.schedule_id() {
        None => true,
        Some(id) => facts
            .schedules
            .iter()
            .any(|schedule| schedule.id() == id && schedule.allows(now)),
    });
    if in_schedule {
        Ok(())
    } else {
        Err(DenialReason::OutsideSchedule)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, NaiveTime, TimeZone, Utc};
    use chrono_tz::UTC;

    use super::*;
    use crate::domain::{
        CardNumber, ControllerId, DaySet, DoorName, Email, Location, ScheduleName, ScheduleRule,
        UserName,
    };

    /// Wednesday 2026-09-30 10:00 UTC.
    fn now() -> Timestamp {
        Utc.with_ymd_and_hms(2026, 9, 30, 10, 0, 0).unwrap()
    }

    /// Owns all the data; tests tweak one field, then borrow it as facts.
    struct World {
        user: User,
        card: AccessCard,
        door: Door,
        groups: Vec<AccessGroupId>,
        permissions: Vec<AccessPermission>,
        schedules: Vec<AccessSchedule>,
    }

    impl World {
        /// A situation where access is granted: active user and card, online
        /// door, and an unscheduled permission through the user's group.
        fn granted() -> Self {
            let created = now() - Duration::days(1);
            let user = User::register(
                UserName::parse("Alice").unwrap(),
                Email::parse("alice@example.com").unwrap(),
                created,
            );
            let card = AccessCard::issue(
                &user,
                CardNumber::parse("CARD-10001").unwrap(),
                created,
                Some(now() + Duration::days(30)),
            )
            .unwrap();
            let mut door = Door::create(
                DoorName::parse("Main Entrance").unwrap(),
                Location::parse("Building A").unwrap(),
                ControllerId::parse("ctrl-001").unwrap(),
                created,
            );
            door.mark_online().unwrap();
            let group = AccessGroupId::generate();
            let permission = AccessPermission::grant(group, door.id(), None, created);

            Self {
                user,
                card,
                door,
                groups: vec![group],
                permissions: vec![permission],
                schedules: vec![],
            }
        }

        /// Borrows everything; `'_` ties the facts' lifetime to `&self`.
        fn facts(&self) -> AccessFacts<'_> {
            AccessFacts {
                presented_card: Some(self.card.card_number()),
                requested_door: self.door.id(),
                card: Some(&self.card),
                holder: Some(&self.user),
                door: Some(&self.door),
                holder_groups: &self.groups,
                permissions: &self.permissions,
                schedules: &self.schedules,
            }
        }

        fn decide(&self) -> AccessDecision {
            decide(&self.facts(), now())
        }

        /// Replaces the permission with one limited to Mon–Fri `start`–`end` UTC.
        fn with_schedule(mut self, start: u32, end: u32) -> Self {
            let schedule = AccessSchedule::new(
                ScheduleName::parse("Hours").unwrap(),
                UTC,
                vec![ScheduleRule::new(
                    DaySet::WEEKDAYS,
                    NaiveTime::from_hms_opt(start, 0, 0).unwrap(),
                    NaiveTime::from_hms_opt(end, 0, 0).unwrap(),
                )],
                None,
                None,
            )
            .unwrap();
            self.permissions = vec![AccessPermission::grant(
                self.groups[0],
                self.door.id(),
                Some(schedule.id()),
                now(),
            )];
            self.schedules = vec![schedule];
            self
        }
    }

    fn denied(reason: DenialReason) -> AccessDecision {
        AccessDecision::Denied(reason)
    }

    #[test]
    fn valid_request_is_granted() {
        let decision = World::granted().decide();
        assert_eq!(decision, AccessDecision::Granted);
        assert!(decision.is_granted());
        assert_eq!(decision.reason(), None);
    }

    #[test]
    fn unknown_card() {
        let world = World::granted();
        let facts = AccessFacts {
            card: None,
            ..world.facts()
        };
        assert_eq!(decide(&facts, now()), denied(DenialReason::UnknownCard));
    }

    #[test]
    fn malformed_number_never_matches_a_loaded_card() {
        let world = World::granted();
        let facts = AccessFacts {
            presented_card: None,
            ..world.facts()
        };
        assert_eq!(decide(&facts, now()), denied(DenialReason::UnknownCard));
    }

    #[test]
    fn loaded_card_must_have_the_presented_number() {
        let world = World::granted();
        let other = CardNumber::parse("CARD-99999").unwrap();
        let facts = AccessFacts {
            presented_card: Some(&other),
            ..world.facts()
        };
        assert_eq!(decide(&facts, now()), denied(DenialReason::UnknownCard));
    }

    #[test]
    fn loaded_door_must_be_the_requested_door() {
        let world = World::granted();
        let facts = AccessFacts {
            requested_door: DoorId::generate(),
            ..world.facts()
        };
        assert_eq!(decide(&facts, now()), denied(DenialReason::UnknownDoor));
    }

    #[test]
    fn revoked_card() {
        let mut world = World::granted();
        world.card.revoke().unwrap();
        assert_eq!(world.decide(), denied(DenialReason::CardRevoked));
    }

    #[test]
    fn suspended_card() {
        let mut world = World::granted();
        world.card.suspend().unwrap();
        assert_eq!(world.decide(), denied(DenialReason::CardSuspended));
    }

    #[test]
    fn card_stored_as_expired() {
        let mut world = World::granted();
        world.card = AccessCard::restore(
            world.card.id(),
            world.card.user_id(),
            world.card.card_number().clone(),
            CardStatus::Expired,
            world.card.issued_at(),
            world.card.expires_at(),
        );
        assert_eq!(world.decide(), denied(DenialReason::CardExpired));
    }

    #[test]
    fn card_expiry_boundary() {
        let world = World::granted();
        let expiry = world.card.expires_at().unwrap();
        let facts = world.facts();

        assert!(decide(&facts, expiry - Duration::microseconds(1)).is_granted());
        assert_eq!(decide(&facts, expiry), denied(DenialReason::CardExpired));
    }

    #[test]
    fn missing_holder_is_treated_as_unknown_card() {
        let world = World::granted();
        let facts = AccessFacts {
            holder: None,
            ..world.facts()
        };
        assert_eq!(decide(&facts, now()), denied(DenialReason::UnknownCard));
    }

    #[test]
    fn holder_must_own_the_card() {
        let world = World::granted();
        let someone_else = User::register(
            UserName::parse("Mallory").unwrap(),
            Email::parse("mallory@example.com").unwrap(),
            now(),
        );
        let facts = AccessFacts {
            holder: Some(&someone_else),
            ..world.facts()
        };
        assert_eq!(decide(&facts, now()), denied(DenialReason::UnknownCard));
    }

    #[test]
    fn suspended_user() {
        let mut world = World::granted();
        world.user.suspend(now()).unwrap();
        assert_eq!(world.decide(), denied(DenialReason::UserSuspended));
    }

    #[test]
    fn archived_user() {
        let mut world = World::granted();
        world.user.archive(now()).unwrap();
        assert_eq!(world.decide(), denied(DenialReason::UserArchived));
    }

    #[test]
    fn unknown_door() {
        let world = World::granted();
        let facts = AccessFacts {
            door: None,
            ..world.facts()
        };
        assert_eq!(decide(&facts, now()), denied(DenialReason::UnknownDoor));
    }

    #[test]
    fn disabled_door() {
        let mut world = World::granted();
        world.door.disable().unwrap();
        assert_eq!(world.decide(), denied(DenialReason::DoorDisabled));
    }

    #[test]
    fn offline_door() {
        let mut world = World::granted();
        world.door.mark_offline().unwrap();
        assert_eq!(world.decide(), denied(DenialReason::DoorOffline));
    }

    #[test]
    fn no_permission_at_all() {
        let mut world = World::granted();
        world.permissions.clear();
        assert_eq!(world.decide(), denied(DenialReason::PermissionDenied));
    }

    #[test]
    fn permission_for_another_door_does_not_count() {
        let mut world = World::granted();
        world.permissions = vec![AccessPermission::grant(
            world.groups[0],
            DoorId::generate(),
            None,
            now(),
        )];
        assert_eq!(world.decide(), denied(DenialReason::PermissionDenied));
    }

    #[test]
    fn permission_for_a_group_the_user_is_not_in_does_not_count() {
        let mut world = World::granted();
        world.groups = vec![AccessGroupId::generate()];
        assert_eq!(world.decide(), denied(DenialReason::PermissionDenied));
    }

    #[test]
    fn access_within_schedule() {
        let world = World::granted().with_schedule(9, 17);
        assert_eq!(world.decide(), AccessDecision::Granted);
    }

    #[test]
    fn access_outside_schedule() {
        let world = World::granted().with_schedule(12, 17);
        assert_eq!(world.decide(), denied(DenialReason::OutsideSchedule));
    }

    #[test]
    fn schedule_boundaries() {
        let world = World::granted().with_schedule(10, 11);
        let facts = world.facts();
        assert!(decide(&facts, now()).is_granted()); // 10:00, start inclusive
        assert_eq!(
            decide(&facts, now() + Duration::hours(1)), // 11:00, end exclusive
            denied(DenialReason::OutsideSchedule)
        );
    }

    #[test]
    fn missing_schedule_denies() {
        let mut world = World::granted().with_schedule(9, 17);
        world.schedules.clear();
        assert_eq!(world.decide(), denied(DenialReason::OutsideSchedule));
    }

    #[test]
    fn any_applicable_permission_can_grant() {
        let mut world = World::granted().with_schedule(12, 17); // outside now
        let always = AccessPermission::grant(world.groups[0], world.door.id(), None, now());
        world.permissions.push(always);
        assert_eq!(world.decide(), AccessDecision::Granted);
    }

    #[test]
    fn earlier_checks_take_precedence() {
        // Everything is wrong at once; the first check in spec order wins.
        let mut world = World::granted();
        world.card.revoke().unwrap();
        world.user.suspend(now()).unwrap();
        world.door.disable().unwrap();
        world.permissions.clear();
        assert_eq!(world.decide(), denied(DenialReason::CardRevoked));

        let mut world = World::granted();
        world.user.suspend(now()).unwrap();
        world.door.mark_offline().unwrap();
        assert_eq!(world.decide(), denied(DenialReason::UserSuspended));

        let mut world = World::granted();
        world.door.disable().unwrap();
        world.permissions.clear();
        assert_eq!(world.decide(), denied(DenialReason::DoorDisabled));
    }

    #[test]
    fn decisions_are_deterministic() {
        let world = World::granted().with_schedule(9, 17);
        let facts = world.facts();
        let first = decide(&facts, now());
        for _ in 0..100 {
            assert_eq!(decide(&facts, now()), first);
        }
    }

    #[test]
    fn denial_reasons_round_trip_through_str() {
        for reason in DenialReason::ALL {
            assert_eq!(reason.as_str().parse::<DenialReason>(), Ok(reason));
            assert_eq!(reason.to_string(), reason.as_str());
        }
        assert!("UnknownCard".parse::<DenialReason>().is_err());
    }
}
