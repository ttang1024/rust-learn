//! Append-only log of administrative actions ("who did what, when").

use std::str::FromStr;

use super::{AdministratorId, AuditEntryId, DomainError, Timestamp, text::bounded_text};

bounded_text!(
    /// What an action was about, e.g. the username involved.
    AuditSubject,
    field = "subject",
    max = 200
);

/// Declares `AuditAction` from a single list of `Variant => "stored_name"`.
///
/// The `$( ... ),+` repetition expands once per entry, so the enum, the
/// `ALL` list and `as_str` are generated from the same source and can never
/// disagree (a variant missing from `ALL` would otherwise be a silent bug).
macro_rules! audit_actions {
    ($( $(#[$doc:meta])* $variant:ident => $name:literal ),+ $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum AuditAction {
            $( $(#[$doc])* $variant, )+
        }

        impl AuditAction {
            pub const ALL: &'static [Self] = &[ $( Self::$variant ),+ ];

            pub const fn as_str(self) -> &'static str {
                match self {
                    $( Self::$variant => $name, )+
                }
            }
        }
    };
}

audit_actions! {
    LoginSucceeded => "login_succeeded",
    LoginFailed => "login_failed",
    LoggedOut => "logged_out",
    /// A refresh token was presented a second time: likely stolen. All of the
    /// administrator's sessions were revoked in response.
    RefreshTokenReused => "refresh_token_reused",
    AdministratorCreated => "administrator_created",
    UserRegistered => "user_registered",
    UserUpdated => "user_updated",
    UserSuspended => "user_suspended",
    UserReactivated => "user_reactivated",
    UserArchived => "user_archived",
    CardIssued => "card_issued",
    CardSuspended => "card_suspended",
    CardReactivated => "card_reactivated",
    CardRevoked => "card_revoked",
    DoorCreated => "door_created",
    DoorUpdated => "door_updated",
    DoorStatusChanged => "door_status_changed",
    AccessGroupCreated => "access_group_created",
    AccessGroupUpdated => "access_group_updated",
    AccessGroupDeleted => "access_group_deleted",
    GroupMemberAdded => "group_member_added",
    GroupMemberRemoved => "group_member_removed",
    ScheduleCreated => "schedule_created",
    PermissionGranted => "permission_granted",
    PermissionRevoked => "permission_revoked",
    ControllerRegistered => "controller_registered",
    ControllerKeyRotated => "controller_key_rotated",
}

impl FromStr for AuditAction {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .iter()
            .copied()
            .find(|action| action.as_str() == s)
            .ok_or(DomainError::Validation {
                field: "action",
                reason: "unknown audit action",
            })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditEntry {
    id: AuditEntryId,
    /// Who acted. `None` for anonymous attempts and command-line bootstrap.
    actor: Option<AdministratorId>,
    action: AuditAction,
    subject: Option<AuditSubject>,
    occurred_at: Timestamp,
}

impl AuditEntry {
    pub fn record(
        actor: Option<AdministratorId>,
        action: AuditAction,
        subject: Option<AuditSubject>,
        now: Timestamp,
    ) -> Self {
        Self {
            id: AuditEntryId::generate(),
            actor,
            action,
            subject,
            occurred_at: now,
        }
    }

    /// Rebuilds an entry from stored values. For repositories only.
    pub fn restore(
        id: AuditEntryId,
        actor: Option<AdministratorId>,
        action: AuditAction,
        subject: Option<AuditSubject>,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            id,
            actor,
            action,
            subject,
            occurred_at,
        }
    }

    pub fn id(&self) -> AuditEntryId {
        self.id
    }

    pub fn actor(&self) -> Option<AdministratorId> {
        self.actor
    }

    pub fn action(&self) -> AuditAction {
        self.action
    }

    pub fn subject(&self) -> Option<&AuditSubject> {
        self.subject.as_ref()
    }

    pub fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actions_round_trip() {
        for &action in AuditAction::ALL {
            assert_eq!(action.as_str().parse::<AuditAction>(), Ok(action));
        }
        // Stored names are unique (a copy-paste slip would break parsing).
        let mut names: Vec<&str> = AuditAction::ALL.iter().map(|a| a.as_str()).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), AuditAction::ALL.len());
        assert!("hacked".parse::<AuditAction>().is_err());
    }
}
