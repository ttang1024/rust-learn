//! Persistence ports. Use cases depend on these traits, never on SQLx.
//!
//! # Why `fn … -> impl Future + Send` instead of `async fn`?
//!
//! Both are allowed in traits on stable Rust, and implementations may still
//! write `async fn`. Spelling out the return type lets the trait promise the
//! future is `Send`, which Axum needs because Tokio may move a request's
//! future between worker threads. With a plain `async fn` in the trait,
//! generic callers could not rely on that.
//!
//! In edition 2024 the returned `impl Future` automatically captures the
//! `&self` and argument lifetimes, so the future may borrow them.

use std::{error::Error as StdError, future::Future};

use thiserror::Error;

use super::{AccessSnapshot, EventFilter, PageRequest};
use crate::domain::{
    AccessCard, AccessEvent, AccessGroup, AccessGroupId, AccessPermission, AccessSchedule,
    Administrator, AdministratorId, AuditEntry, CardId, CardNumber, Controller, ControllerId,
    DomainError, Door, DoorId, Email, EventId, PermissionId, ScheduleId, Timestamp, User, UserId,
    Username,
};

/// Storage failures, described without exposing the storage technology.
#[derive(Debug, Error)]
pub enum RepositoryError {
    /// An update targeted a row that does not exist, or a referenced row is missing.
    #[error("{entity} not found")]
    NotFound { entity: &'static str },

    /// A uniqueness rule was violated, e.g. an email already in use.
    #[error("{field} is already in use")]
    Duplicate { field: &'static str },

    /// A stored row no longer satisfies the domain rules.
    #[error("stored data is invalid: {0}")]
    InvalidData(#[from] DomainError),

    /// Anything else (connection lost, timeout, ...). The source is boxed so
    /// this layer does not depend on the concrete database error type.
    #[error("storage error")]
    Storage(#[source] Box<dyn StdError + Send + Sync>),
}

pub type RepositoryResult<T> = Result<T, RepositoryError>;

pub trait UserRepository: Send + Sync {
    fn insert(&self, user: &User) -> impl Future<Output = RepositoryResult<()>> + Send;

    /// Persists all mutable fields. `NotFound` if the user does not exist.
    fn update(&self, user: &User) -> impl Future<Output = RepositoryResult<()>> + Send;

    fn find_by_id(&self, id: UserId)
    -> impl Future<Output = RepositoryResult<Option<User>>> + Send;

    fn find_by_email(
        &self,
        email: &Email,
    ) -> impl Future<Output = RepositoryResult<Option<User>>> + Send;

    /// Users ordered by creation time.
    fn list(&self, page: PageRequest) -> impl Future<Output = RepositoryResult<Vec<User>>> + Send;
}

pub trait CardRepository: Send + Sync {
    /// `NotFound { entity: "user" }` if the card's user does not exist.
    fn insert(&self, card: &AccessCard) -> impl Future<Output = RepositoryResult<()>> + Send;

    fn update(&self, card: &AccessCard) -> impl Future<Output = RepositoryResult<()>> + Send;

    fn find_by_id(
        &self,
        id: CardId,
    ) -> impl Future<Output = RepositoryResult<Option<AccessCard>>> + Send;

    fn find_by_number(
        &self,
        number: &CardNumber,
    ) -> impl Future<Output = RepositoryResult<Option<AccessCard>>> + Send;

    fn list(
        &self,
        page: PageRequest,
    ) -> impl Future<Output = RepositoryResult<Vec<AccessCard>>> + Send;

    fn list_for_user(
        &self,
        user_id: UserId,
    ) -> impl Future<Output = RepositoryResult<Vec<AccessCard>>> + Send;
}

pub trait DoorRepository: Send + Sync {
    fn insert(&self, door: &Door) -> impl Future<Output = RepositoryResult<()>> + Send;

    fn update(&self, door: &Door) -> impl Future<Output = RepositoryResult<()>> + Send;

    fn find_by_id(&self, id: DoorId)
    -> impl Future<Output = RepositoryResult<Option<Door>>> + Send;

    fn list(&self, page: PageRequest) -> impl Future<Output = RepositoryResult<Vec<Door>>> + Send;

    /// Every door wired to `controller_id`.
    fn list_by_controller(
        &self,
        controller_id: &ControllerId,
    ) -> impl Future<Output = RepositoryResult<Vec<Door>>> + Send;
}

pub trait AccessGroupRepository: Send + Sync {
    /// `Duplicate { field: "name" }` if the name is taken.
    fn insert(&self, group: &AccessGroup) -> impl Future<Output = RepositoryResult<()>> + Send;

    fn update(&self, group: &AccessGroup) -> impl Future<Output = RepositoryResult<()>> + Send;

    /// Also removes the group's memberships and permissions.
    fn delete(&self, id: AccessGroupId) -> impl Future<Output = RepositoryResult<()>> + Send;

    fn find_by_id(
        &self,
        id: AccessGroupId,
    ) -> impl Future<Output = RepositoryResult<Option<AccessGroup>>> + Send;

    /// Groups ordered by name.
    fn list(
        &self,
        page: PageRequest,
    ) -> impl Future<Output = RepositoryResult<Vec<AccessGroup>>> + Send;

    /// `NotFound` for an unknown user or group; `Duplicate { field: "membership" }`
    /// if the user is already a member.
    fn add_member(
        &self,
        group_id: AccessGroupId,
        user_id: UserId,
    ) -> impl Future<Output = RepositoryResult<()>> + Send;

    /// `NotFound { entity: "membership" }` if the user is not a member.
    fn remove_member(
        &self,
        group_id: AccessGroupId,
        user_id: UserId,
    ) -> impl Future<Output = RepositoryResult<()>> + Send;

    /// Ids of the group's members, in a stable order.
    fn list_members(
        &self,
        group_id: AccessGroupId,
    ) -> impl Future<Output = RepositoryResult<Vec<UserId>>> + Send;
}

pub trait ScheduleRepository: Send + Sync {
    /// Stores the schedule and all its rules atomically.
    fn insert(
        &self,
        schedule: &AccessSchedule,
    ) -> impl Future<Output = RepositoryResult<()>> + Send;

    fn find_by_id(
        &self,
        id: ScheduleId,
    ) -> impl Future<Output = RepositoryResult<Option<AccessSchedule>>> + Send;

    /// Schedules ordered by name.
    fn list(
        &self,
        page: PageRequest,
    ) -> impl Future<Output = RepositoryResult<Vec<AccessSchedule>>> + Send;
}

pub trait PermissionRepository: Send + Sync {
    /// `NotFound` if the group, door or schedule does not exist;
    /// `Duplicate { field: "permission" }` if an identical permission exists.
    fn insert(
        &self,
        permission: &AccessPermission,
    ) -> impl Future<Output = RepositoryResult<()>> + Send;

    fn delete(&self, id: PermissionId) -> impl Future<Output = RepositoryResult<()>> + Send;

    fn find_by_id(
        &self,
        id: PermissionId,
    ) -> impl Future<Output = RepositoryResult<Option<AccessPermission>>> + Send;

    fn list(
        &self,
        page: PageRequest,
    ) -> impl Future<Output = RepositoryResult<Vec<AccessPermission>>> + Send;
}

/// The audit log. There is deliberately no update or delete.
pub trait AccessEventRepository: Send + Sync {
    fn append(&self, event: &AccessEvent) -> impl Future<Output = RepositoryResult<()>> + Send;

    fn find_by_id(
        &self,
        id: EventId,
    ) -> impl Future<Output = RepositoryResult<Option<AccessEvent>>> + Send;

    /// Events matching `filter`, newest first.
    fn list(
        &self,
        filter: &EventFilter,
        page: PageRequest,
    ) -> impl Future<Output = RepositoryResult<Vec<AccessEvent>>> + Send;
}

/// Loads everything the decision engine needs for one request.
///
/// A single read port rather than six repositories, so an implementation
/// can read all facts from one consistent snapshot (e.g. one transaction).
pub trait AccessDataSource: Send + Sync {
    /// `card_number` is `None` when the presented number was malformed; the
    /// door is still loaded so the event can reference it.
    fn load(
        &self,
        card_number: Option<&CardNumber>,
        door_id: DoorId,
    ) -> impl Future<Output = RepositoryResult<AccessSnapshot>> + Send;
}

pub trait AdministratorRepository: Send + Sync {
    /// `Duplicate { field: "username" }` if the username is taken.
    fn insert(&self, admin: &Administrator) -> impl Future<Output = RepositoryResult<()>> + Send;

    fn find_by_id(
        &self,
        id: AdministratorId,
    ) -> impl Future<Output = RepositoryResult<Option<Administrator>>> + Send;

    fn find_by_username(
        &self,
        username: &Username,
    ) -> impl Future<Output = RepositoryResult<Option<Administrator>>> + Send;
}

/// A stored refresh token. Only the hash is kept, never the token itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefreshTokenRecord {
    pub token_hash: String,
    pub admin_id: AdministratorId,
    pub created_at: Timestamp,
    pub expires_at: Timestamp,
}

/// Why a refresh token stopped being usable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RevocationReason {
    /// Exchanged for a new pair. Seeing it again means it was copied.
    Rotated,
    /// The session was ended by its owner.
    LoggedOut,
    /// Revoked by the system, e.g. after reuse was detected.
    Revoked,
}

impl RevocationReason {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Rotated => "rotated",
            Self::LoggedOut => "logged_out",
            Self::Revoked => "revoked",
        }
    }
}

/// Result of trying to use a refresh token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsumeOutcome {
    /// The token was valid and unused; it is now marked with the given reason.
    Consumed(RefreshTokenRecord),
    /// The token was already *rotated*: someone kept a copy. Likely theft.
    Reused { admin_id: AdministratorId },
    /// The token was logged out or revoked. Rejected, but not suspicious:
    /// clients commonly retry an old token after logging out.
    Revoked,
    /// No such token.
    Unknown,
}

pub trait RefreshTokenRepository: Send + Sync {
    fn insert(
        &self,
        record: &RefreshTokenRecord,
    ) -> impl Future<Output = RepositoryResult<()>> + Send;

    /// Atomically marks the token as no longer usable, for `reason`. If two
    /// requests race with the same token, exactly one gets `Consumed`.
    fn consume(
        &self,
        token_hash: &str,
        reason: RevocationReason,
        now: Timestamp,
    ) -> impl Future<Output = RepositoryResult<ConsumeOutcome>> + Send;

    /// Revokes every still-valid refresh token of an administrator.
    fn revoke_all(
        &self,
        admin_id: AdministratorId,
        now: Timestamp,
    ) -> impl Future<Output = RepositoryResult<()>> + Send;
}

/// The administrative audit trail. There is deliberately no update or delete.
pub trait AuditLogRepository: Send + Sync {
    fn append(&self, entry: &AuditEntry) -> impl Future<Output = RepositoryResult<()>> + Send;

    /// Newest first.
    fn list_recent(
        &self,
        page: PageRequest,
    ) -> impl Future<Output = RepositoryResult<Vec<AuditEntry>>> + Send;
}

pub trait ControllerRepository: Send + Sync {
    /// `Duplicate { field: "controller_id" }` if the id is taken.
    fn insert(
        &self,
        controller: &Controller,
        key_hash: &str,
    ) -> impl Future<Output = RepositoryResult<()>> + Send;

    /// Persists status and last-seen time.
    fn update(&self, controller: &Controller) -> impl Future<Output = RepositoryResult<()>> + Send;

    fn set_key_hash(
        &self,
        id: &ControllerId,
        key_hash: &str,
    ) -> impl Future<Output = RepositoryResult<()>> + Send;

    fn find_by_id(
        &self,
        id: &ControllerId,
    ) -> impl Future<Output = RepositoryResult<Option<Controller>>> + Send;

    fn find_by_key_hash(
        &self,
        key_hash: &str,
    ) -> impl Future<Output = RepositoryResult<Option<Controller>>> + Send;

    fn list(
        &self,
        page: PageRequest,
    ) -> impl Future<Output = RepositoryResult<Vec<Controller>>> + Send;

    /// Atomically marks every online controller last seen at or before
    /// `cutoff` as offline (the same boundary as `Controller::is_stale`) and
    /// returns their ids. A heartbeat that lands concurrently either happens
    /// before (and the controller stays online) or after (and brings it back
    /// online); it is never silently overwritten.
    fn expire_stale(
        &self,
        cutoff: Timestamp,
    ) -> impl Future<Output = RepositoryResult<Vec<ControllerId>>> + Send;
}
