use sqlx::PgPool;
use uuid::Uuid;

use super::{into_entity, map_error};
use crate::{
    application::{AdministratorRepository, RepositoryResult},
    domain::{Administrator, AdministratorId, DomainError, PasswordHash, Timestamp, Username},
};

#[derive(Debug, Clone)]
pub struct PgAdministratorRepository {
    pool: PgPool,
}

impl PgAdministratorRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct AdministratorRow {
    id: Uuid,
    username: String,
    password_hash: String,
    role: String,
    status: String,
    created_at: Timestamp,
}

impl TryFrom<AdministratorRow> for Administrator {
    type Error = DomainError;

    fn try_from(row: AdministratorRow) -> Result<Self, Self::Error> {
        Ok(Administrator::restore(
            AdministratorId::from_uuid(row.id),
            Username::parse(&row.username)?,
            PasswordHash::new(row.password_hash),
            row.role.parse()?,
            row.status.parse()?,
            row.created_at,
        ))
    }
}

impl AdministratorRepository for PgAdministratorRepository {
    async fn insert(&self, admin: &Administrator) -> RepositoryResult<()> {
        sqlx::query(
            "INSERT INTO administrators (id, username, password_hash, role, status, created_at)
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(admin.id().as_uuid())
        .bind(admin.username().as_str())
        .bind(admin.password_hash().as_str())
        .bind(admin.role().as_str())
        .bind(admin.status().as_str())
        .bind(admin.created_at())
        .execute(&self.pool)
        .await
        .map_err(map_error)?;
        Ok(())
    }

    async fn find_by_id(&self, id: AdministratorId) -> RepositoryResult<Option<Administrator>> {
        let row = sqlx::query_as::<_, AdministratorRow>(
            "SELECT id, username, password_hash, role, status, created_at
             FROM administrators WHERE id = $1",
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_error)?;
        into_entity(row)
    }

    async fn find_by_username(
        &self,
        username: &Username,
    ) -> RepositoryResult<Option<Administrator>> {
        let row = sqlx::query_as::<_, AdministratorRow>(
            "SELECT id, username, password_hash, role, status, created_at
             FROM administrators WHERE username = $1",
        )
        .bind(username.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_error)?;
        into_entity(row)
    }
}
