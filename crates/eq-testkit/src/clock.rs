//! TST-20: deterministic test time fixtures.

use chrono::{DateTime, Utc};
pub use eq_core::clock::test::ManualClock;

/// TST-20: creates a controllable clock at the canonical Monday start instant.
#[expect(
    clippy::expect_used,
    reason = "the task's fixed RFC3339 fixture is known-valid"
)]
pub fn test_clock() -> ManualClock {
    let start = DateTime::parse_from_rfc3339("2026-01-05T00:00:00+07:00")
        .expect("the TST-20 fixed test timestamp is valid")
        .with_timezone(&Utc);
    ManualClock::new(start)
}
