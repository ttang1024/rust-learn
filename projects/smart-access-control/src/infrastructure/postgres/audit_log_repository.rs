use sqlx::PgPool;
use uuid::Uuid;

use super::{into_entities, map_error};
use crate::{
    application::{AuditLogRepository, PageRequest, RepositoryResult},
    domain::{AdministratorId, AuditEntry, AuditEntryId, AuditSubject, DomainError, Timestamp},
};

#[derive(Debug, Clone)]
pub struct PgAuditLogRepository {
    pool: PgPool,
}

impl PgAuditLogRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct AuditRow {
    id: Uuid,
    administrator_id: Option<Uuid>,
    action: String,
    subject: Option<String>,
    occurred_at: Timestamp,
}

impl TryFrom<AuditRow> for AuditEntry {
    type Error = DomainError;

    fn try_from(row: AuditRow) -> Result<Self, Self::Error> {
        Ok(AuditEntry::restore(
            AuditEntryId::from_uuid(row.id),
            row.administrator_id.map(AdministratorId::from_uuid),
            row.action.parse()?,
            row.subject
                .as_deref()
                .map(AuditSubject::parse)
                .transpose()?,
            row.occurred_at,
        ))
    }
}

impl AuditLogRepository for PgAuditLogRepository {
    async fn append(&self, entry: &AuditEntry) -> RepositoryResult<()> {
        sqlx::query(
            "INSERT INTO admin_audit_log (id, administrator_id, action, subject, occurred_at)
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(entry.id().as_uuid())
        .bind(entry.actor().map(|id| id.as_uuid()))
        .bind(entry.action().as_str())
        .bind(entry.subject().map(AuditSubject::as_str))
        .bind(entry.occurred_at())
        .execute(&self.pool)
        .await
        .map_err(map_error)?;
        Ok(())
    }

    async fn list_recent(&self, page: PageRequest) -> RepositoryResult<Vec<AuditEntry>> {
        let rows = sqlx::query_as::<_, AuditRow>(
            "SELECT id, administrator_id, action, subject, occurred_at
             FROM admin_audit_log ORDER BY occurred_at DESC, id DESC LIMIT $1 OFFSET $2",
        )
        .bind(i64::from(page.limit()))
        .bind(i64::from(page.offset()))
        .fetch_all(&self.pool)
        .await
        .map_err(map_error)?;
        into_entities(rows)
    }
}
