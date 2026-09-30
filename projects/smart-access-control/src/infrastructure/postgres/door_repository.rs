use sqlx::{PgExecutor, PgPool};
use uuid::Uuid;

use super::{expect_one_row, into_entities, map_error};
use crate::{
    application::{DoorRepository, PageRequest, RepositoryResult},
    domain::{ControllerId, DomainError, Door, DoorId, DoorName, Location, Timestamp},
};

#[derive(Debug, Clone)]
pub struct PgDoorRepository {
    pool: PgPool,
}

impl PgDoorRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct DoorRow {
    id: Uuid,
    name: String,
    location: String,
    controller_id: String,
    status: String,
    created_at: Timestamp,
}

impl TryFrom<DoorRow> for Door {
    type Error = DomainError;

    fn try_from(row: DoorRow) -> Result<Self, Self::Error> {
        Ok(Door::restore(
            DoorId::from_uuid(row.id),
            DoorName::parse(&row.name)?,
            Location::parse(&row.location)?,
            ControllerId::parse(&row.controller_id)?,
            row.status.parse()?,
            row.created_at,
        ))
    }
}

/// Also run inside `PgAccessDataSource`'s snapshot transaction.
pub(super) async fn door_by_id<'e>(
    db: impl PgExecutor<'e>,
    id: DoorId,
) -> RepositoryResult<Option<Door>> {
    let row = sqlx::query_as::<_, DoorRow>(
        "SELECT id, name, location, controller_id, status, created_at
         FROM doors WHERE id = $1",
    )
    .bind(id.as_uuid())
    .fetch_optional(db)
    .await
    .map_err(map_error)?;
    Ok(row.map(Door::try_from).transpose()?)
}

impl DoorRepository for PgDoorRepository {
    async fn insert(&self, door: &Door) -> RepositoryResult<()> {
        sqlx::query(
            "INSERT INTO doors (id, name, location, controller_id, status, created_at)
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(door.id().as_uuid())
        .bind(door.name().as_str())
        .bind(door.location().as_str())
        .bind(door.controller_id().as_str())
        .bind(door.status().as_str())
        .bind(door.created_at())
        .execute(&self.pool)
        .await
        .map_err(map_error)?;
        Ok(())
    }

    async fn update(&self, door: &Door) -> RepositoryResult<()> {
        let result = sqlx::query(
            "UPDATE doors SET name = $2, location = $3, controller_id = $4, status = $5
             WHERE id = $1",
        )
        .bind(door.id().as_uuid())
        .bind(door.name().as_str())
        .bind(door.location().as_str())
        .bind(door.controller_id().as_str())
        .bind(door.status().as_str())
        .execute(&self.pool)
        .await
        .map_err(map_error)?;
        expect_one_row(result, "door")
    }

    async fn find_by_id(&self, id: DoorId) -> RepositoryResult<Option<Door>> {
        door_by_id(&self.pool, id).await
    }

    async fn list(&self, page: PageRequest) -> RepositoryResult<Vec<Door>> {
        let rows = sqlx::query_as::<_, DoorRow>(
            "SELECT id, name, location, controller_id, status, created_at
             FROM doors ORDER BY created_at, id LIMIT $1 OFFSET $2",
        )
        .bind(i64::from(page.limit()))
        .bind(i64::from(page.offset()))
        .fetch_all(&self.pool)
        .await
        .map_err(map_error)?;
        into_entities(rows)
    }

    async fn list_by_controller(
        &self,
        controller_id: &ControllerId,
    ) -> RepositoryResult<Vec<Door>> {
        let rows = sqlx::query_as::<_, DoorRow>(
            "SELECT id, name, location, controller_id, status, created_at
             FROM doors WHERE controller_id = $1 ORDER BY created_at, id",
        )
        .bind(controller_id.as_str())
        .fetch_all(&self.pool)
        .await
        .map_err(map_error)?;
        into_entities(rows)
    }
}
