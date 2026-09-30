//! Access credentials. Card numbers are simulated identifiers, never real
//! payment card data.

use std::{fmt, str::FromStr};

use super::{CardId, DomainError, Timestamp, User, UserId};

/// A normalised (trimmed, upper-cased) simulated card number, e.g. `CARD-10001`.
///
/// 4–32 ASCII letters, digits or `-`, not starting or ending with `-`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CardNumber(String);

impl CardNumber {
    pub fn parse(raw: &str) -> Result<Self, DomainError> {
        let invalid = |reason| DomainError::Validation {
            field: "card_number",
            reason,
        };

        let number = raw.trim().to_ascii_uppercase();
        if number.is_empty() {
            return Err(invalid("must not be empty"));
        }
        if !number
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-')
        {
            return Err(invalid("may only contain letters, digits and '-'"));
        }
        // All bytes are ASCII now, so byte length equals character count.
        if !(4..=32).contains(&number.len()) {
            return Err(invalid("must be 4 to 32 characters"));
        }
        if number.starts_with('-') || number.ends_with('-') {
            return Err(invalid("must not start or end with '-'"));
        }
        Ok(Self(number))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CardNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CardStatus {
    Active,
    Suspended,
    /// Terminal: a revoked credential can never be used again.
    Revoked,
    Expired,
}

impl CardStatus {
    pub const ALL: [Self; 4] = [Self::Active, Self::Suspended, Self::Revoked, Self::Expired];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }

    pub const fn can_transition_to(self, next: Self) -> bool {
        matches!(
            (self, next),
            (Self::Active, Self::Suspended)
                | (Self::Suspended, Self::Active)
                | (
                    Self::Active | Self::Suspended | Self::Expired,
                    Self::Revoked
                )
        )
    }
}

impl FromStr for CardStatus {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|status| status.as_str() == s)
            .ok_or(DomainError::Validation {
                field: "status",
                reason: "unknown card status",
            })
    }
}

/// A credential assigned to one user.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessCard {
    id: CardId,
    user_id: UserId,
    card_number: CardNumber,
    status: CardStatus,
    issued_at: Timestamp,
    /// `None` means the card does not expire.
    expires_at: Option<Timestamp>,
}

impl AccessCard {
    /// Issues a new active card to `user`.
    ///
    /// Borrows the user (`&User`) because it only needs to read the status and
    /// ID; the caller keeps ownership. Only active users may receive cards.
    pub fn issue(
        user: &User,
        card_number: CardNumber,
        issued_at: Timestamp,
        expires_at: Option<Timestamp>,
    ) -> Result<Self, DomainError> {
        if !user.is_active() {
            return Err(DomainError::InvalidTransition {
                entity: "user",
                action: "issue a card to",
                status: user.status().as_str(),
            });
        }
        if expires_at.is_some_and(|expiry| expiry <= issued_at) {
            return Err(DomainError::Validation {
                field: "expires_at",
                reason: "must be after the issue time",
            });
        }

        Ok(Self {
            id: CardId::generate(),
            user_id: user.id(),
            card_number,
            status: CardStatus::Active,
            issued_at,
            expires_at,
        })
    }

    /// Rebuilds a card from stored values. For repositories only.
    pub fn restore(
        id: CardId,
        user_id: UserId,
        card_number: CardNumber,
        status: CardStatus,
        issued_at: Timestamp,
        expires_at: Option<Timestamp>,
    ) -> Self {
        Self {
            id,
            user_id,
            card_number,
            status,
            issued_at,
            expires_at,
        }
    }

    pub fn id(&self) -> CardId {
        self.id
    }

    pub fn user_id(&self) -> UserId {
        self.user_id
    }

    pub fn card_number(&self) -> &CardNumber {
        &self.card_number
    }

    /// The stored status. Use [`Self::effective_status_at`] for decisions.
    pub fn status(&self) -> CardStatus {
        self.status
    }

    pub fn issued_at(&self) -> Timestamp {
        self.issued_at
    }

    pub fn expires_at(&self) -> Option<Timestamp> {
        self.expires_at
    }

    /// A card is expired from the exact `expires_at` instant onwards.
    pub fn is_expired_at(&self, now: Timestamp) -> bool {
        self.expires_at.is_some_and(|expiry| now >= expiry)
    }

    /// The status as of `now`: an `Active` card past its expiry counts as
    /// `Expired` even if nothing has updated the stored status yet.
    pub fn effective_status_at(&self, now: Timestamp) -> CardStatus {
        match self.status {
            CardStatus::Active if self.is_expired_at(now) => CardStatus::Expired,
            status => status,
        }
    }

    pub fn suspend(&mut self) -> Result<(), DomainError> {
        self.transition(CardStatus::Suspended, "suspend")
    }

    /// Re-activates a suspended card, unless it has expired in the meantime.
    pub fn reactivate(&mut self, now: Timestamp) -> Result<(), DomainError> {
        if self.is_expired_at(now) {
            return Err(DomainError::Validation {
                field: "expires_at",
                reason: "card has already expired",
            });
        }
        self.transition(CardStatus::Active, "reactivate")
    }

    pub fn revoke(&mut self) -> Result<(), DomainError> {
        self.transition(CardStatus::Revoked, "revoke")
    }

    fn transition(&mut self, next: CardStatus, action: &'static str) -> Result<(), DomainError> {
        if !self.status.can_transition_to(next) {
            return Err(DomainError::InvalidTransition {
                entity: "card",
                action,
                status: self.status.as_str(),
            });
        }
        self.status = next;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, TimeZone, Utc};

    use super::*;
    use crate::domain::{Email, UserName};

    fn at(hour: u32) -> Timestamp {
        Utc.with_ymd_and_hms(2026, 9, 30, hour, 0, 0).unwrap()
    }

    fn user() -> User {
        User::register(
            UserName::parse("Alice").unwrap(),
            Email::parse("alice@example.com").unwrap(),
            at(8),
        )
    }

    fn number() -> CardNumber {
        CardNumber::parse("CARD-10001").unwrap()
    }

    fn card(expires_at: Option<Timestamp>) -> AccessCard {
        AccessCard::issue(&user(), number(), at(9), expires_at).unwrap()
    }

    #[test]
    fn card_number_is_normalised() {
        assert_eq!(
            CardNumber::parse("  card-10001 ").unwrap().as_str(),
            "CARD-10001"
        );
    }

    #[test]
    fn rejects_malformed_card_numbers() {
        let too_long = "A".repeat(33);
        let cases = [
            "",
            "abc",
            too_long.as_str(),
            "CARD 10001",
            "CARD_10001",
            "-CARD1",
            "CARD1-",
            "CÄRD-1",
        ];
        for raw in cases {
            let err = CardNumber::parse(raw).unwrap_err();
            assert!(
                matches!(
                    err,
                    DomainError::Validation {
                        field: "card_number",
                        ..
                    }
                ),
                "{raw:?} should be rejected, got {err:?}"
            );
        }
    }

    #[test]
    fn issue_creates_active_card_for_user() {
        let user = user();
        let card = AccessCard::issue(&user, number(), at(9), Some(at(17))).unwrap();

        assert_eq!(card.user_id(), user.id());
        assert_eq!(card.status(), CardStatus::Active);
        assert_eq!(card.issued_at(), at(9));
        assert_eq!(card.expires_at(), Some(at(17)));
    }

    #[test]
    fn cannot_issue_to_inactive_user() {
        let mut user = user();
        user.suspend(at(9)).unwrap();

        assert_eq!(
            AccessCard::issue(&user, number(), at(10), None),
            Err(DomainError::InvalidTransition {
                entity: "user",
                action: "issue a card to",
                status: "suspended",
            })
        );
    }

    #[test]
    fn expiry_must_be_after_issue() {
        let user = user();
        assert!(AccessCard::issue(&user, number(), at(9), Some(at(9))).is_err());
        assert!(AccessCard::issue(&user, number(), at(9), Some(at(8))).is_err());
    }

    #[test]
    fn expiry_boundary_is_inclusive() {
        let expiry = at(17);
        let card = card(Some(expiry));

        let just_before = expiry - Duration::nanoseconds(1);
        assert!(!card.is_expired_at(just_before));
        assert_eq!(card.effective_status_at(just_before), CardStatus::Active);

        assert!(card.is_expired_at(expiry));
        assert_eq!(card.effective_status_at(expiry), CardStatus::Expired);
    }

    #[test]
    fn card_without_expiry_never_expires() {
        let card = card(None);
        let far_future = at(9) + Duration::days(365 * 100);
        assert_eq!(card.effective_status_at(far_future), CardStatus::Active);
    }

    #[test]
    fn effective_status_keeps_non_active_statuses() {
        let mut card = card(Some(at(17)));
        card.revoke().unwrap();
        assert_eq!(card.effective_status_at(at(18)), CardStatus::Revoked);
    }

    #[test]
    fn suspend_and_reactivate() {
        let mut card = card(Some(at(17)));
        card.suspend().unwrap();
        assert_eq!(card.status(), CardStatus::Suspended);

        card.reactivate(at(10)).unwrap();
        assert_eq!(card.status(), CardStatus::Active);
    }

    #[test]
    fn cannot_reactivate_after_expiry() {
        let mut card = card(Some(at(17)));
        card.suspend().unwrap();

        assert!(card.reactivate(at(17)).is_err());
        assert_eq!(card.status(), CardStatus::Suspended);
    }

    #[test]
    fn revoked_is_terminal() {
        let mut card = card(None);
        card.revoke().unwrap();

        let expected = |action| {
            Err(DomainError::InvalidTransition {
                entity: "card",
                action,
                status: "revoked",
            })
        };
        assert_eq!(card.suspend(), expected("suspend"));
        assert_eq!(card.reactivate(at(10)), expected("reactivate"));
        assert_eq!(card.revoke(), expected("revoke"));
    }

    #[test]
    fn transition_table() {
        use CardStatus::*;
        let all = CardStatus::ALL;
        let allowed = [
            (Active, Suspended),
            (Suspended, Active),
            (Active, Revoked),
            (Suspended, Revoked),
            (Expired, Revoked),
        ];
        for from in all {
            for to in all {
                assert_eq!(
                    from.can_transition_to(to),
                    allowed.contains(&(from, to)),
                    "{from:?} -> {to:?}"
                );
            }
        }
    }

    #[test]
    fn status_round_trips_through_str() {
        for status in CardStatus::ALL {
            assert_eq!(status.as_str().parse::<CardStatus>(), Ok(status));
        }
        assert!("ACTIVE".parse::<CardStatus>().is_err());
        assert!("".parse::<CardStatus>().is_err());
    }
}
