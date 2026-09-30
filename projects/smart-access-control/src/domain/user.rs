//! A person who may be granted access to doors.

use std::{fmt, str::FromStr};

use super::{DomainError, Timestamp, UserId, text::bounded_text};

bounded_text!(
    /// A user's display name: trimmed, 1–100 characters.
    UserName,
    field = "name",
    max = 100
);

/// A normalised (trimmed, lower-cased) email address.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Email(String);

impl Email {
    pub const MAX_CHARS: usize = 254;

    /// Parses an email address with a structural check (`local@domain.tld`).
    ///
    /// This is deliberately not full RFC 5322 validation: the only reliable
    /// check is delivering mail. Lower-casing makes the uniqueness rule
    /// case-insensitive.
    pub fn parse(raw: &str) -> Result<Self, DomainError> {
        let invalid = |reason| DomainError::Validation {
            field: "email",
            reason,
        };

        let email = raw.trim().to_lowercase();
        if email.is_empty() {
            return Err(invalid("must not be empty"));
        }
        if email.chars().count() > Self::MAX_CHARS {
            return Err(invalid("must be at most 254 characters"));
        }
        if email.chars().any(|c| c.is_whitespace() || c.is_control()) {
            return Err(invalid("must not contain whitespace"));
        }

        // `let ... else` binds on success and must diverge (return) otherwise.
        let Some((local, domain)) = email.split_once('@') else {
            return Err(invalid("must contain '@'"));
        };
        let domain_ok = domain.contains('.') && domain.split('.').all(|label| !label.is_empty());
        if local.is_empty() || domain.contains('@') || !domain_ok {
            return Err(invalid("must look like name@example.com"));
        }

        Ok(Self(email))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Email {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UserStatus {
    Active,
    Suspended,
    /// Terminal: kept only so historical access events still resolve.
    Archived,
}

impl UserStatus {
    pub const ALL: [Self; 3] = [Self::Active, Self::Suspended, Self::Archived];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Archived => "archived",
        }
    }

    /// The whole user lifecycle in one place.
    pub const fn can_transition_to(self, next: Self) -> bool {
        matches!(
            (self, next),
            (Self::Active, Self::Suspended)
                | (Self::Suspended, Self::Active)
                | (Self::Active | Self::Suspended, Self::Archived)
        )
    }
}

impl FromStr for UserStatus {
    type Err = DomainError;

    /// Inverse of [`UserStatus::as_str`]; derived from `ALL` so the two cannot drift.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|status| status.as_str() == s)
            .ok_or(DomainError::Validation {
                field: "status",
                reason: "unknown user status",
            })
    }
}

/// A person who can hold access cards.
///
/// Fields are private: every change goes through a method that enforces the
/// rules and keeps `updated_at` correct. Getters hand out `&` references, so
/// callers can read without cloning but cannot mutate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    id: UserId,
    name: UserName,
    email: Email,
    status: UserStatus,
    created_at: Timestamp,
    updated_at: Timestamp,
}

impl User {
    /// Registers a new, active user. Takes ownership of the validated values.
    pub fn register(name: UserName, email: Email, now: Timestamp) -> Self {
        Self {
            id: UserId::generate(),
            name,
            email,
            status: UserStatus::Active,
            created_at: now,
            updated_at: now,
        }
    }

    /// Rebuilds a user from stored values. For repositories only: it skips the
    /// lifecycle rules because the data was valid when it was saved.
    pub fn restore(
        id: UserId,
        name: UserName,
        email: Email,
        status: UserStatus,
        created_at: Timestamp,
        updated_at: Timestamp,
    ) -> Self {
        Self {
            id,
            name,
            email,
            status,
            created_at,
            updated_at,
        }
    }

    pub fn id(&self) -> UserId {
        self.id
    }

    pub fn name(&self) -> &UserName {
        &self.name
    }

    pub fn email(&self) -> &Email {
        &self.email
    }

    pub fn status(&self) -> UserStatus {
        self.status
    }

    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }

    pub fn updated_at(&self) -> Timestamp {
        self.updated_at
    }

    pub fn is_active(&self) -> bool {
        self.status == UserStatus::Active
    }

    pub fn rename(&mut self, name: UserName, now: Timestamp) -> Result<(), DomainError> {
        self.ensure_editable("rename")?;
        self.name = name;
        self.updated_at = now;
        Ok(())
    }

    pub fn change_email(&mut self, email: Email, now: Timestamp) -> Result<(), DomainError> {
        self.ensure_editable("change email of")?;
        self.email = email;
        self.updated_at = now;
        Ok(())
    }

    pub fn suspend(&mut self, now: Timestamp) -> Result<(), DomainError> {
        self.transition(UserStatus::Suspended, "suspend", now)
    }

    pub fn reactivate(&mut self, now: Timestamp) -> Result<(), DomainError> {
        self.transition(UserStatus::Active, "reactivate", now)
    }

    pub fn archive(&mut self, now: Timestamp) -> Result<(), DomainError> {
        self.transition(UserStatus::Archived, "archive", now)
    }

    fn ensure_editable(&self, action: &'static str) -> Result<(), DomainError> {
        if self.status == UserStatus::Archived {
            return Err(self.rejected(action));
        }
        Ok(())
    }

    /// Changes status only if the lifecycle allows it; on error `self` is untouched.
    fn transition(
        &mut self,
        next: UserStatus,
        action: &'static str,
        now: Timestamp,
    ) -> Result<(), DomainError> {
        if !self.status.can_transition_to(next) {
            return Err(self.rejected(action));
        }
        self.status = next;
        self.updated_at = now;
        Ok(())
    }

    fn rejected(&self, action: &'static str) -> DomainError {
        DomainError::InvalidTransition {
            entity: "user",
            action,
            status: self.status.as_str(),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::*;

    fn at(hour: u32) -> Timestamp {
        Utc.with_ymd_and_hms(2026, 9, 30, hour, 0, 0).unwrap()
    }

    fn alice() -> User {
        User::register(
            UserName::parse("Alice").unwrap(),
            Email::parse("alice@example.com").unwrap(),
            at(9),
        )
    }

    #[test]
    fn register_creates_active_user() {
        let user = alice();
        assert_eq!(user.status(), UserStatus::Active);
        assert!(user.is_active());
        assert_eq!(user.created_at(), at(9));
        assert_eq!(user.updated_at(), at(9));
    }

    #[test]
    fn email_is_trimmed_and_lower_cased() {
        let email = Email::parse("  Alice.Smith@Example.COM ").unwrap();
        assert_eq!(email.as_str(), "alice.smith@example.com");
    }

    #[test]
    fn rejects_malformed_emails() {
        let cases = [
            "",
            "   ",
            "alice",
            "@example.com",
            "alice@",
            "alice@example",
            "alice@example.",
            "alice@.com",
            "alice@@example.com",
            "a@b@example.com",
            "al ice@example.com",
        ];
        for raw in cases {
            let err = Email::parse(raw).unwrap_err();
            assert!(
                matches!(err, DomainError::Validation { field: "email", .. }),
                "{raw:?} should be rejected, got {err:?}"
            );
        }
    }

    #[test]
    fn rejects_overlong_email() {
        let raw = format!("{}@example.com", "a".repeat(250));
        assert!(Email::parse(&raw).is_err());
    }

    #[test]
    fn suspend_and_reactivate() {
        let mut user = alice();

        user.suspend(at(10)).unwrap();
        assert_eq!(user.status(), UserStatus::Suspended);
        assert!(!user.is_active());
        assert_eq!(user.updated_at(), at(10));

        user.reactivate(at(11)).unwrap();
        assert!(user.is_active());
        assert_eq!(user.updated_at(), at(11));
    }

    #[test]
    fn archived_is_terminal() {
        let mut user = alice();
        user.archive(at(10)).unwrap();

        assert_eq!(
            user.reactivate(at(11)),
            Err(DomainError::InvalidTransition {
                entity: "user",
                action: "reactivate",
                status: "archived",
            })
        );
        assert!(user.suspend(at(11)).is_err());
        assert!(
            user.rename(UserName::parse("Bob").unwrap(), at(11))
                .is_err()
        );
        assert!(
            user.change_email(Email::parse("bob@example.com").unwrap(), at(11))
                .is_err()
        );
    }

    #[test]
    fn rejected_change_leaves_user_unchanged() {
        let mut user = alice();
        let before = user.clone();

        assert!(user.reactivate(at(10)).is_err()); // already active
        assert_eq!(user, before);
    }

    #[test]
    fn rename_and_change_email_update_timestamp() {
        let mut user = alice();
        user.rename(UserName::parse("Alice Smith").unwrap(), at(10))
            .unwrap();
        user.change_email(Email::parse("asmith@example.com").unwrap(), at(11))
            .unwrap();

        assert_eq!(user.name().as_str(), "Alice Smith");
        assert_eq!(user.email().as_str(), "asmith@example.com");
        assert_eq!(user.updated_at(), at(11));
    }

    #[test]
    fn transition_table() {
        use UserStatus::*;
        let all = UserStatus::ALL;
        let allowed = [
            (Active, Suspended),
            (Suspended, Active),
            (Active, Archived),
            (Suspended, Archived),
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
        for status in UserStatus::ALL {
            assert_eq!(status.as_str().parse::<UserStatus>(), Ok(status));
        }
        assert!("ACTIVE".parse::<UserStatus>().is_err());
        assert!("".parse::<UserStatus>().is_err());
    }
}
