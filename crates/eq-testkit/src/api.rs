//! TST-04/TST-20: API tests use the least-privileged database pool.

use axum_test::TestServer;
use eq_api::{AppState, router};
use eq_core::config::Config;
use std::{fmt, sync::Arc};

use crate::{TestDb, clock::test_clock};

/// TST-04: router test harness backed by the RLS-enforcing application pool.
pub struct TestApp {
    pub server: TestServer,
    pub db: TestDb,
    pub clock: eq_core::clock::test::ManualClock,
}

impl fmt::Debug for TestApp {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TestApp")
            .field("server", &"[TEST SERVER]")
            .field("db", &self.db)
            .field("clock", &self.clock)
            .finish()
    }
}

impl TestApp {
    /// TST-04: creates a router test server using `TestDb::app_pool`.
    #[expect(
        clippy::panic,
        reason = "TestApp has a fixed task signature; setup failures stop the test without formatting database credentials"
    )]
    pub async fn new() -> Self {
        let db = TestDb::new().await;
        let clock = test_clock();
        let state = AppState {
            pool: db.app_pool.clone(),
            config: Arc::new(Config::for_tests()),
            clock: Arc::new(clock.clone()),
        };
        let Ok(server) = TestServer::new(router(state)) else {
            panic!("test API server setup failed; details redacted");
        };

        Self { server, db, clock }
    }
}
