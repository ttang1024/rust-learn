//! Slows down password guessing.

use std::{collections::HashMap, net::IpAddr, sync::Mutex};

use chrono::Duration;

use crate::{application::Clock, domain::Timestamp};

#[derive(Debug, Clone, Copy)]
pub struct ThrottlePolicy {
    /// Failed logins per account name before that name is blocked.
    pub max_failures_per_username: u32,
    /// Failed logins per client address, across all names ("spraying").
    pub max_failures_per_ip: u32,
    /// How long failures count, and how long a block lasts at most.
    pub window: Duration,
    /// Entries tracked per table. Names are chosen by the attacker, so the
    /// tables must not grow without bound.
    pub max_tracked: usize,
}

impl Default for ThrottlePolicy {
    fn default() -> Self {
        Self {
            max_failures_per_username: 5,
            max_failures_per_ip: 20,
            window: Duration::minutes(15),
            max_tracked: 10_000,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Window {
    started: Timestamp,
    failures: u32,
}

/// In-memory login throttle (per process; state is lost on restart).
///
/// Every account name is treated the same whether or not it exists, so a
/// block never reveals which names are real.
pub struct LoginThrottle {
    clock: std::sync::Arc<dyn Clock>,
    policy: ThrottlePolicy,
    by_username: Mutex<HashMap<String, Window>>,
    by_ip: Mutex<HashMap<IpAddr, Window>>,
}

/// How long until another attempt is allowed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetryAfter(pub Duration);

impl LoginThrottle {
    pub fn new(clock: std::sync::Arc<dyn Clock>, policy: ThrottlePolicy) -> Self {
        Self {
            clock,
            policy,
            by_username: Mutex::default(),
            by_ip: Mutex::default(),
        }
    }

    /// `Err` if this name or address has used up its attempts.
    pub fn check(&self, username: &str, ip: Option<IpAddr>) -> Result<(), RetryAfter> {
        let now = self.clock.now();
        let key = normalise(username);
        blocked(
            &self.by_username,
            &key,
            self.policy.max_failures_per_username,
            self.policy.window,
            now,
        )?;
        if let Some(ip) = ip {
            blocked(
                &self.by_ip,
                &ip,
                self.policy.max_failures_per_ip,
                self.policy.window,
                now,
            )?;
        }
        Ok(())
    }

    pub fn record_failure(&self, username: &str, ip: Option<IpAddr>) {
        let now = self.clock.now();
        count(&self.by_username, normalise(username), &self.policy, now);
        if let Some(ip) = ip {
            count(&self.by_ip, ip, &self.policy, now);
        }
    }

    /// A correct password clears the name's failures (not the address's:
    /// one success must not unlock spraying from that address).
    pub fn record_success(&self, username: &str) {
        lock(&self.by_username).remove(&normalise(username));
    }
}

/// Same normalisation as `Username`, bounded in length, so "Ops" and "ops"
/// share a counter and a huge name cannot bloat memory.
fn normalise(username: &str) -> String {
    username.trim().to_lowercase().chars().take(64).collect()
}

fn lock<K, V>(map: &Mutex<HashMap<K, V>>) -> std::sync::MutexGuard<'_, HashMap<K, V>> {
    // Every update is a single insert or remove: a panic cannot leave the
    // map half-written, so recovering from poisoning is safe.
    map.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn blocked<K: std::hash::Hash + Eq>(
    map: &Mutex<HashMap<K, Window>>,
    key: &K,
    max: u32,
    window: Duration,
    now: Timestamp,
) -> Result<(), RetryAfter> {
    match lock(map).get(key) {
        Some(w) if w.started + window > now && w.failures >= max => {
            Err(RetryAfter(w.started + window - now))
        }
        _ => Ok(()),
    }
}

fn count<K: std::hash::Hash + Eq + Clone>(
    map: &Mutex<HashMap<K, Window>>,
    key: K,
    policy: &ThrottlePolicy,
    now: Timestamp,
) {
    let mut map = lock(map);
    if !map.contains_key(&key) && map.len() >= policy.max_tracked {
        // Make room: first drop expired windows, then the oldest one.
        map.retain(|_, w| w.started + policy.window > now);
        if map.len() >= policy.max_tracked
            && let Some(oldest) = map
                .iter()
                .min_by_key(|(_, w)| w.started)
                .map(|(k, _)| k.clone())
        {
            map.remove(&oldest);
        }
    }
    let entry = map.entry(key).or_insert(Window {
        started: now,
        failures: 0,
    });
    if entry.started + policy.window <= now {
        *entry = Window {
            started: now,
            failures: 0,
        };
    }
    entry.failures += 1;
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use chrono::{TimeZone, Utc};

    use super::*;
    use crate::application::fakes::FixedClock;

    fn setup(policy: ThrottlePolicy) -> (LoginThrottle, Arc<FixedClock>) {
        let clock = FixedClock::at(Utc.with_ymd_and_hms(2026, 9, 30, 10, 0, 0).unwrap());
        (LoginThrottle::new(clock.clone(), policy), clock)
    }

    const IP: Option<IpAddr> = Some(IpAddr::V4(std::net::Ipv4Addr::new(10, 0, 0, 1)));

    #[test]
    fn blocks_a_name_after_five_failures_until_the_window_ends() {
        let (throttle, clock) = setup(ThrottlePolicy::default());
        for _ in 0..5 {
            assert!(throttle.check("ops", None).is_ok());
            throttle.record_failure("ops", None);
        }

        assert_eq!(
            throttle.check("ops", None),
            Err(RetryAfter(Duration::minutes(15)))
        );
        clock.advance(Duration::minutes(10));
        assert_eq!(
            throttle.check("ops", None),
            Err(RetryAfter(Duration::minutes(5)))
        );
        clock.advance(Duration::minutes(5));
        assert!(throttle.check("ops", None).is_ok(), "window over");
    }

    #[test]
    fn names_are_normalised_and_independent() {
        let (throttle, _) = setup(ThrottlePolicy::default());
        for _ in 0..5 {
            throttle.record_failure(" OPS ", None);
        }
        assert!(
            throttle.check("ops", None).is_err(),
            "same name, different case"
        );
        assert!(throttle.check("someone-else", None).is_ok());
    }

    #[test]
    fn success_clears_the_name_but_not_the_address() {
        let (throttle, _) = setup(ThrottlePolicy {
            max_failures_per_ip: 3,
            ..ThrottlePolicy::default()
        });
        for _ in 0..3 {
            throttle.record_failure("ops", IP);
        }
        throttle.record_success("ops");

        // The address is still blocked, so the name check passing is not enough.
        assert!(throttle.check("ops", None).is_ok());
        assert!(throttle.check("ops", IP).is_err());
    }

    #[test]
    fn spraying_many_names_from_one_address_is_blocked() {
        let (throttle, _) = setup(ThrottlePolicy::default());
        for n in 0..20 {
            let name = format!("user{n}");
            assert!(throttle.check(&name, IP).is_ok());
            throttle.record_failure(&name, IP);
        }
        assert!(throttle.check("user99", IP).is_err());
        assert!(
            throttle.check("user99", None).is_ok(),
            "another address is unaffected"
        );
    }

    #[test]
    fn failures_after_an_expired_window_start_counting_again() {
        let (throttle, clock) = setup(ThrottlePolicy::default());
        for _ in 0..4 {
            throttle.record_failure("ops", None);
        }
        clock.advance(Duration::minutes(15));
        throttle.record_failure("ops", None);
        assert!(
            throttle.check("ops", None).is_ok(),
            "only 1 failure in the new window"
        );
    }

    #[test]
    fn memory_is_bounded() {
        let (throttle, _) = setup(ThrottlePolicy {
            max_tracked: 100,
            ..ThrottlePolicy::default()
        });
        for n in 0..1_000 {
            throttle.record_failure(&format!("user{n}"), None);
        }
        assert!(lock(&throttle.by_username).len() <= 100);
    }
}
