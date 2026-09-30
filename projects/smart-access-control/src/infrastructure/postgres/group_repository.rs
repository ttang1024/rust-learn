use sqlx::PgPool;
use uuid::Uuid;

use super::{expect_one_row, into_entities, map_error};
use crate::{
    application::{AccessGroupRepository, PageRequest, RepositoryResult},
    domain::{AccessGroup, AccessGroupId, Description, DomainError, GroupName, UserId},
};

#[derive(Debug, Clone)]
pub struct PgAccessGroupRepository {
    pool: PgPool,
}

impl PgAccessGroupRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct GroupRow {
    id: Uuid,
    name: String,
    description: Option<String>,
}

impl TryFrom<GroupRow> for AccessGroup {
    type Error = DomainError;

    fn try_from(row: GroupRow) -> Result<Self, Self::Error> {
        Ok(AccessGroup::restore(
            AccessGroupId::from_uuid(row.id),
            GroupName::parse(&row.name)?,
            row.description
                .as_deref()
                .map(Description::parse)
                .transpose()?,
        ))
    }
}

impl AccessGroupRepository for PgAccessGroupRepository {
    async fn insert(&self, group: &AccessGroup) -> RepositoryResult<()> {
        sqlx::query("INSERT INTO access_groups (id, name, description) VALUES ($1, $2, $3)")
            .bind(group.id().as_uuid())
            .bind(group.name().as_str())
            .bind(group.description().map(Description::as_str))
            .execute(&self.pool)
            .await
            .map_err(map_error)?;
        Ok(())
    }

    async fn update(&self, group: &AccessGroup) -> RepositoryResult<()> {
        let result =
            sqlx::query("UPDATE access_groups SET name = $2, description = $3 WHERE id = $1")
                .bind(group.id().as_uuid())
                .bind(group.name().as_str())
                .bind(group.description().map(Description::as_str))
                .execute(&self.pool)
                .await
                .map_err(map_error)?;
        expect_one_row(result, "access_group")
    }

    /// Memberships and permissions go with it (`ON DELETE CASCADE`).
    async fn delete(&self, id: AccessGroupId) -> RepositoryResult<()> {
        let result = sqlx::query("DELETE FROM access_groups WHERE id = $1")
            .bind(id.as_uuid())
            .execute(&self.pool)
            .await
            .map_err(map_error)?;
        expect_one_row(result, "access_group")
    }

    async fn find_by_id(&self, id: AccessGroupId) -> RepositoryResult<Option<AccessGroup>> {
        let row = sqlx::query_as::<_, GroupRow>(
            "SELECT id, name, description FROM access_groups WHERE id = $1",
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_error)?;
        Ok(row.map(AccessGroup::try_from).transpose()?)
    }

    async fn list(&self, page: PageRequest) -> RepositoryResult<Vec<AccessGroup>> {
        let rows = sqlx::query_as::<_, GroupRow>(
            "SELECT id, name, description FROM access_groups
             ORDER BY name, id LIMIT $1 OFFSET $2",
        )
        .bind(i64::from(page.limit()))
        .bind(i64::from(page.offset()))
        .fetch_all(&self.pool)
        .await
        .map_err(map_error)?;
        into_entities(rows)
    }

    async fn add_member(&self, group_id: AccessGroupId, user_id: UserId) -> RepositoryResult<()> {
        sqlx::query("INSERT INTO user_access_groups (user_id, group_id) VALUES ($1, $2)")
            .bind(user_id.as_uuid())
            .bind(group_id.as_uuid())
            .execute(&self.pool)
            .await
            .map_err(map_error)?;
        Ok(())
    }

    async fn remove_member(
        &self,
        group_id: AccessGroupId,
        user_id: UserId,
    ) -> RepositoryResult<()> {
        let result =
            sqlx::query("DELETE FROM user_access_groups WHERE user_id = $1 AND group_id = $2")
                .bind(user_id.as_uuid())
                .bind(group_id.as_uuid())
                .execute(&self.pool)
                .await
                .map_err(map_error)?;
        expect_one_row(result, "membership")
    }

    async fn list_members(&self, group_id: AccessGroupId) -> RepositoryResult<Vec<UserId>> {
        let ids: Vec<Uuid> = sqlx::query_scalar(
            "SELECT user_id FROM user_access_groups WHERE group_id = $1 ORDER BY user_id",
        )
        .bind(group_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(map_error)?;
        Ok(ids.into_iter().map(UserId::from_uuid).collect())
    }
}
