//! PostgreSQL adapters for the application's repository ports.
//!
//! Queries are built at runtime with `sqlx::query_as` and bound parameters
//! (`$1`, `$2`, ...), so values never become part of the SQL text. The
//! compile-time checked `query!` macros are not used: they need a live
//! database (or a checked-in `.sqlx` cache) just to build the crate. The
//! repository integration tests cover the SQL instead.

mod access_data;
mod administrator_repository;
mod audit_log_repository;
mod card_repository;
mod controller_repository;
mod door_repository;
mod event_repository;
mod group_repository;
mod permission_repository;
mod refresh_token_repository;
mod schedule_repository;
mod user_repository;

use std::time::Duration;

use sqlx::{
    PgPool,
    migrate::Migrator,
    postgres::{PgPoolOptions, PgQueryResult},
};

pub use access_data::PgAccessDataSource;
pub use administrator_repository::PgAdministratorRepository;
pub use audit_log_repository::PgAuditLogRepository;
pub use card_repository::PgCardRepository;
pub use controller_repository::PgControllerRepository;
pub use door_repository::PgDoorRepository;
pub use event_repository::PgAccessEventRepository;
pub use group_repository::PgAccessGroupRepository;
pub use permission_repository::PgPermissionRepository;
pub use refresh_token_repository::PgRefreshTokenRepository;
pub use schedule_repository::PgScheduleRepository;
pub use user_repository::PgUserRepository;

use crate::{
    application::{RepositoryError, RepositoryResult},
    config::DatabaseConfig,
    domain::DomainError,
};

/// The SQL files in `./migrations`, embedded into the binary at compile time.
pub static MIGRATOR: Migrator = sqlx::migrate!();

/// Opens a connection pool. Fails fast if the database is unreachable.
pub async fn connect(config: &DatabaseConfig) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(config.max_connections)
        .acquire_timeout(Duration::from_secs(5))
        .connect(config.url.expose())
        .await
}

/// Round-trips a trivial query, for readiness checks.
pub async fn ping(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT 1").execute(pool).await.map(|_| ())
}

/// What a constraint violation means for the application.
enum Violation {
    Duplicate(&'static str),
    Missing(&'static str),
}

/// Constraint names from the migrations and what violating them means.
///
/// Matching on names is stable, unlike matching on error text, and keeps
/// database details out of upper layers. The Postgres default name for a
/// foreign key is `<table>_<column>_fkey`.
const CONSTRAINTS: &[(&str, Violation)] = &[
    ("users_email_key", Violation::Duplicate("email")),
    (
        "access_cards_card_number_key",
        Violation::Duplicate("card_number"),
    ),
    ("access_cards_user_id_fkey", Violation::Missing("user")),
    ("access_groups_name_key", Violation::Duplicate("name")),
    (
        "user_access_groups_pkey",
        Violation::Duplicate("membership"),
    ),
    (
        "user_access_groups_user_id_fkey",
        Violation::Missing("user"),
    ),
    (
        "user_access_groups_group_id_fkey",
        Violation::Missing("access_group"),
    ),
    ("access_schedules_name_key", Violation::Duplicate("name")),
    (
        "access_permissions_group_door_schedule_key",
        Violation::Duplicate("permission"),
    ),
    (
        "access_permissions_group_id_fkey",
        Violation::Missing("access_group"),
    ),
    (
        "access_permissions_door_id_fkey",
        Violation::Missing("door"),
    ),
    // Only raised on insert today. Once schedules can be deleted, a
    // RESTRICT violation on this key will need its own "in use" error.
    (
        "access_permissions_schedule_id_fkey",
        Violation::Missing("schedule"),
    ),
    (
        "administrators_username_key",
        Violation::Duplicate("username"),
    ),
    (
        "refresh_tokens_administrator_id_fkey",
        Violation::Missing("administrator"),
    ),
    ("controllers_pkey", Violation::Duplicate("controller_id")),
];

/// Translates a SQLx error into the application's vocabulary.
fn map_error(err: sqlx::Error) -> RepositoryError {
    if let sqlx::Error::Database(db_err) = &err
        && let Some(name) = db_err.constraint()
        && let Some((_, violation)) = CONSTRAINTS.iter().find(|(known, _)| *known == name)
    {
        return match *violation {
            Violation::Duplicate(field) => RepositoryError::Duplicate { field },
            Violation::Missing(entity) => RepositoryError::NotFound { entity },
        };
    }
    RepositoryError::Storage(Box::new(err))
}

/// An `UPDATE`/`DELETE ... WHERE <key>` that touched no rows means the target is gone.
fn expect_one_row(result: PgQueryResult, entity: &'static str) -> RepositoryResult<()> {
    if result.rows_affected() == 0 {
        return Err(RepositoryError::NotFound { entity });
    }
    Ok(())
}

/// Converts database rows into domain entities, re-validating every field.
///
/// Generic over the row type `R` and entity `T`: any pair connected by a
/// `TryFrom` impl works.
fn into_entities<R, T>(rows: Vec<R>) -> RepositoryResult<Vec<T>>
where
    T: TryFrom<R, Error = DomainError>,
{
    rows.into_iter()
        .map(|row| T::try_from(row).map_err(RepositoryError::from))
        .collect()
}
