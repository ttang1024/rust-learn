//! Loads the facts for one access decision from a consistent snapshot.

use sqlx::PgPool;
use uuid::Uuid;

use super::{
    card_repository::card_by_number, door_repository::door_by_id, map_error,
    permission_repository::PermissionRow, schedule_repository::schedules_by_ids,
    user_repository::user_by_id,
};
use crate::{
    application::{AccessDataSource, AccessSnapshot, RepositoryResult},
    domain::{AccessGroupId, AccessPermission, CardNumber, DoorId},
};

#[derive(Debug, Clone)]
pub struct PgAccessDataSource {
    pool: PgPool,
}

impl PgAccessDataSource {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl AccessDataSource for PgAccessDataSource {
    /// Runs every query in one `REPEATABLE READ READ ONLY` transaction, so all
    /// facts come from the same committed state. Without it, a permission
    /// could be revoked between reading the group list and the permissions,
    /// and the engine would see a mix of before and after.
    async fn load(
        &self,
        card_number: Option<&CardNumber>,
        door_id: DoorId,
    ) -> RepositoryResult<AccessSnapshot> {
        let mut tx = self
            .pool
            .begin_with("BEGIN ISOLATION LEVEL REPEATABLE READ READ ONLY")
            .await
            .map_err(map_error)?;

        let card = match card_number {
            Some(number) => card_by_number(&mut *tx, number).await?,
            None => None,
        };
        let holder = match &card {
            Some(card) => user_by_id(&mut *tx, card.user_id()).await?,
            None => None,
        };
        let door = door_by_id(&mut *tx, door_id).await?;

        let group_ids: Vec<Uuid> = match &holder {
            Some(holder) => {
                sqlx::query_scalar("SELECT group_id FROM user_access_groups WHERE user_id = $1")
                    .bind(holder.id().as_uuid())
                    .fetch_all(&mut *tx)
                    .await
                    .map_err(map_error)?
            }
            None => Vec::new(),
        };

        // Only permissions that could apply: this door, the holder's groups.
        let permissions: Vec<AccessPermission> = if group_ids.is_empty() || door.is_none() {
            Vec::new()
        } else {
            sqlx::query_as::<_, PermissionRow>(
                "SELECT id, group_id, door_id, schedule_id, created_at
                 FROM access_permissions WHERE door_id = $1 AND group_id = ANY($2)",
            )
            .bind(door_id.as_uuid())
            .bind(group_ids.as_slice())
            .fetch_all(&mut *tx)
            .await
            .map_err(map_error)?
            .into_iter()
            .map(AccessPermission::from)
            .collect()
        };

        let mut schedule_ids: Vec<Uuid> = permissions
            .iter()
            .filter_map(|permission| permission.schedule_id())
            .map(|id| id.as_uuid())
            .collect();
        schedule_ids.sort_unstable();
        schedule_ids.dedup();
        let schedules = if schedule_ids.is_empty() {
            Vec::new()
        } else {
            schedules_by_ids(&mut tx, &schedule_ids).await?
        };

        tx.commit().await.map_err(map_error)?;

        Ok(AccessSnapshot {
            card,
            holder,
            door,
            holder_groups: group_ids
                .into_iter()
                .map(AccessGroupId::from_uuid)
                .collect(),
            permissions,
            schedules,
        })
    }
}
