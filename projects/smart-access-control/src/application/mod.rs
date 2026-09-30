//! Application layer: use cases (register users, issue cards, evaluate
//! access requests, ...) and the ports (traits) they depend on.
//!
//! Depends only on `domain`. Concrete implementations of the traits live in
//! `infrastructure`, so use cases can be tested with in-memory fakes.
//!
//! Each service follows the same shape: parse raw input into domain types,
//! load what it needs, let the domain apply the rule, then persist.

mod access;
mod audit;
mod auth;
mod cards;
mod clock;
mod controllers;
mod doors;
mod error;
pub(crate) mod events;
mod groups;
mod pagination;
mod permissions;
mod repositories;
mod schedules;
mod users;

#[cfg(test)]
mod fakes;

pub use access::{AccessDecisionService, AccessRequest, AccessSnapshot};
pub use audit::AuditTrail;
pub use auth::{
    AccessClaims, AuthService, BoxError, CreateAdministrator, IssuedAccessToken, LoginThrottle,
    Password, PasswordHasher, RetryAfter, SecretGenerator, ThrottlePolicy, TokenIssuer, TokenPair,
};
pub use cards::{CardService, IssueCard};
pub use clock::Clock;
pub use controllers::{ControllerService, HeartbeatReport, IssuedControllerKey};
pub use doors::{CreateDoor, DoorService, UpdateDoor};
pub use error::ApplicationError;
pub use events::{AccessEventService, DecisionFilter, EventFilter, EventPublisher};
pub use groups::{AccessGroupService, CreateGroup, UpdateGroup};
pub use pagination::PageRequest;
pub use permissions::{GrantPermission, PermissionService};
pub use repositories::{
    AccessDataSource, AccessEventRepository, AccessGroupRepository, AdministratorRepository,
    AuditLogRepository, CardRepository, ConsumeOutcome, ControllerRepository, DoorRepository,
    PermissionRepository, RefreshTokenRecord, RefreshTokenRepository, RepositoryError,
    RepositoryResult, RevocationReason, ScheduleRepository, UserRepository,
};
pub use schedules::{CreateSchedule, RuleInput, ScheduleService};
pub use users::{RegisterUser, UpdateUser, UserService};
