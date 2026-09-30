//! Domain layer: entities, value objects and business rules
//! (users, cards, doors, schedules, access decisions).
//!
//! Must not depend on Axum, SQLx, HTTP or any other infrastructure crate.
//!
//! Entities receive the current time as an explicit `now` argument instead of
//! reading a clock, so every rule is deterministic and easy to test.

mod access;
mod access_group;
mod administrator;
mod audit;
mod card;
mod controller;
mod door;
mod error;
mod event;
mod ids;
mod permission;
mod schedule;
mod text;
mod user;

pub use access::{AccessDecision, AccessFacts, DenialReason, decide};
pub use access_group::{AccessGroup, Description, GroupName};
pub use administrator::{Administrator, AdministratorStatus, PasswordHash, Role, Username};
pub use audit::{AuditAction, AuditEntry, AuditSubject};
pub use card::{AccessCard, CardNumber, CardStatus};
pub use controller::{Controller, ControllerStatus};
pub use door::{ControllerId, Door, DoorName, DoorStatus, Location};
pub use error::DomainError;
pub use event::AccessEvent;
pub use ids::{
    AccessGroupId, AdministratorId, AuditEntryId, CardId, DoorId, EventId, PermissionId,
    ScheduleId, UserId,
};
pub use permission::AccessPermission;
pub use schedule::{AccessSchedule, DaySet, ScheduleName, ScheduleRule, parse_time_zone};
pub use user::{Email, User, UserName, UserStatus};

/// A point in time. Always UTC; local time zones only matter for schedules.
pub type Timestamp = chrono::DateTime<chrono::Utc>;
