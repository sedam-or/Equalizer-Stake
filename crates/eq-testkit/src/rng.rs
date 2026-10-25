//! TST-20: deterministic randomness for tests.

use rand::{SeedableRng, rngs::StdRng};

/// TST-20: creates a repeatable standard RNG from the caller's fixed seed.
pub fn seeded(seed: u64) -> StdRng {
    StdRng::seed_from_u64(seed)
}
