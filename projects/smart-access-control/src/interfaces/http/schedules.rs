//! `/schedules`: weekly time windows in an explicit time zone.

use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use chrono::{NaiveDate, NaiveTime, Timelike, Weekday};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    ApiError, ApiJson, ApiPath, ApiQuery, AppState, AuthenticatedAdmin, Page, PageParams,
    RequireAdmin, subject,
};
use crate::{
    application::{CreateSchedule, PageRequest, RuleInput},
    domain::{AccessSchedule, AuditAction, DomainError, ScheduleId, ScheduleRule},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/schedules", get(list).post(create))
        .route("/schedules/{id}", get(get_one))
}

const WEEK: [Weekday; 7] = [
    Weekday::Mon,
    Weekday::Tue,
    Weekday::Wed,
    Weekday::Thu,
    Weekday::Fri,
    Weekday::Sat,
    Weekday::Sun,
];

/// One window: `{"days": ["mon", "tue"], "start": "09:00", "end": "17:00"}`.
/// `end` at or before `start` means the window ends the next day.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuleDto {
    pub days: Vec<String>,
    pub start: String,
    pub end: String,
}

impl From<&ScheduleRule> for RuleDto {
    fn from(rule: &ScheduleRule) -> Self {
        Self {
            days: WEEK
                .into_iter()
                .filter(|day| rule.days().contains(*day))
                .map(|day| day.to_string().to_lowercase())
                .collect(),
            start: format_time(rule.start()),
            end: format_time(rule.end()),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ScheduleResponse {
    pub id: Uuid,
    pub name: String,
    pub timezone: &'static str,
    pub rules: Vec<RuleDto>,
    pub effective_from: Option<NaiveDate>,
    pub effective_until: Option<NaiveDate>,
}

impl From<&AccessSchedule> for ScheduleResponse {
    fn from(schedule: &AccessSchedule) -> Self {
        Self {
            id: schedule.id().as_uuid(),
            name: schedule.name().to_string(),
            timezone: schedule.timezone().name(),
            rules: schedule.rules().iter().map(RuleDto::from).collect(),
            effective_from: schedule.effective_from(),
            effective_until: schedule.effective_until(),
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateScheduleRequest {
    pub name: String,
    /// IANA name, e.g. `Europe/London`.
    pub timezone: String,
    pub rules: Vec<RuleDto>,
    /// `YYYY-MM-DD`, local to `timezone`, inclusive.
    pub effective_from: Option<NaiveDate>,
    pub effective_until: Option<NaiveDate>,
}

/// `HH:MM`, or `HH:MM:SS` when seconds are set.
fn format_time(time: NaiveTime) -> String {
    let format = if time.second() == 0 {
        "%H:%M"
    } else {
        "%H:%M:%S"
    };
    time.format(format).to_string()
}

fn parse_time(field: &'static str, raw: &str) -> Result<NaiveTime, DomainError> {
    NaiveTime::parse_from_str(raw, "%H:%M")
        .or_else(|_| NaiveTime::parse_from_str(raw, "%H:%M:%S"))
        .map_err(|_| DomainError::Validation {
            field,
            reason: "must be a time like 09:00 or 09:00:00",
        })
}

fn parse_rule(rule: RuleDto) -> Result<RuleInput, DomainError> {
    let days = rule
        .days
        .iter()
        .map(|day| {
            day.parse::<Weekday>().map_err(|_| DomainError::Validation {
                field: "days",
                reason: "must be weekday names like mon, tue, ... sun",
            })
        })
        .collect::<Result<_, _>>()?;
    Ok(RuleInput {
        days,
        start: parse_time("start", &rule.start)?,
        end: parse_time("end", &rule.end)?,
    })
}

async fn list(
    _: AuthenticatedAdmin,
    State(state): State<AppState>,
    ApiQuery(params): ApiQuery<PageParams>,
) -> Result<Json<Page<ScheduleResponse>>, ApiError> {
    let page = PageRequest::from(params);
    let schedules = state.schedules.list(page).await?;
    Ok(Json(Page::of(&schedules, page)))
}

async fn get_one(
    _: AuthenticatedAdmin,
    State(state): State<AppState>,
    ApiPath(id): ApiPath<Uuid>,
) -> Result<Json<ScheduleResponse>, ApiError> {
    let schedule = state.schedules.get(ScheduleId::from_uuid(id)).await?;
    Ok(Json(ScheduleResponse::from(&schedule)))
}

async fn create(
    RequireAdmin(admin): RequireAdmin,
    State(state): State<AppState>,
    ApiJson(body): ApiJson<CreateScheduleRequest>,
) -> Result<(StatusCode, Json<ScheduleResponse>), ApiError> {
    let rules = body
        .rules
        .into_iter()
        .map(parse_rule)
        .collect::<Result<_, _>>()?;
    let schedule = state
        .schedules
        .create(CreateSchedule {
            name: body.name,
            timezone: body.timezone,
            rules,
            effective_from: body.effective_from,
            effective_until: body.effective_until,
        })
        .await?;
    state
        .record_audit(
            admin,
            AuditAction::ScheduleCreated,
            &subject("schedule", schedule.id()),
        )
        .await;
    Ok((StatusCode::CREATED, Json(ScheduleResponse::from(&schedule))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn times_parse_and_format() {
        assert_eq!(
            parse_time("start", "09:00").unwrap(),
            NaiveTime::from_hms_opt(9, 0, 0).unwrap()
        );
        assert_eq!(
            parse_time("start", "23:59:30").unwrap(),
            NaiveTime::from_hms_opt(23, 59, 30).unwrap()
        );
        for bad in ["9", "24:00", "09:60", "noon", ""] {
            assert!(parse_time("start", bad).is_err(), "{bad:?}");
        }
        assert_eq!(
            format_time(NaiveTime::from_hms_opt(9, 0, 0).unwrap()),
            "09:00"
        );
        assert_eq!(
            format_time(NaiveTime::from_hms_opt(9, 0, 5).unwrap()),
            "09:00:05"
        );
    }

    #[test]
    fn rule_round_trips_through_dto() {
        let input = RuleDto {
            days: vec!["Mon".into(), "friday".into()],
            start: "22:00".into(),
            end: "06:00".into(),
        };
        let parsed = parse_rule(input).unwrap();
        let rule = ScheduleRule::new(
            crate::domain::DaySet::from_days(parsed.days).unwrap(),
            parsed.start,
            parsed.end,
        );
        let dto = RuleDto::from(&rule);
        assert_eq!(dto.days, ["mon", "fri"]);
        assert_eq!((dto.start.as_str(), dto.end.as_str()), ("22:00", "06:00"));
    }
}
