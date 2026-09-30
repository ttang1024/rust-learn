//! People who operate the system (not to be confused with `User`, a person
//! who holds access cards).

use std::{fmt, str::FromStr};

use super::{AdministratorId, DomainError, Timestamp};

/// A login name: 3–64 characters of lowercase ASCII letters, digits, `.`,
/// `_` or `-`. Input is trimmed and lower-cased, so logins are case-insensitive.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Username(String);

impl Username {
    pub fn parse(raw: &str) -> Result<Self, DomainError> {
        let invalid = |reason| DomainError::Validation {
            field: "username",
            reason,
        };
        let name = raw.trim().to_ascii_lowercase();
        if !(3..=64).contains(&name.len()) {
            return Err(invalid("must be 3 to 64 characters"));
        }
        if !name.bytes().all(|b| {
            b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'.' | b'_' | b'-')
        }) {
            return Err(invalid(
                "may only contain letters, digits, '.', '_' and '-'",
            ));
        }
        Ok(Self(name))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Username {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// What an administrator may do.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    /// Full read and write access.
    Admin,
    /// Read-only access (monitoring, auditing).
    Viewer,
}

impl Role {
    pub const ALL: [Self; 2] = [Self::Admin, Self::Viewer];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::Viewer => "viewer",
        }
    }

    /// Whether this role may change configuration (users, cards, doors, ...).
    pub const fn can_manage(self) -> bool {
        matches!(self, Self::Admin)
    }
}

impl FromStr for Role {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|role| role.as_str() == s)
            .ok_or(DomainError::Validation {
                field: "role",
                reason: "must be 'admin' or 'viewer'",
            })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AdministratorStatus {
    Active,
    /// Cannot log in or refresh tokens.
    Disabled,
}

impl AdministratorStatus {
    pub const ALL: [Self; 2] = [Self::Active, Self::Disabled];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Disabled => "disabled",
        }
    }
}

impl FromStr for AdministratorStatus {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|status| status.as_str() == s)
            .ok_or(DomainError::Validation {
                field: "status",
                reason: "unknown administrator status",
            })
    }
}

/// A password hash in PHC string format (`$argon2id$v=19$...`).
///
/// Opaque to the domain: only the application's password hasher creates or
/// interprets it. `Debug` is redacted; hashes do not belong in logs.
#[derive(Clone, PartialEq, Eq)]
pub struct PasswordHash(String);

impl PasswordHash {
    pub fn new(phc: String) -> Self {
        Self(phc)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for PasswordHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PasswordHash(<redacted>)")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Administrator {
    id: AdministratorId,
    username: Username,
    password_hash: PasswordHash,
    role: Role,
    status: AdministratorStatus,
    created_at: Timestamp,
}

impl Administrator {
    pub fn create(
        username: Username,
        password_hash: PasswordHash,
        role: Role,
        now: Timestamp,
    ) -> Self {
        Self {
            id: AdministratorId::generate(),
            username,
            password_hash,
            role,
            status: AdministratorStatus::Active,
            created_at: now,
        }
    }

    /// Rebuilds an administrator from stored values. For repositories only.
    pub fn restore(
        id: AdministratorId,
        username: Username,
        password_hash: PasswordHash,
        role: Role,
        status: AdministratorStatus,
        created_at: Timestamp,
    ) -> Self {
        Self {
            id,
            username,
            password_hash,
            role,
            status,
            created_at,
        }
    }

    pub fn id(&self) -> AdministratorId {
        self.id
    }

    pub fn username(&self) -> &Username {
        &self.username
    }

    pub fn password_hash(&self) -> &PasswordHash {
        &self.password_hash
    }

    pub fn role(&self) -> Role {
        self.role
    }

    pub fn status(&self) -> AdministratorStatus {
        self.status
    }

    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }

    pub fn is_active(&self) -> bool {
        self.status == AdministratorStatus::Active
    }

    pub fn disable(&mut self) {
        self.status = AdministratorStatus::Disabled;
    }

    pub fn enable(&mut self) {
        self.status = AdministratorStatus::Active;
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::*;

    #[test]
    fn username_is_normalised_and_validated() {
        assert_eq!(
            Username::parse("  Ops.Lead_1 ").unwrap().as_str(),
            "ops.lead_1"
        );
        let too_long = "a".repeat(65);
        for raw in [
            "",
            "ab",
            "has space",
            "émile",
            "semi;colon",
            too_long.as_str(),
        ] {
            assert!(Username::parse(raw).is_err(), "{raw:?}");
        }
    }

    #[test]
    fn roles_and_statuses_round_trip() {
        for role in Role::ALL {
            assert_eq!(role.as_str().parse::<Role>(), Ok(role));
        }
        for status in AdministratorStatus::ALL {
            assert_eq!(status.as_str().parse::<AdministratorStatus>(), Ok(status));
        }
        assert!("root".parse::<Role>().is_err());
        assert!(Role::Admin.can_manage());
        assert!(!Role::Viewer.can_manage());
    }

    #[test]
    fn new_administrator_is_active_and_can_be_disabled() {
        let mut admin = Administrator::create(
            Username::parse("ops").unwrap(),
            PasswordHash::new("$argon2id$fake".into()),
            Role::Admin,
            Utc.with_ymd_and_hms(2026, 9, 30, 9, 0, 0).unwrap(),
        );
        assert!(admin.is_active());
        admin.disable();
        assert!(!admin.is_active());
        admin.enable();
        assert!(admin.is_active());
    }

    #[test]
    fn password_hash_is_redacted_in_debug() {
        let hash = PasswordHash::new("$argon2id$v=19$secret-material".into());
        assert_eq!(format!("{hash:?}"), "PasswordHash(<redacted>)");
    }
}
