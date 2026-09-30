//! Links an access group to a door, optionally limited by a schedule.

use super::{AccessGroupId, DoorId, PermissionId, ScheduleId, Timestamp};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessPermission {
    id: PermissionId,
    group_id: AccessGroupId,
    door_id: DoorId,
    /// `None` means "at any time" (all other checks still apply).
    schedule_id: Option<ScheduleId>,
    created_at: Timestamp,
}

impl AccessPermission {
    pub fn grant(
        group_id: AccessGroupId,
        door_id: DoorId,
        schedule_id: Option<ScheduleId>,
        now: Timestamp,
    ) -> Self {
        Self {
            id: PermissionId::generate(),
            group_id,
            door_id,
            schedule_id,
            created_at: now,
        }
    }

    /// Rebuilds a permission from stored values. For repositories only.
    pub fn restore(
        id: PermissionId,
        group_id: AccessGroupId,
        door_id: DoorId,
        schedule_id: Option<ScheduleId>,
        created_at: Timestamp,
    ) -> Self {
        Self {
            id,
            group_id,
            door_id,
            schedule_id,
            created_at,
        }
    }

    pub fn id(&self) -> PermissionId {
        self.id
    }

    pub fn group_id(&self) -> AccessGroupId {
        self.group_id
    }

    pub fn door_id(&self) -> DoorId {
        self.door_id
    }

    pub fn schedule_id(&self) -> Option<ScheduleId> {
        self.schedule_id
    }

    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }

    /// Whether this permission covers `door` for a member of any of `groups`.
    pub fn applies_to(&self, door: DoorId, groups: &[AccessGroupId]) -> bool {
        self.door_id == door && groups.contains(&self.group_id)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::*;

    #[test]
    fn applies_only_to_its_door_and_group() {
        let (group, other_group) = (AccessGroupId::generate(), AccessGroupId::generate());
        let (door, other_door) = (DoorId::generate(), DoorId::generate());
        let now = Utc.with_ymd_and_hms(2026, 9, 30, 9, 0, 0).unwrap();
        let permission = AccessPermission::grant(group, door, None, now);

        assert!(permission.applies_to(door, &[other_group, group]));
        assert!(!permission.applies_to(other_door, &[group]));
        assert!(!permission.applies_to(door, &[other_group]));
        assert!(!permission.applies_to(door, &[]));
    }
}
