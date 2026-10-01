use sqlx::PgPool;

use super::{expect_one_row, into_entities, into_entity, map_error};
use crate::{
    application::{ControllerRepository, PageRequest, RepositoryResult},
    domain::{Controller, ControllerId, DomainError, Timestamp},
};

#[derive(Debug, Clone)]
pub struct PgControllerRepository {
    pool: PgPool,
}

impl PgControllerRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct ControllerRow {
    id: String,
    status: String,
    last_seen_at: Option<Timestamp>,
    created_at: Timestamp,
}

impl TryFrom<ControllerRow> for Controller {
    type Error = DomainError;

    fn try_from(row: ControllerRow) -> Result<Self, Self::Error> {
        Ok(Controller::restore(
            ControllerId::parse(&row.id)?,
            row.status.parse()?,
            row.last_seen_at,
            row.created_at,
        ))
    }
}

impl ControllerRepository for PgControllerRepository {
    async fn insert(&self, controller: &Controller, key_hash: &str) -> RepositoryResult<()> {
        sqlx::query(
            "INSERT INTO controllers (id, key_hash, status, last_seen_at, created_at)
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(controller.id().as_str())
        .bind(key_hash)
        .bind(controller.status().as_str())
        .bind(controller.last_seen_at())
        .bind(controller.created_at())
        .execute(&self.pool)
        .await
        .map_err(map_error)?;
        Ok(())
    }

    async fn update(&self, controller: &Controller) -> RepositoryResult<()> {
        let result =
            sqlx::query("UPDATE controllers SET status = $2, last_seen_at = $3 WHERE id = $1")
                .bind(controller.id().as_str())
                .bind(controller.status().as_str())
                .bind(controller.last_seen_at())
                .execute(&self.pool)
                .await
                .map_err(map_error)?;
        expect_one_row(result, "controller")
    }

    async fn set_key_hash(&self, id: &ControllerId, key_hash: &str) -> RepositoryResult<()> {
        let result = sqlx::query("UPDATE controllers SET key_hash = $2 WHERE id = $1")
            .bind(id.as_str())
            .bind(key_hash)
            .execute(&self.pool)
            .await
            .map_err(map_error)?;
        expect_one_row(result, "controller")
    }

    async fn find_by_id(&self, id: &ControllerId) -> RepositoryResult<Option<Controller>> {
        let row = sqlx::query_as::<_, ControllerRow>(
            "SELECT id, status, last_seen_at, created_at FROM controllers WHERE id = $1",
        )
        .bind(id.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_error)?;
        into_entity(row)
    }

    async fn find_by_key_hash(&self, key_hash: &str) -> RepositoryResult<Option<Controller>> {
        let row = sqlx::query_as::<_, ControllerRow>(
            "SELECT id, status, last_seen_at, created_at FROM controllers WHERE key_hash = $1",
        )
        .bind(key_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_error)?;
        into_entity(row)
    }

    async fn list(&self, page: PageRequest) -> RepositoryResult<Vec<Controller>> {
        let rows = sqlx::query_as::<_, ControllerRow>(
            "SELECT id, status, last_seen_at, created_at FROM controllers
             ORDER BY id LIMIT $1 OFFSET $2",
        )
        .bind(i64::from(page.limit()))
        .bind(i64::from(page.offset()))
        .fetch_all(&self.pool)
        .await
        .map_err(map_error)?;
        into_entities(rows)
    }

    /// Check and update in one statement, so it cannot overwrite a heartbeat
    /// that arrived after the check (the classic check-then-act race).
    async fn expire_stale(&self, cutoff: Timestamp) -> RepositoryResult<Vec<ControllerId>> {
        let ids: Vec<String> = sqlx::query_scalar(
            "UPDATE controllers SET status = 'offline'
             WHERE status = 'online' AND (last_seen_at IS NULL OR last_seen_at <= $1)
             RETURNING id",
        )
        .bind(cutoff)
        .fetch_all(&self.pool)
        .await
        .map_err(map_error)?;
        Ok(ids
            .iter()
            .map(|id| ControllerId::parse(id))
            .collect::<Result<_, _>>()?)
    }
}
