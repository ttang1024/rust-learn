use sqlx::PgPool;
use uuid::Uuid;

use super::{expect_one_row, map_error};
use crate::{
    application::{PageRequest, PermissionRepository, RepositoryResult},
    domain::{AccessGroupId, AccessPermission, DoorId, PermissionId, ScheduleId, Timestamp},
};

#[derive(Debug, Clone)]
pub struct PgPermissionRepository {
    pool: PgPool,
}

impl PgPermissionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
pub(super) struct PermissionRow {
    id: Uuid,
    group_id: Uuid,
    door_id: Uuid,
    schedule_id: Option<Uuid>,
    created_at: Timestamp,
}

/// `From`, not `TryFrom`: every column is an ID or timestamp, so there is
/// nothing to validate and the conversion cannot fail.
impl From<PermissionRow> for AccessPermission {
    fn from(row: PermissionRow) -> Self {
        AccessPermission::restore(
            PermissionId::from_uuid(row.id),
            AccessGroupId::from_uuid(row.group_id),
            DoorId::from_uuid(row.door_id),
            row.schedule_id.map(ScheduleId::from_uuid),
            row.created_at,
        )
    }
}

impl PermissionRepository for PgPermissionRepository {
    async fn insert(&self, permission: &AccessPermission) -> RepositoryResult<()> {
        sqlx::query(
            "INSERT INTO access_permissions (id, group_id, door_id, schedule_id, created_at)
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(permission.id().as_uuid())
        .bind(permission.group_id().as_uuid())
        .bind(permission.door_id().as_uuid())
        .bind(permission.schedule_id().map(|id| id.as_uuid()))
        .bind(permission.created_at())
        .execute(&self.pool)
        .await
        .map_err(map_error)?;
        Ok(())
    }

    async fn delete(&self, id: PermissionId) -> RepositoryResult<()> {
        let result = sqlx::query("DELETE FROM access_permissions WHERE id = $1")
            .bind(id.as_uuid())
            .execute(&self.pool)
            .await
            .map_err(map_error)?;
        expect_one_row(result, "permission")
    }

    async fn find_by_id(&self, id: PermissionId) -> RepositoryResult<Option<AccessPermission>> {
        let row = sqlx::query_as::<_, PermissionRow>(
            "SELECT id, group_id, door_id, schedule_id, created_at
             FROM access_permissions WHERE id = $1",
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_error)?;
        Ok(row.map(AccessPermission::from))
    }

    async fn list(&self, page: PageRequest) -> RepositoryResult<Vec<AccessPermission>> {
        let rows = sqlx::query_as::<_, PermissionRow>(
            "SELECT id, group_id, door_id, schedule_id, created_at
             FROM access_permissions ORDER BY created_at, id LIMIT $1 OFFSET $2",
        )
        .bind(i64::from(page.limit()))
        .bind(i64::from(page.offset()))
        .fetch_all(&self.pool)
        .await
        .map_err(map_error)?;
        Ok(rows.into_iter().map(AccessPermission::from).collect())
    }
}
