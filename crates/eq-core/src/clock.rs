//! Clock seams used by services and tests (ARC-10).

use chrono::{DateTime, Utc};

/// ARC-10: UTC instant supplied to application services.
pub type Timestamp = DateTime<Utc>;

/// ARC-10: time source injected into application services.
pub trait Clock: Send + Sync + std::fmt::Debug + 'static {
    /// Returns the current UTC instant from this clock.
    fn now(&self) -> Timestamp;
}

/// ARC-10: production clock; the only wall-clock reader in the workspace.
#[derive(Debug, Default, Clone)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Timestamp {
        #[expect(
            clippy::disallowed_methods,
            reason = "ARC-10: SystemClock is the only wall-clock reader"
        )]
        Utc::now()
    }
}

/// Deterministic clock for tests and testkit consumers.
#[cfg(any(test, feature = "testkit"))]
pub mod test {
    use super::{Clock, Timestamp};
    use chrono::Duration;
    use std::sync::{Arc, Mutex, PoisonError};

    /// ARC-10: mutable clock shared by deterministic tests.
    #[derive(Debug, Clone)]
    pub struct ManualClock(Arc<Mutex<Timestamp>>);

    impl ManualClock {
        /// Creates a manual clock at `timestamp`.
        pub fn new(timestamp: Timestamp) -> Self {
            Self(Arc::new(Mutex::new(timestamp)))
        }

        /// Replaces the current instant.
        pub fn set(&self, timestamp: Timestamp) {
            *self.lock() = timestamp;
        }

        /// Advances the instant when the resulting timestamp is representable.
        pub fn advance(&self, duration: Duration) {
            let mut current = self.lock();
            if let Some(advanced) = current.checked_add_signed(duration) {
                *current = advanced;
            }
        }

        fn lock(&self) -> std::sync::MutexGuard<'_, Timestamp> {
            self.0.lock().unwrap_or_else(PoisonError::into_inner)
        }
    }

    impl Clock for ManualClock {
        fn now(&self) -> Timestamp {
            *self.lock()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Clock;
    use super::test::ManualClock;
    use chrono::{Duration, TimeZone, Utc};

    #[test]
    fn manual_clock_advances() {
        let initial = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        let clock = ManualClock::new(initial);

        clock.advance(Duration::minutes(15));

        assert_eq!(clock.now(), initial + Duration::minutes(15));
    }
}
