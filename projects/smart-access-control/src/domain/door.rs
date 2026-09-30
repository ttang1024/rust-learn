//! Physical or simulated access points.

use std::{fmt, str::FromStr};

use super::{DomainError, DoorId, Timestamp, text::bounded_text};

bounded_text!(
    /// A door's display name, e.g. "Main Entrance".
    DoorName,
    field = "name",
    max = 100
);

bounded_text!(
    /// Where a door is, e.g. "Building A, Level 1".
    Location,
    field = "location",
    max = 200
);

/// Identifies the (simulated) controller a door is wired to, e.g. `ctrl-001`.
///
/// 1–64 ASCII letters, digits, `-` or `_`. Case is preserved.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ControllerId(String);

impl ControllerId {
    pub fn parse(raw: &str) -> Result<Self, DomainError> {
        let invalid = |reason| DomainError::Validation {
            field: "controller_id",
            reason,
        };

        let id = raw.trim();
        if id.is_empty() {
            return Err(invalid("must not be empty"));
        }
        if !id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        {
            return Err(invalid("may only contain letters, digits, '-' and '_'"));
        }
        if id.len() > 64 {
            return Err(invalid("must be at most 64 characters"));
        }
        Ok(Self(id.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ControllerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DoorStatus {
    /// The controller is connected; the door can process requests.
    Online,
    /// The controller is not connected (the state of a newly created door).
    Offline,
    /// Taken out of service by an administrator. Controller reports are
    /// ignored until the door is enabled again.
    Disabled,
}

impl DoorStatus {
    pub const ALL: [Self; 3] = [Self::Online, Self::Offline, Self::Disabled];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Online => "online",
            Self::Offline => "offline",
            Self::Disabled => "disabled",
        }
    }
}

impl FromStr for DoorStatus {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|status| status.as_str() == s)
            .ok_or(DomainError::Validation {
                field: "status",
                reason: "unknown door status",
            })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Door {
    id: DoorId,
    name: DoorName,
    location: Location,
    controller_id: ControllerId,
    status: DoorStatus,
    created_at: Timestamp,
}

impl Door {
    /// Creates a door. It starts `Offline` until its controller reports in.
    pub fn create(
        name: DoorName,
        location: Location,
        controller_id: ControllerId,
        now: Timestamp,
    ) -> Self {
        Self {
            id: DoorId::generate(),
            name,
            location,
            controller_id,
            status: DoorStatus::Offline,
            created_at: now,
        }
    }

    /// Rebuilds a door from stored values. For repositories only.
    pub fn restore(
        id: DoorId,
        name: DoorName,
        location: Location,
        controller_id: ControllerId,
        status: DoorStatus,
        created_at: Timestamp,
    ) -> Self {
        Self {
            id,
            name,
            location,
            controller_id,
            status,
            created_at,
        }
    }

    pub fn id(&self) -> DoorId {
        self.id
    }

    pub fn name(&self) -> &DoorName {
        &self.name
    }

    pub fn location(&self) -> &Location {
        &self.location
    }

    pub fn controller_id(&self) -> &ControllerId {
        &self.controller_id
    }

    pub fn status(&self) -> DoorStatus {
        self.status
    }

    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }

    pub fn is_online(&self) -> bool {
        self.status == DoorStatus::Online
    }

    pub fn rename(&mut self, name: DoorName) {
        self.name = name;
    }

    pub fn relocate(&mut self, location: Location) {
        self.location = location;
    }

    /// Rewires the door to another controller. Its online state belonged to
    /// the old controller, so an enabled door goes offline until the new
    /// one reports in.
    pub fn assign_controller(&mut self, controller_id: ControllerId) {
        if controller_id == self.controller_id {
            return;
        }
        self.controller_id = controller_id;
        if self.status == DoorStatus::Online {
            self.status = DoorStatus::Offline;
        }
    }

    /// Records that the controller is connected. Repeated reports are
    /// harmless (heartbeats); reports for a disabled door are rejected.
    pub fn mark_online(&mut self) -> Result<(), DomainError> {
        self.apply_controller_report(DoorStatus::Online, "mark online")
    }

    /// Records that the controller has disconnected.
    pub fn mark_offline(&mut self) -> Result<(), DomainError> {
        self.apply_controller_report(DoorStatus::Offline, "mark offline")
    }

    pub fn disable(&mut self) -> Result<(), DomainError> {
        if self.status == DoorStatus::Disabled {
            return Err(self.rejected("disable"));
        }
        self.status = DoorStatus::Disabled;
        Ok(())
    }

    /// Returns a disabled door to service. It becomes `Offline` because its
    /// controller has to report in again before it is trusted as online.
    pub fn enable(&mut self) -> Result<(), DomainError> {
        if self.status != DoorStatus::Disabled {
            return Err(self.rejected("enable"));
        }
        self.status = DoorStatus::Offline;
        Ok(())
    }

    fn apply_controller_report(
        &mut self,
        reported: DoorStatus,
        action: &'static str,
    ) -> Result<(), DomainError> {
        if self.status == DoorStatus::Disabled {
            return Err(self.rejected(action));
        }
        self.status = reported;
        Ok(())
    }

    fn rejected(&self, action: &'static str) -> DomainError {
        DomainError::InvalidTransition {
            entity: "door",
            action,
            status: self.status.as_str(),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::*;

    fn door() -> Door {
        Door::create(
            DoorName::parse("Main Entrance").unwrap(),
            Location::parse("Building A, Level 1").unwrap(),
            ControllerId::parse("ctrl-001").unwrap(),
            Utc.with_ymd_and_hms(2026, 9, 30, 9, 0, 0).unwrap(),
        )
    }

    #[test]
    fn new_door_starts_offline() {
        let door = door();
        assert_eq!(door.status(), DoorStatus::Offline);
        assert!(!door.is_online());
        assert_eq!(door.name().as_str(), "Main Entrance");
        assert_eq!(door.controller_id().as_str(), "ctrl-001");
    }

    #[test]
    fn controller_reports_toggle_online_state() {
        let mut door = door();
        door.mark_online().unwrap();
        assert!(door.is_online());
        door.mark_online().unwrap(); // heartbeat: idempotent
        assert!(door.is_online());
        door.mark_offline().unwrap();
        assert_eq!(door.status(), DoorStatus::Offline);
    }

    #[test]
    fn disabled_door_ignores_controller_reports() {
        let mut door = door();
        door.mark_online().unwrap();
        door.disable().unwrap();

        assert_eq!(
            door.mark_online(),
            Err(DomainError::InvalidTransition {
                entity: "door",
                action: "mark online",
                status: "disabled",
            })
        );
        assert!(door.mark_offline().is_err());
        assert_eq!(door.status(), DoorStatus::Disabled);
    }

    #[test]
    fn enable_returns_door_offline() {
        let mut door = door();
        door.mark_online().unwrap();
        door.disable().unwrap();
        door.enable().unwrap();
        assert_eq!(door.status(), DoorStatus::Offline);
    }

    #[test]
    fn enable_and_disable_reject_no_op_changes() {
        let mut door = door();
        assert!(door.enable().is_err());
        door.disable().unwrap();
        assert!(door.disable().is_err());
    }

    #[test]
    fn new_controller_must_report_in_again() {
        let mut door = door();
        door.mark_online().unwrap();

        door.assign_controller(ControllerId::parse("ctrl-001").unwrap());
        assert!(door.is_online(), "same controller: nothing changes");

        door.assign_controller(ControllerId::parse("ctrl-002").unwrap());
        assert_eq!(door.controller_id().as_str(), "ctrl-002");
        assert_eq!(door.status(), DoorStatus::Offline);

        // A disabled door stays disabled.
        door.disable().unwrap();
        door.assign_controller(ControllerId::parse("ctrl-003").unwrap());
        assert_eq!(door.status(), DoorStatus::Disabled);
    }

    #[test]
    fn controller_id_validation() {
        assert_eq!(
            ControllerId::parse("  Ctrl_01 ").unwrap().as_str(),
            "Ctrl_01"
        );
        let too_long = "c".repeat(65);
        for raw in ["", "ctrl 01", "ctrl/01", too_long.as_str()] {
            assert!(ControllerId::parse(raw).is_err(), "{raw:?}");
        }
    }

    #[test]
    fn location_allows_longer_text_than_name() {
        let text = "x".repeat(150);
        assert!(Location::parse(&text).is_ok());
        assert!(DoorName::parse(&text).is_err());
    }

    #[test]
    fn status_round_trips_through_str() {
        for status in DoorStatus::ALL {
            assert_eq!(status.as_str().parse::<DoorStatus>(), Ok(status));
        }
        assert!("ACTIVE".parse::<DoorStatus>().is_err());
        assert!("".parse::<DoorStatus>().is_err());
    }
}
