//! Test fixtures and helpers shared by Ritma crates (spec/02 §2).

pub mod api;
pub mod clock;
#[expect(
    clippy::panic,
    reason = "TestDb setup must fail test startup with redacted database errors"
)]
pub mod db;
pub mod module_sinks;
pub mod rng;

pub use db::TestDb;
