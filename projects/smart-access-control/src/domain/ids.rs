//! Strongly typed entity identifiers.
//!
//! Each ID wraps a `Uuid` in its own type (the "newtype" pattern), so passing
//! a `DoorId` where a `UserId` is expected is a compile error rather than a
//! silent bug. The wrapper has no runtime cost.

use std::fmt;

use uuid::Uuid;

/// Declares one identifier newtype with the same API as the others.
macro_rules! id_type {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(Uuid);

        impl $name {
            /// Generates a new time-ordered (UUIDv7) identifier.
            ///
            /// v7 IDs sort by creation time, which keeps database index
            /// inserts sequential. IDs never influence business decisions,
            /// so their randomness does not affect determinism.
            pub fn generate() -> Self {
                Self(Uuid::now_v7())
            }

            /// Wraps an existing UUID, e.g. one loaded from storage.
            pub const fn from_uuid(uuid: Uuid) -> Self {
                Self(uuid)
            }

            pub const fn as_uuid(&self) -> Uuid {
                self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}

id_type!(
    /// Identifies a [`User`](super::User).
    UserId
);
id_type!(
    /// Identifies an [`AccessCard`](super::AccessCard).
    CardId
);
id_type!(
    /// Identifies a [`Door`](super::Door).
    DoorId
);
id_type!(
    /// Identifies an [`AccessGroup`](super::AccessGroup).
    AccessGroupId
);
id_type!(
    /// Identifies an [`AccessPermission`](super::AccessPermission).
    PermissionId
);
id_type!(
    /// Identifies an [`AccessSchedule`](super::AccessSchedule).
    ScheduleId
);
id_type!(
    /// Identifies an [`AccessEvent`](super::AccessEvent).
    EventId
);
id_type!(
    /// Identifies an [`Administrator`](super::Administrator).
    AdministratorId
);
id_type!(
    /// Identifies an [`AuditEntry`](super::AuditEntry).
    AuditEntryId
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_ids_are_unique_v7() {
        let a = UserId::generate();
        let b = UserId::generate();
        assert_ne!(a, b);
        assert_eq!(a.as_uuid().get_version_num(), 7);
    }

    #[test]
    fn round_trips_through_uuid() {
        let uuid = Uuid::now_v7();
        let id = DoorId::from_uuid(uuid);
        assert_eq!(id.as_uuid(), uuid);
        assert_eq!(id.to_string(), uuid.to_string());
    }
}
