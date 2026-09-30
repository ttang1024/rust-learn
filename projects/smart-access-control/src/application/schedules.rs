//! Access schedule management use cases.

use chrono::{NaiveDate, NaiveTime, Weekday};

use super::{ApplicationError, PageRequest, ScheduleRepository};
use crate::domain::{
    AccessSchedule, DaySet, ScheduleId, ScheduleName, ScheduleRule, parse_time_zone,
};

#[derive(Debug, Clone)]
pub struct RuleInput {
    pub days: Vec<Weekday>,
    pub start: NaiveTime,
    pub end: NaiveTime,
}

#[derive(Debug, Clone)]
pub struct CreateSchedule {
    pub name: String,
    /// IANA name, e.g. `Europe/London`.
    pub timezone: String,
    pub rules: Vec<RuleInput>,
    pub effective_from: Option<NaiveDate>,
    pub effective_until: Option<NaiveDate>,
}

pub struct ScheduleService<S> {
    schedules: S,
}

impl<S: ScheduleRepository> ScheduleService<S> {
    pub fn new(schedules: S) -> Self {
        Self { schedules }
    }

    pub async fn create(&self, cmd: CreateSchedule) -> Result<AccessSchedule, ApplicationError> {
        // `collect` into `Result<Vec<_>, _>` stops at the first invalid rule.
        let rules = cmd
            .rules
            .into_iter()
            .map(|rule| {
                Ok::<_, ApplicationError>(ScheduleRule::new(
                    DaySet::from_days(rule.days)?,
                    rule.start,
                    rule.end,
                ))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let schedule = AccessSchedule::new(
            ScheduleName::parse(&cmd.name)?,
            parse_time_zone(&cmd.timezone)?,
            rules,
            cmd.effective_from,
            cmd.effective_until,
        )?;
        self.schedules.insert(&schedule).await?;
        Ok(schedule)
    }

    pub async fn get(&self, id: ScheduleId) -> Result<AccessSchedule, ApplicationError> {
        self.schedules
            .find_by_id(id)
            .await?
            .ok_or(ApplicationError::NotFound { entity: "schedule" })
    }

    pub async fn list(&self, page: PageRequest) -> Result<Vec<AccessSchedule>, ApplicationError> {
        Ok(self.schedules.list(page).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{application::fakes::InMemorySchedules, domain::DomainError};

    fn service() -> ScheduleService<InMemorySchedules> {
        ScheduleService::new(InMemorySchedules::default())
    }

    fn business_hours() -> CreateSchedule {
        CreateSchedule {
            name: "Business hours".into(),
            timezone: "Europe/London".into(),
            rules: vec![RuleInput {
                days: vec![
                    Weekday::Mon,
                    Weekday::Tue,
                    Weekday::Wed,
                    Weekday::Thu,
                    Weekday::Fri,
                ],
                start: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
                end: NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
            }],
            effective_from: None,
            effective_until: None,
        }
    }

    fn validation_field(err: ApplicationError) -> &'static str {
        match err {
            ApplicationError::Domain(DomainError::Validation { field, .. }) => field,
            other => panic!("expected a validation error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn create_and_get() {
        let service = service();
        let schedule = service.create(business_hours()).await.unwrap();

        assert_eq!(schedule.timezone().name(), "Europe/London");
        assert_eq!(schedule.rules()[0].days(), DaySet::WEEKDAYS);
        assert_eq!(service.get(schedule.id()).await.unwrap(), schedule);
    }

    #[tokio::test]
    async fn rejects_unknown_time_zone() {
        let mut cmd = business_hours();
        cmd.timezone = "Mars/Olympus".into();
        let err = service().create(cmd).await.unwrap_err();
        assert_eq!(validation_field(err), "timezone");
    }

    #[tokio::test]
    async fn rejects_rule_without_days() {
        let mut cmd = business_hours();
        cmd.rules[0].days.clear();
        let err = service().create(cmd).await.unwrap_err();
        assert_eq!(validation_field(err), "days");
    }

    #[tokio::test]
    async fn rejects_schedule_without_rules() {
        let mut cmd = business_hours();
        cmd.rules.clear();
        let err = service().create(cmd).await.unwrap_err();
        assert_eq!(validation_field(err), "rules");
    }

    #[tokio::test]
    async fn unknown_schedule_is_not_found() {
        let err = service().get(ScheduleId::generate()).await.unwrap_err();
        assert!(matches!(
            err,
            ApplicationError::NotFound { entity: "schedule" }
        ));
    }
}
