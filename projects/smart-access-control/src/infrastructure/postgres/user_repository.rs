use sqlx::{PgExecutor, PgPool};
use uuid::Uuid;

use super::{expect_one_row, into_entities, map_error};
use crate::{
    application::{PageRequest, RepositoryResult, UserRepository},
    domain::{DomainError, Email, Timestamp, User, UserId, UserName},
};

#[derive(Debug, Clone)]
pub struct PgUserRepository {
    pool: PgPool,
}

impl PgUserRepository {
    /// `PgPool` is a cheap, reference-counted handle; cloning shares the pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// One `users` row exactly as stored. Private: only this module sees SQL shapes.
#[derive(sqlx::FromRow)]
struct UserRow {
    id: Uuid,
    name: String,
    email: String,
    status: String,
    created_at: Timestamp,
    updated_at: Timestamp,
}

impl TryFrom<UserRow> for User {
    type Error = DomainError;

    fn try_from(row: UserRow) -> Result<Self, Self::Error> {
        Ok(User::restore(
            UserId::from_uuid(row.id),
            UserName::parse(&row.name)?,
            Email::parse(&row.email)?,
            row.status.parse()?,
            row.created_at,
            row.updated_at,
        ))
    }
}

/// Also run inside `PgAccessDataSource`'s snapshot transaction.
pub(super) async fn user_by_id<'e>(
    db: impl PgExecutor<'e>,
    id: UserId,
) -> RepositoryResult<Option<User>> {
    let row = sqlx::query_as::<_, UserRow>(
        "SELECT id, name, email, status, created_at, updated_at
         FROM users WHERE id = $1",
    )
    .bind(id.as_uuid())
    .fetch_optional(db)
    .await
    .map_err(map_error)?;

    // Option<Result<T, E>> -> Result<Option<T>, E>, then `?` for the error.
    Ok(row.map(User::try_from).transpose()?)
}

// The trait declares `fn ... -> impl Future + Send`; implementing it with
// `async fn` is allowed, and the compiler verifies the future is `Send`.
impl UserRepository for PgUserRepository {
    async fn insert(&self, user: &User) -> RepositoryResult<()> {
        sqlx::query(
            "INSERT INTO users (id, name, email, status, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(user.id().as_uuid())
        .bind(user.name().as_str())
        .bind(user.email().as_str())
        .bind(user.status().as_str())
        .bind(user.created_at())
        .bind(user.updated_at())
        .execute(&self.pool)
        .await
        .map_err(map_error)?;
        Ok(())
    }

    async fn update(&self, user: &User) -> RepositoryResult<()> {
        let result = sqlx::query(
            "UPDATE users SET name = $2, email = $3, status = $4, updated_at = $5
             WHERE id = $1",
        )
        .bind(user.id().as_uuid())
        .bind(user.name().as_str())
        .bind(user.email().as_str())
        .bind(user.status().as_str())
        .bind(user.updated_at())
        .execute(&self.pool)
        .await
        .map_err(map_error)?;
        expect_one_row(result, "user")
    }

    async fn find_by_id(&self, id: UserId) -> RepositoryResult<Option<User>> {
        user_by_id(&self.pool, id).await
    }

    async fn find_by_email(&self, email: &Email) -> RepositoryResult<Option<User>> {
        let row = sqlx::query_as::<_, UserRow>(
            "SELECT id, name, email, status, created_at, updated_at
             FROM users WHERE email = $1",
        )
        .bind(email.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_error)?;
        Ok(row.map(User::try_from).transpose()?)
    }

    async fn list(&self, page: PageRequest) -> RepositoryResult<Vec<User>> {
        let rows = sqlx::query_as::<_, UserRow>(
            "SELECT id, name, email, status, created_at, updated_at
             FROM users ORDER BY created_at, id LIMIT $1 OFFSET $2",
        )
        .bind(i64::from(page.limit()))
        .bind(i64::from(page.offset()))
        .fetch_all(&self.pool)
        .await
        .map_err(map_error)?;
        into_entities(rows)
    }
}
