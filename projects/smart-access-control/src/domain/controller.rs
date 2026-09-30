//! Door controllers: the devices (here: simulated) that read cards and
//! drive doors. Not compatible with, or a model of, any vendor's hardware.

use std::str::FromStr;

use chrono::Duration;

use super::{ControllerId, DomainError, Timestamp};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ControllerStatus {
    /// Heard from within the heartbeat timeout.
    Online,
    /// Never connected, disconnected cleanly, or went silent.
    Offline,
}

impl ControllerStatus {
    pub const ALL: [Self; 2] = [Self::Online, Self::Offline];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Online => "online",
            Self::Offline => "offline",
        }
    }
}

impl FromStr for ControllerStatus {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|status| status.as_str() == s)
            .ok_or(DomainError::Validation {
                field: "status",
                reason: "unknown controller status",
            })
    }
}

/// A registered controller. Its secret key is not part of the entity: only
/// the credential store ever sees (the hash of) it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Controller {
    id: ControllerId,
    status: ControllerStatus,
    last_seen_at: Option<Timestamp>,
    created_at: Timestamp,
}

impl Controller {
    /// A newly registered controller is offline until its first heartbeat.
    pub fn register(id: ControllerId, now: Timestamp) -> Self {
        Self {
            id,
            status: ControllerStatus::Offline,
            last_seen_at: None,
            created_at: now,
        }
    }

    /// Rebuilds a controller from stored values. For repositories only.
    pub fn restore(
        id: ControllerId,
        status: ControllerStatus,
        last_seen_at: Option<Timestamp>,
        created_at: Timestamp,
    ) -> Self {
        Self {
            id,
            status,
            last_seen_at,
            created_at,
        }
    }

    pub fn id(&self) -> &ControllerId {
        &self.id
    }

    pub fn status(&self) -> ControllerStatus {
        self.status
    }

    pub fn last_seen_at(&self) -> Option<Timestamp> {
        self.last_seen_at
    }

    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }

    pub fn is_online(&self) -> bool {
        self.status == ControllerStatus::Online
    }

    /// The controller made contact.
    pub fn record_heartbeat(&mut self, now: Timestamp) {
        self.status = ControllerStatus::Online;
        self.last_seen_at = Some(now);
    }

    pub fn mark_offline(&mut self) {
        self.status = ControllerStatus::Offline;
    }

    /// Online, but silent for at least `timeout`.
    pub fn is_stale(&self, now: Timestamp, timeout: Duration) -> bool {
        self.is_online() && self.last_seen_at.is_none_or(|seen| seen + timeout <= now)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::*;

    fn at(second: i64) -> Timestamp {
        Utc.with_ymd_and_hms(2026, 9, 30, 10, 0, 0).unwrap() + Duration::seconds(second)
    }

    fn controller() -> Controller {
        Controller::register(ControllerId::parse("ctrl-001").unwrap(), at(0))
    }

    #[test]
    fn starts_offline_and_comes_online_on_heartbeat() {
        let mut c = controller();
        assert!(!c.is_online());
        assert_eq!(c.last_seen_at(), None);

        c.record_heartbeat(at(5));
        assert!(c.is_online());
        assert_eq!(c.last_seen_at(), Some(at(5)));
    }

    #[test]
    fn stale_after_the_timeout_exactly() {
        let mut c = controller();
        c.record_heartbeat(at(0));
        let timeout = Duration::seconds(30);

        assert!(!c.is_stale(at(29), timeout));
        assert!(c.is_stale(at(30), timeout));

        c.mark_offline();
        assert!(
            !c.is_stale(at(60), timeout),
            "offline controllers are not 'stale'"
        );
    }

    #[test]
    fn statuses_round_trip() {
        for status in ControllerStatus::ALL {
            assert_eq!(status.as_str().parse::<ControllerStatus>(), Ok(status));
        }
    }
}
