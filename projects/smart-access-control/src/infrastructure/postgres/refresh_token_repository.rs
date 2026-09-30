use sqlx::PgPool;
use uuid::Uuid;

use super::map_error;
use crate::{
    application::{
        ConsumeOutcome, RefreshTokenRecord, RefreshTokenRepository, RepositoryResult,
        RevocationReason,
    },
    domain::{AdministratorId, Timestamp},
};

#[derive(Debug, Clone)]
pub struct PgRefreshTokenRepository {
    pool: PgPool,
}

impl PgRefreshTokenRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct TokenRow {
    token_hash: String,
    administrator_id: Uuid,
    created_at: Timestamp,
    expires_at: Timestamp,
}

impl From<TokenRow> for RefreshTokenRecord {
    fn from(row: TokenRow) -> Self {
        Self {
            token_hash: row.token_hash,
            admin_id: AdministratorId::from_uuid(row.administrator_id),
            created_at: row.created_at,
            expires_at: row.expires_at,
        }
    }
}

impl RefreshTokenRepository for PgRefreshTokenRepository {
    async fn insert(&self, record: &RefreshTokenRecord) -> RepositoryResult<()> {
        sqlx::query(
            "INSERT INTO refresh_tokens (token_hash, administrator_id, created_at, expires_at)
             VALUES ($1, $2, $3, $4)",
        )
        .bind(&record.token_hash)
        .bind(record.admin_id.as_uuid())
        .bind(record.created_at)
        .bind(record.expires_at)
        .execute(&self.pool)
        .await
        .map_err(map_error)?;
        Ok(())
    }

    /// One conditional `UPDATE ... RETURNING` does check-and-mark atomically:
    /// Postgres row locking guarantees that of two concurrent requests with
    /// the same token, only one sees `revoked_at IS NULL` and gets the row.
    /// A separate SELECT-then-UPDATE would let both succeed.
    async fn consume(
        &self,
        token_hash: &str,
        reason: RevocationReason,
        now: Timestamp,
    ) -> RepositoryResult<ConsumeOutcome> {
        let consumed = sqlx::query_as::<_, TokenRow>(
            "UPDATE refresh_tokens SET revoked_at = $2, revoked_reason = $3
             WHERE token_hash = $1 AND revoked_at IS NULL
             RETURNING token_hash, administrator_id, created_at, expires_at",
        )
        .bind(token_hash)
        .bind(now)
        .bind(reason.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_error)?;
        if let Some(row) = consumed {
            return Ok(ConsumeOutcome::Consumed(row.into()));
        }

        // Not consumable: find out whether it exists, and why it is dead.
        let previous: Option<(Uuid, Option<String>)> = sqlx::query_as(
            "SELECT administrator_id, revoked_reason FROM refresh_tokens WHERE token_hash = $1",
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_error)?;
        Ok(match previous {
            None => ConsumeOutcome::Unknown,
            Some((admin_id, reason)) if reason.as_deref() == Some("rotated") => {
                ConsumeOutcome::Reused {
                    admin_id: AdministratorId::from_uuid(admin_id),
                }
            }
            Some(_) => ConsumeOutcome::Revoked,
        })
    }

    async fn revoke_all(&self, admin_id: AdministratorId, now: Timestamp) -> RepositoryResult<()> {
        sqlx::query(
            "UPDATE refresh_tokens SET revoked_at = $2, revoked_reason = 'revoked'
             WHERE administrator_id = $1 AND revoked_at IS NULL",
        )
        .bind(admin_id.as_uuid())
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(map_error)?;
        Ok(())
    }
}
