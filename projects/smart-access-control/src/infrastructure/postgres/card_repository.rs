use sqlx::PgPool;
use uuid::Uuid;

use super::{expect_one_row, into_entities, map_error};
use crate::{
    application::{CardRepository, PageRequest, RepositoryResult},
    domain::{AccessCard, CardId, CardNumber, DomainError, Timestamp, UserId},
};

#[derive(Debug, Clone)]
pub struct PgCardRepository {
    pool: PgPool,
}

impl PgCardRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
pub(super) struct CardRow {
    id: Uuid,
    user_id: Uuid,
    card_number: String,
    status: String,
    issued_at: Timestamp,
    expires_at: Option<Timestamp>,
}

impl TryFrom<CardRow> for AccessCard {
    type Error = DomainError;

    fn try_from(row: CardRow) -> Result<Self, Self::Error> {
        Ok(AccessCard::restore(
            CardId::from_uuid(row.id),
            UserId::from_uuid(row.user_id),
            CardNumber::parse(&row.card_number)?,
            row.status.parse()?,
            row.issued_at,
            row.expires_at,
        ))
    }
}

impl CardRepository for PgCardRepository {
    async fn insert(&self, card: &AccessCard) -> RepositoryResult<()> {
        sqlx::query(
            "INSERT INTO access_cards (id, user_id, card_number, status, issued_at, expires_at)
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(card.id().as_uuid())
        .bind(card.user_id().as_uuid())
        .bind(card.card_number().as_str())
        .bind(card.status().as_str())
        .bind(card.issued_at())
        .bind(card.expires_at())
        .execute(&self.pool)
        .await
        .map_err(map_error)?;
        Ok(())
    }

    /// Only the status can change after issue; number, owner and dates are fixed.
    async fn update(&self, card: &AccessCard) -> RepositoryResult<()> {
        let result = sqlx::query("UPDATE access_cards SET status = $2 WHERE id = $1")
            .bind(card.id().as_uuid())
            .bind(card.status().as_str())
            .execute(&self.pool)
            .await
            .map_err(map_error)?;
        expect_one_row(result, "card")
    }

    async fn find_by_id(&self, id: CardId) -> RepositoryResult<Option<AccessCard>> {
        let row = sqlx::query_as::<_, CardRow>(
            "SELECT id, user_id, card_number, status, issued_at, expires_at
             FROM access_cards WHERE id = $1",
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_error)?;
        Ok(row.map(AccessCard::try_from).transpose()?)
    }

    async fn find_by_number(&self, number: &CardNumber) -> RepositoryResult<Option<AccessCard>> {
        let row = sqlx::query_as::<_, CardRow>(
            "SELECT id, user_id, card_number, status, issued_at, expires_at
             FROM access_cards WHERE card_number = $1",
        )
        .bind(number.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_error)?;
        Ok(row.map(AccessCard::try_from).transpose()?)
    }

    async fn list(&self, page: PageRequest) -> RepositoryResult<Vec<AccessCard>> {
        let rows = sqlx::query_as::<_, CardRow>(
            "SELECT id, user_id, card_number, status, issued_at, expires_at
             FROM access_cards ORDER BY issued_at, id LIMIT $1 OFFSET $2",
        )
        .bind(i64::from(page.limit()))
        .bind(i64::from(page.offset()))
        .fetch_all(&self.pool)
        .await
        .map_err(map_error)?;
        into_entities(rows)
    }

    async fn list_for_user(&self, user_id: UserId) -> RepositoryResult<Vec<AccessCard>> {
        let rows = sqlx::query_as::<_, CardRow>(
            "SELECT id, user_id, card_number, status, issued_at, expires_at
             FROM access_cards WHERE user_id = $1 ORDER BY issued_at, id",
        )
        .bind(user_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(map_error)?;
        into_entities(rows)
    }
}
