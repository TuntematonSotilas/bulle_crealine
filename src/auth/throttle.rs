//! Rate limiting of login attempts, per IP address.
//!
//! Argon2 already slows a brute-force attack down considerably, but nothing
//! would stop it from being run in parallel. An in-memory counter is enough
//! here: the site runs on a single instance, and losing the counters on a
//! restart is harmless since the password itself stays out of reach.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

/// How many consecutive failures are tolerated before locking out.
const MAX_FAILURES: u32 = 5;

/// How long a lockout lasts, and after how long an idle counter is forgotten.
const LOCKOUT: Duration = Duration::from_secs(15 * 60);

/// Past this size, stale counters are pruned: without it, requests forging their
/// origin address would grow the table without bound.
const PRUNE_THRESHOLD: usize = 1024;

static FAILURES: OnceLock<Mutex<HashMap<String, Attempts>>> = OnceLock::new();

#[derive(Debug)]
struct Attempts {
    count: u32,
    last: Instant,
}

/// Checks whether `ip` is still allowed to try.
///
/// Returns the remaining wait when a lockout is under way.
pub fn check(ip: &str) -> Result<(), Duration> {
    let mut failures = lock();
    match failures.get(ip) {
        Some(attempts) if attempts.count >= MAX_FAILURES => {
            match LOCKOUT.checked_sub(attempts.last.elapsed()) {
                Some(remaining) => Err(remaining),
                // Lockout elapsed: start over from zero.
                None => {
                    failures.remove(ip);
                    Ok(())
                }
            }
        }
        _ => Ok(()),
    }
}

/// Records a failed login.
pub fn record_failure(ip: &str) {
    let mut failures = lock();

    if failures.len() >= PRUNE_THRESHOLD {
        failures.retain(|_, attempts| attempts.last.elapsed() < LOCKOUT);
    }

    failures
        .entry(ip.to_owned())
        .and_modify(|attempts| {
            // A run of failures interrupted for long enough no longer counts.
            attempts.count = if attempts.last.elapsed() < LOCKOUT {
                attempts.count.saturating_add(1)
            } else {
                1
            };
            attempts.last = Instant::now();
        })
        .or_insert(Attempts {
            count: 1,
            last: Instant::now(),
        });
}

/// Clears the counter after a successful login.
pub fn record_success(ip: &str) {
    lock().remove(ip);
}

fn lock() -> std::sync::MutexGuard<'static, HashMap<String, Attempts>> {
    let failures = FAILURES.get_or_init(|| Mutex::new(HashMap::new()));
    // Poisoning can only come from a panic while holding the lock; the counters
    // are still usable, and losing them would be worse than picking them back up.
    failures
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_after_the_tolerated_number_of_failures() {
        let ip = "203.0.113.1";
        for _ in 0..MAX_FAILURES - 1 {
            record_failure(ip);
            assert!(check(ip).is_ok());
        }

        record_failure(ip);
        assert!(check(ip).is_err(), "the lockout should have kicked in");

        record_success(ip);
        assert!(check(ip).is_ok(), "a success should have lifted the lockout");
    }

    #[test]
    fn counts_addresses_separately() {
        let blocked = "203.0.113.2";
        for _ in 0..MAX_FAILURES {
            record_failure(blocked);
        }

        assert!(check(blocked).is_err());
        assert!(check("203.0.113.3").is_ok());
    }
}
