//! The real, system-backed clock.

use chrono::{SubsecRound, Utc};

use crate::{application::Clock, domain::Timestamp};

#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Timestamp {
        to_storage_precision(Utc::now())
    }
}

/// Rounds down to whole microseconds.
///
/// PostgreSQL `TIMESTAMPTZ` stores microseconds, while Linux clocks report
/// nanoseconds. Truncating up front means an entity reads back from the
/// database exactly as it was saved. (macOS clocks happen to report whole
/// microseconds already, which is why this is tested with a fixed value.)
fn to_storage_precision(time: Timestamp) -> Timestamp {
    time.trunc_subsecs(6)
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Timelike};

    use super::*;

    #[test]
    fn truncates_to_microseconds() {
        let precise = Utc
            .with_ymd_and_hms(2026, 9, 30, 9, 0, 0)
            .unwrap()
            .with_nanosecond(123_456_789)
            .unwrap();

        let stored = to_storage_precision(precise);

        assert_eq!(stored.nanosecond(), 123_456_000);
        assert_eq!(stored.timestamp(), precise.timestamp());
    }

    #[test]
    fn system_clock_output_is_storage_precision() {
        let now = SystemClock.now();
        assert_eq!(now, to_storage_precision(now));
    }
}
