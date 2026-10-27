//! Job handler and cron registration contracts consumed by modules (ARC-09).

use crate::clock::{Clock, Timestamp};
use crate::module::ModuleId;
use crate::tenant::TenantInfo;
use async_trait::async_trait;
use chrono::DateTime;
use chrono_tz::Tz;
use serde_json::Value;
use sqlx::PgPool;
use std::fmt;
use std::sync::Arc;

/// ARC-09: context supplied to one tenant or platform job handler.
#[derive(Clone)]
pub struct JobCtx {
    pub tenant: Option<Arc<TenantInfo>>,
    pub pool: PgPool,
    pub clock: Arc<dyn Clock>,
    pub now: Timestamp,
}

impl fmt::Debug for JobCtx {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("JobCtx")
            .field("tenant", &self.tenant)
            .field("pool", &"[configured]")
            .field("clock", &"[configured]")
            .field("now", &self.now)
            .finish()
    }
}

/// ARC-09: tenant-local schedule registered by a module.
#[derive(Clone, Copy, Debug)]
pub struct CronSpec {
    pub kind: &'static str,
    pub module: ModuleId,
    pub schedule: &'static str,
    pub dedupe: fn(crate::ids::TenantId, DateTime<Tz>) -> String,
}

/// ARC-09: one module-owned job handler.
#[async_trait]
pub trait JobHandler: Send + Sync {
    fn kind(&self) -> &'static str;
    fn module(&self) -> ModuleId;
    async fn run(&self, ctx: &JobCtx, payload: &Value) -> anyhow::Result<()>;
}
