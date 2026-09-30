use std::collections::HashMap;

use chrono::{NaiveDate, NaiveTime};
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use super::map_error;
use crate::{
    application::{PageRequest, RepositoryResult, ScheduleRepository},
    domain::{
        AccessSchedule, DaySet, DomainError, ScheduleId, ScheduleName, ScheduleRule,
        parse_time_zone,
    },
};

#[derive(Debug, Clone)]
pub struct PgScheduleRepository {
    pool: PgPool,
}

impl PgScheduleRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct ScheduleRow {
    id: Uuid,
    name: String,
    timezone: String,
    effective_from: Option<NaiveDate>,
    effective_until: Option<NaiveDate>,
}

#[derive(sqlx::FromRow)]
struct RuleRow {
    schedule_id: Uuid,
    days: i16,
    start_time: NaiveTime,
    end_time: NaiveTime,
}

impl TryFrom<RuleRow> for ScheduleRule {
    type Error = DomainError;

    fn try_from(row: RuleRow) -> Result<Self, Self::Error> {
        // SMALLINT -> u8: out-of-range values are corrupt data, not a panic.
        let bits = u8::try_from(row.days).map_err(|_| DomainError::Validation {
            field: "days",
            reason: "contains an invalid weekday bit",
        })?;
        Ok(ScheduleRule::new(
            DaySet::from_bits(bits)?,
            row.start_time,
            row.end_time,
        ))
    }
}

/// Loads schedules by ID, including their rules.
///
/// Takes `&mut PgConnection` so callers can run it on a pooled connection or
/// inside a transaction (`&mut *tx`); `PgAccessDataSource` relies on that.
pub(super) async fn schedules_by_ids(
    conn: &mut PgConnection,
    ids: &[Uuid],
) -> RepositoryResult<Vec<AccessSchedule>> {
    let rows = sqlx::query_as::<_, ScheduleRow>(
        "SELECT id, name, timezone, effective_from, effective_until
         FROM access_schedules WHERE id = ANY($1) ORDER BY name, id",
    )
    .bind(ids)
    .fetch_all(&mut *conn)
    .await
    .map_err(map_error)?;
    with_rules(conn, rows).await
}

/// Attaches rules to schedule rows with one extra query (not one per schedule).
async fn with_rules(
    conn: &mut PgConnection,
    rows: Vec<ScheduleRow>,
) -> RepositoryResult<Vec<AccessSchedule>> {
    let ids: Vec<Uuid> = rows.iter().map(|row| row.id).collect();
    let rule_rows = sqlx::query_as::<_, RuleRow>(
        "SELECT schedule_id, days, start_time, end_time
         FROM access_schedule_rules WHERE schedule_id = ANY($1)
         ORDER BY schedule_id, position",
    )
    .bind(ids.as_slice())
    .fetch_all(&mut *conn)
    .await
    .map_err(map_error)?;

    let mut rules: HashMap<Uuid, Vec<ScheduleRule>> = HashMap::new();
    for row in rule_rows {
        let schedule_id = row.schedule_id;
        rules
            .entry(schedule_id)
            .or_default()
            .push(ScheduleRule::try_from(row)?);
    }

    rows.into_iter()
        .map(|row| {
            Ok(AccessSchedule::restore(
                ScheduleId::from_uuid(row.id),
                ScheduleName::parse(&row.name)?,
                parse_time_zone(&row.timezone)?,
                // `remove` moves the Vec out of the map instead of cloning it.
                rules.remove(&row.id).unwrap_or_default(),
                row.effective_from,
                row.effective_until,
            ))
        })
        .collect()
}

impl ScheduleRepository for PgScheduleRepository {
    /// Schedule and rules are written in one transaction: either all rows
    /// exist afterwards or none do.
    async fn insert(&self, schedule: &AccessSchedule) -> RepositoryResult<()> {
        let mut tx = self.pool.begin().await.map_err(map_error)?;

        sqlx::query(
            "INSERT INTO access_schedules (id, name, timezone, effective_from, effective_until)
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(schedule.id().as_uuid())
        .bind(schedule.name().as_str())
        .bind(schedule.timezone().name())
        .bind(schedule.effective_from())
        .bind(schedule.effective_until())
        .execute(&mut *tx)
        .await
        .map_err(map_error)?;

        for (position, rule) in schedule.rules().iter().enumerate() {
            // At most MAX_RULES (32) rules, so the position always fits.
            let position = i16::try_from(position).expect("rule count is bounded");
            sqlx::query(
                "INSERT INTO access_schedule_rules (schedule_id, position, days, start_time, end_time)
                 VALUES ($1, $2, $3, $4, $5)",
            )
            .bind(schedule.id().as_uuid())
            .bind(position)
            .bind(i16::from(rule.days().bits()))
            .bind(rule.start())
            .bind(rule.end())
            .execute(&mut *tx)
            .await
            .map_err(map_error)?;
        }

        // If any `?` above returned early, `tx` is dropped without commit and
        // SQLx rolls the transaction back: RAII cleanup, no explicit rollback.
        tx.commit().await.map_err(map_error)?;
        Ok(())
    }

    async fn find_by_id(&self, id: ScheduleId) -> RepositoryResult<Option<AccessSchedule>> {
        let mut conn = self.pool.acquire().await.map_err(map_error)?;
        let mut found = schedules_by_ids(&mut conn, &[id.as_uuid()]).await?;
        Ok(found.pop())
    }

    async fn list(&self, page: PageRequest) -> RepositoryResult<Vec<AccessSchedule>> {
        let mut conn = self.pool.acquire().await.map_err(map_error)?;
        let rows = sqlx::query_as::<_, ScheduleRow>(
            "SELECT id, name, timezone, effective_from, effective_until
             FROM access_schedules ORDER BY name, id LIMIT $1 OFFSET $2",
        )
        .bind(i64::from(page.limit()))
        .bind(i64::from(page.offset()))
        .fetch_all(&mut *conn)
        .await
        .map_err(map_error)?;
        with_rules(&mut conn, rows).await
    }
}
