//! When access is allowed: weekly time windows in an explicit IANA time zone.
//!
//! Windows are local wall-clock times ("09:00–17:00 in New York"). Checking an
//! instant converts it from UTC to local time, which is always unambiguous,
//! even across daylight saving changes (the reverse conversion is not).

use chrono::{Datelike, NaiveDate, NaiveTime, Weekday};
use chrono_tz::Tz;

use super::{DomainError, ScheduleId, Timestamp, text::bounded_text};

bounded_text!(
    /// A schedule's display name, e.g. "Business hours".
    ScheduleName,
    field = "name",
    max = 100
);

/// Parses an IANA time zone name such as `Europe/London`.
pub fn parse_time_zone(raw: &str) -> Result<Tz, DomainError> {
    raw.trim().parse().map_err(|_| DomainError::Validation {
        field: "timezone",
        reason: "must be an IANA time zone name, e.g. Europe/London",
    })
}

/// A non-empty set of weekdays, stored as a 7-bit mask
/// (bit 0 = Monday … bit 6 = Sunday).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DaySet(u8);

impl DaySet {
    pub const WEEKDAYS: Self = Self(0b001_1111);
    pub const WEEKEND: Self = Self(0b110_0000);
    pub const EVERY_DAY: Self = Self(0b111_1111);

    pub fn from_days(days: impl IntoIterator<Item = Weekday>) -> Result<Self, DomainError> {
        let bits = days.into_iter().fold(0, |bits, day| bits | Self::bit(day));
        Self::from_bits(bits)
    }

    /// Rebuilds a set from its mask, e.g. when loading from storage.
    pub fn from_bits(bits: u8) -> Result<Self, DomainError> {
        let invalid = |reason| DomainError::Validation {
            field: "days",
            reason,
        };
        if bits == 0 {
            return Err(invalid("must contain at least one weekday"));
        }
        if bits > Self::EVERY_DAY.0 {
            return Err(invalid("contains an invalid weekday bit"));
        }
        Ok(Self(bits))
    }

    pub const fn bits(self) -> u8 {
        self.0
    }

    pub fn contains(self, day: Weekday) -> bool {
        self.0 & Self::bit(day) != 0
    }

    fn bit(day: Weekday) -> u8 {
        1 << day.num_days_from_monday()
    }
}

/// One weekly window, e.g. "Mon–Fri 09:00–17:00".
///
/// The window is half-open, `[start, end)`: 17:00 itself is outside
/// "09:00–17:00". If `end <= start` the window closes on the *next* day, so
/// "22:00–06:00" is a night shift and `start == end` is a full 24 hours.
/// `days` are the days on which the window *opens*.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScheduleRule {
    days: DaySet,
    start: NaiveTime,
    end: NaiveTime,
}

impl ScheduleRule {
    pub fn new(days: DaySet, start: NaiveTime, end: NaiveTime) -> Self {
        Self { days, start, end }
    }

    pub fn days(&self) -> DaySet {
        self.days
    }

    pub fn start(&self) -> NaiveTime {
        self.start
    }

    pub fn end(&self) -> NaiveTime {
        self.end
    }

    pub fn crosses_midnight(&self) -> bool {
        self.end <= self.start
    }

    /// Whether the local weekday and time fall inside this window.
    fn contains(&self, day: Weekday, time: NaiveTime) -> bool {
        if self.crosses_midnight() {
            // Either the window opened today, or it opened yesterday and has
            // not closed yet.
            (self.days.contains(day) && time >= self.start)
                || (self.days.contains(day.pred()) && time < self.end)
        } else {
            self.days.contains(day) && self.start <= time && time < self.end
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessSchedule {
    id: ScheduleId,
    name: ScheduleName,
    timezone: Tz,
    rules: Vec<ScheduleRule>,
    /// First local date on which the schedule applies (inclusive).
    effective_from: Option<NaiveDate>,
    /// Last local date on which the schedule applies (inclusive).
    effective_until: Option<NaiveDate>,
}

impl AccessSchedule {
    pub const MAX_RULES: usize = 32;

    pub fn new(
        name: ScheduleName,
        timezone: Tz,
        rules: Vec<ScheduleRule>,
        effective_from: Option<NaiveDate>,
        effective_until: Option<NaiveDate>,
    ) -> Result<Self, DomainError> {
        if rules.is_empty() {
            return Err(DomainError::Validation {
                field: "rules",
                reason: "must contain at least one rule",
            });
        }
        if rules.len() > Self::MAX_RULES {
            return Err(DomainError::Validation {
                field: "rules",
                reason: "must contain at most 32 rules",
            });
        }
        if let (Some(from), Some(until)) = (effective_from, effective_until)
            && until < from
        {
            return Err(DomainError::Validation {
                field: "effective_until",
                reason: "must not be before effective_from",
            });
        }

        Ok(Self {
            id: ScheduleId::generate(),
            name,
            timezone,
            rules,
            effective_from,
            effective_until,
        })
    }

    /// Rebuilds a schedule from stored values. For repositories only.
    pub fn restore(
        id: ScheduleId,
        name: ScheduleName,
        timezone: Tz,
        rules: Vec<ScheduleRule>,
        effective_from: Option<NaiveDate>,
        effective_until: Option<NaiveDate>,
    ) -> Self {
        Self {
            id,
            name,
            timezone,
            rules,
            effective_from,
            effective_until,
        }
    }

    pub fn id(&self) -> ScheduleId {
        self.id
    }

    pub fn name(&self) -> &ScheduleName {
        &self.name
    }

    pub fn timezone(&self) -> Tz {
        self.timezone
    }

    /// Borrowed as a slice: callers can iterate without taking the `Vec`.
    pub fn rules(&self) -> &[ScheduleRule] {
        &self.rules
    }

    pub fn effective_from(&self) -> Option<NaiveDate> {
        self.effective_from
    }

    pub fn effective_until(&self) -> Option<NaiveDate> {
        self.effective_until
    }

    /// Whether this schedule allows access at `instant`.
    pub fn allows(&self, instant: Timestamp) -> bool {
        let local = instant.with_timezone(&self.timezone);

        let date = local.date_naive();
        let before_start = self.effective_from.is_some_and(|from| date < from);
        let after_end = self.effective_until.is_some_and(|until| date > until);
        if before_start || after_end {
            return false;
        }

        let (day, time) = (local.weekday(), local.time());
        self.rules.iter().any(|rule| rule.contains(day, time))
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use chrono_tz::{America::New_York, Asia::Tokyo, UTC};

    use super::*;

    fn utc(y: i32, m: u32, d: u32, h: u32, min: u32) -> Timestamp {
        Utc.with_ymd_and_hms(y, m, d, h, min, 0).unwrap()
    }

    fn t(h: u32, m: u32) -> NaiveTime {
        NaiveTime::from_hms_opt(h, m, 0).unwrap()
    }

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn schedule(tz: Tz, rules: Vec<ScheduleRule>) -> AccessSchedule {
        AccessSchedule::new(ScheduleName::parse("Test").unwrap(), tz, rules, None, None).unwrap()
    }

    fn business_hours(tz: Tz) -> AccessSchedule {
        schedule(
            tz,
            vec![ScheduleRule::new(DaySet::WEEKDAYS, t(9, 0), t(17, 0))],
        )
    }

    // 2026-09-30 is a Wednesday. New York is on EDT (UTC-4) then.

    #[test]
    fn window_start_is_inclusive_and_end_exclusive() {
        let s = business_hours(New_York);
        assert!(!s.allows(utc(2026, 9, 30, 12, 59))); // 08:59 local
        assert!(s.allows(utc(2026, 9, 30, 13, 0))); //   09:00
        assert!(s.allows(utc(2026, 9, 30, 20, 59))); //  16:59
        assert!(!s.allows(utc(2026, 9, 30, 21, 0))); //  17:00
    }

    #[test]
    fn days_outside_the_set_are_denied() {
        let s = business_hours(New_York);
        // Saturday 2026-10-03, 11:00 local.
        assert!(!s.allows(utc(2026, 10, 3, 15, 0)));
    }

    #[test]
    fn same_instant_differs_by_time_zone() {
        let instant = utc(2026, 9, 30, 13, 0); // Wed 13:00 UTC = Wed 22:00 Tokyo
        assert!(business_hours(UTC).allows(instant));
        assert!(!business_hours(Tokyo).allows(instant));
    }

    #[test]
    fn local_weekday_is_used_not_utc_weekday() {
        let all_day_weekdays = |tz| {
            schedule(
                tz,
                vec![ScheduleRule::new(DaySet::WEEKDAYS, t(0, 0), t(0, 0))],
            )
        };
        // Friday 2026-10-02 20:00 UTC is already Saturday 05:00 in Tokyo.
        let instant = utc(2026, 10, 2, 20, 0);
        assert!(all_day_weekdays(UTC).allows(instant));
        assert!(!all_day_weekdays(Tokyo).allows(instant));
    }

    #[test]
    fn overnight_window_belongs_to_the_day_it_opens() {
        let s = schedule(
            UTC,
            vec![ScheduleRule::new(DaySet::WEEKDAYS, t(22, 0), t(6, 0))],
        );
        assert!(s.allows(utc(2026, 10, 2, 23, 0))); //  Fri 23:00
        assert!(s.allows(utc(2026, 10, 3, 5, 59))); //  Sat 05:59, opened Fri
        assert!(!s.allows(utc(2026, 10, 3, 6, 0))); //  Sat 06:00, closed
        assert!(!s.allows(utc(2026, 10, 3, 23, 0))); // Sat 23:00, Sat not in set
        assert!(!s.allows(utc(2026, 10, 5, 5, 0))); //  Mon 05:00, Sun not in set
        assert!(s.allows(utc(2026, 10, 5, 22, 0))); //  Mon 22:00
    }

    #[test]
    fn equal_start_and_end_is_a_full_day() {
        let wednesday = DaySet::from_days([Weekday::Wed]).unwrap();
        let s = schedule(UTC, vec![ScheduleRule::new(wednesday, t(0, 0), t(0, 0))]);
        assert!(!s.allows(utc(2026, 9, 29, 23, 59))); // Tue
        assert!(s.allows(utc(2026, 9, 30, 0, 0))); //    Wed 00:00
        assert!(s.allows(Utc.with_ymd_and_hms(2026, 9, 30, 23, 59, 59).unwrap()));
        assert!(!s.allows(utc(2026, 10, 1, 0, 0))); //   Thu 00:00
    }

    #[test]
    fn any_rule_may_match() {
        let s = schedule(
            UTC,
            vec![
                ScheduleRule::new(DaySet::WEEKDAYS, t(9, 0), t(12, 0)),
                ScheduleRule::new(DaySet::WEEKDAYS, t(13, 0), t(17, 0)),
            ],
        );
        assert!(s.allows(utc(2026, 9, 30, 10, 0)));
        assert!(!s.allows(utc(2026, 9, 30, 12, 30))); // lunch break
        assert!(s.allows(utc(2026, 9, 30, 14, 0)));
    }

    #[test]
    fn effective_dates_are_inclusive() {
        let s = AccessSchedule::new(
            ScheduleName::parse("October").unwrap(),
            UTC,
            vec![ScheduleRule::new(DaySet::EVERY_DAY, t(0, 0), t(0, 0))],
            Some(date(2026, 10, 1)),
            Some(date(2026, 10, 31)),
        )
        .unwrap();
        assert!(!s.allows(utc(2026, 9, 30, 23, 59)));
        assert!(s.allows(utc(2026, 10, 1, 0, 0)));
        assert!(s.allows(utc(2026, 10, 31, 23, 59)));
        assert!(!s.allows(utc(2026, 11, 1, 0, 0)));
    }

    #[test]
    fn effective_dates_use_the_local_date() {
        let s = AccessSchedule::new(
            ScheduleName::parse("From October").unwrap(),
            New_York,
            vec![ScheduleRule::new(DaySet::EVERY_DAY, t(0, 0), t(0, 0))],
            Some(date(2026, 10, 1)),
            None,
        )
        .unwrap();
        // 2026-10-01 02:00 UTC is still 30 September (22:00) in New York.
        assert!(!s.allows(utc(2026, 10, 1, 2, 0)));
        assert!(s.allows(utc(2026, 10, 1, 4, 0)));
    }

    #[test]
    fn daylight_saving_start_uses_wall_clock_time() {
        // 2026-03-08: New York clocks jump from 02:00 EST to 03:00 EDT.
        let sunday = DaySet::from_days([Weekday::Sun]).unwrap();
        let s = schedule(New_York, vec![ScheduleRule::new(sunday, t(1, 0), t(3, 0))]);
        assert!(s.allows(utc(2026, 3, 8, 6, 30))); //  01:30 EST
        assert!(!s.allows(utc(2026, 3, 8, 7, 0))); //  03:00 EDT: window over
    }

    #[test]
    fn daylight_saving_end_repeats_the_window() {
        // 2026-11-01: New York clocks fall back from 02:00 EDT to 01:00 EST,
        // so 01:00–02:00 happens twice and both occurrences are inside.
        let sunday = DaySet::from_days([Weekday::Sun]).unwrap();
        let s = schedule(New_York, vec![ScheduleRule::new(sunday, t(1, 0), t(2, 0))]);
        assert!(s.allows(utc(2026, 11, 1, 5, 30))); //  01:30 EDT
        assert!(s.allows(utc(2026, 11, 1, 6, 30))); //  01:30 EST
        assert!(!s.allows(utc(2026, 11, 1, 7, 0))); //  02:00 EST
    }

    #[test]
    fn validation() {
        let name = || ScheduleName::parse("Test").unwrap();
        let rule = ScheduleRule::new(DaySet::WEEKDAYS, t(9, 0), t(17, 0));

        assert!(AccessSchedule::new(name(), UTC, vec![], None, None).is_err());
        assert!(AccessSchedule::new(name(), UTC, vec![rule; 33], None, None).is_err());
        assert!(
            AccessSchedule::new(
                name(),
                UTC,
                vec![rule],
                Some(date(2026, 10, 2)),
                Some(date(2026, 10, 1))
            )
            .is_err()
        );
        // A single-day range is fine.
        assert!(
            AccessSchedule::new(
                name(),
                UTC,
                vec![rule],
                Some(date(2026, 10, 1)),
                Some(date(2026, 10, 1))
            )
            .is_ok()
        );
    }

    #[test]
    fn time_zone_parsing() {
        assert_eq!(
            parse_time_zone(" Europe/London ").unwrap().name(),
            "Europe/London"
        );
        for raw in ["", "Mars/Olympus", "EST5EDT/nope"] {
            assert!(parse_time_zone(raw).is_err(), "{raw:?}");
        }
    }

    #[test]
    fn day_set_bits() {
        let set = DaySet::from_days([Weekday::Mon, Weekday::Sun]).unwrap();
        assert_eq!(set.bits(), 0b100_0001);
        assert!(set.contains(Weekday::Mon) && set.contains(Weekday::Sun));
        assert!(!set.contains(Weekday::Wed));
        assert_eq!(DaySet::from_bits(set.bits()), Ok(set));

        assert!(DaySet::from_days([]).is_err());
        assert!(DaySet::from_bits(0).is_err());
        assert!(DaySet::from_bits(0b1000_0000).is_err());
    }
}
