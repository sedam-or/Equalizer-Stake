//! Transactional jobs and tenant-local cron planning (ARC-09, ARC-10, ERR-51).

mod repo;

use crate::clock::{Clock, Timestamp};
use crate::db::{self, Tx};
use crate::error::AppError;
use crate::ids::TenantId;
use crate::module::ModuleRegistry;
use crate::module::gate::EnabledModuleSet;
use crate::policy::PolicyStore;
use crate::tenant::TenantInfo;
use anyhow::anyhow;
use async_trait::async_trait;
use chrono::{DateTime, Duration as ChronoDuration, NaiveDateTime, TimeZone, Utc};
use chrono_tz::Tz;
use cron::Schedule;
use serde_json::Value;
use sqlx::PgPool;
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

const JOB_CLAIM_BATCH_SIZE: i64 = 100; // technical: bound one worker claim, matching the outbox worker.
const JOB_LEASE_SECONDS: i64 = 5 * 60; // technical: ARC-09 lease duration.
const JOB_RETRY_BASE_SECONDS: i64 = 30; // technical: ARC-09 exponential retry base.
const CRON_INITIAL_LOOKBACK_SECONDS: i64 = 60; // technical: T-108 first-tick lookback.
const RETRYABLE_ERROR_TEXT: &str = "job failed (retryable)";
const PERMANENT_ERROR_TEXT: &str = "job failed (permanent)";
const MISSING_HANDLER_ERROR_TEXT: &str = "job handler is not registered";
const JOB_DEAD_LETTER_KIND: &str = "job_dead_letter";
const JOB_DEAD_LETTER_DEDUPE_PREFIX: &str = "job_dead_letter:";

/// ARC-09: context supplied to one tenant or platform job handler.
#[derive(Clone)]
pub struct JobCtx {
    /// Resolved tenant metadata for tenant jobs; `None` identifies a platform job.
    pub tenant: Option<Arc<TenantInfo>>,
    /// Application pool used by the handler's own repositories.
    pub pool: PgPool,
    /// ARC-10 clock injected by the worker.
    pub clock: Arc<dyn Clock>,
    /// ARC-10 instant at which this worker claimed the job.
    pub now: Timestamp,
    /// T-109/POL-04: shared policy service for handlers that evaluate tenant policy.
    pub policy_store: Option<Arc<PolicyStore>>,
}

impl fmt::Debug for JobCtx {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("JobCtx")
            .field("tenant", &self.tenant.as_ref().map(|_| "[tenant]"))
            .field("pool", &"[configured]")
            .field("clock", &"[configured]")
            .field("now", &self.now)
            .field(
                "policy_store",
                &self.policy_store.as_ref().map(|_| "[configured]"),
            )
            .finish()
    }
}

/// ARC-09: tenant-local schedule registered by a module.
#[derive(Clone, Copy, Debug)]
pub struct CronSpec {
    /// Globally stable queue kind used for the schedule and handler lookup.
    pub kind: &'static str,
    /// Module that owns and gates this schedule.
    pub module: crate::module::ModuleId,
    /// Five-field cron expression evaluated against tenant-local wall time.
    pub schedule: &'static str,
    /// CAN-62 dedupe key composed from the tenant id and scheduled instant.
    pub dedupe: fn(TenantId, DateTime<Tz>) -> String,
}

/// ERR-51: classifies a handler failure without exposing its message in logs or persisted state.
pub enum JobError {
    /// Transient error eligible for the configured retry schedule.
    Retryable(anyhow::Error),
    /// Invalid data or a business failure that must not be retried.
    Permanent(anyhow::Error),
}

impl fmt::Debug for JobError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let class = match self {
            Self::Retryable(_) => "Retryable",
            Self::Permanent(_) => "Permanent",
        };
        formatter.debug_tuple("JobError").field(&class).finish()
    }
}

impl fmt::Display for JobError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Retryable(_) => formatter.write_str("retryable job failure"),
            Self::Permanent(_) => formatter.write_str("permanent job failure"),
        }
    }
}

impl std::error::Error for JobError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Retryable(error) | Self::Permanent(error) => Some(error.as_ref()),
        }
    }
}

impl From<anyhow::Error> for JobError {
    fn from(error: anyhow::Error) -> Self {
        Self::Retryable(error)
    }
}

/// ARC-09: one module-owned job handler.
#[async_trait]
pub trait JobHandler: Send + Sync {
    /// Returns the stable kind consumed by this handler.
    fn kind(&self) -> &'static str;
    /// Returns the module whose enabled state gates this handler.
    fn module(&self) -> crate::module::ModuleId;
    /// Runs one payload using the injected tenant, pool and clock context.
    async fn run(&self, ctx: &JobCtx, payload: &Value) -> Result<(), JobError>;
}

/// ARC-09: enqueues work inside the caller's existing database transaction.
pub async fn enqueue(
    tx: &mut Tx,
    tenant: Option<TenantId>,
    kind: &str,
    payload: Value,
    run_at: Timestamp,
    dedupe_key: Option<String>,
) -> Result<Uuid, AppError> {
    if tenant.is_some_and(|tenant| tenant != tx.tenant())
        || (tenant.is_none() && tx.actor().role != crate::actor::ActorRole::Platform)
    {
        return Err(AppError::Internal(anyhow!(
            "job tenant does not match transaction context"
        )));
    }
    repo::enqueue(tx, tenant, kind, payload, run_at, dedupe_key).await
}

/// ARC-09: claims, gates and runs due jobs using the supplied trusted tenant set.
pub struct JobRunner {
    pool: PgPool,
    handlers: HashMap<String, Arc<dyn JobHandler>>,
    enabled: Arc<dyn EnabledModuleSet>,
    clock: Arc<dyn Clock>,
    policy_store: Option<Arc<PolicyStore>>,
}

impl fmt::Debug for JobRunner {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("JobRunner")
            .field("pool", &"[configured]")
            .field("handler_count", &self.handlers.len())
            .field(
                "policy_store",
                &self.policy_store.as_ref().map(|_| "[configured]"),
            )
            .finish_non_exhaustive()
    }
}

impl JobRunner {
    /// MOD-04/ARC-09: creates a runner from the compiled registry's job handlers.
    pub fn new(
        pool: PgPool,
        modules: &ModuleRegistry,
        enabled: Arc<dyn EnabledModuleSet>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        let handlers = modules
            .modules()
            .iter()
            .flat_map(|module| module.jobs())
            .map(|handler| (handler.kind().to_owned(), Arc::from(handler)))
            .collect();
        Self {
            pool,
            handlers,
            enabled,
            clock,
            policy_store: None,
        }
    }

    /// T-109/POL-04: supplies the shared policy service to handlers that need tenant policy.
    pub fn with_policy_store(mut self, policy_store: Arc<PolicyStore>) -> Self {
        self.policy_store = Some(policy_store);
        self
    }

    /// ARC-09: reaps expired leases and processes one bounded batch due at `now`.
    pub async fn run_due(&self, tenants: &[TenantInfo], now: Timestamp) -> Result<usize, AppError> {
        let tenant_ids = tenants.iter().map(|tenant| tenant.id).collect::<Vec<_>>();
        repo::reap_expired(&self.pool, now).await?;
        let mut claimed = 0;
        loop {
            let jobs = repo::claim_due(
                &self.pool,
                &tenant_ids,
                now,
                JOB_CLAIM_BATCH_SIZE,
                JOB_LEASE_SECONDS,
            )
            .await?;
            if jobs.is_empty() {
                break;
            }
            claimed += jobs.len();

            for job in jobs {
                let Some(handler) = self.handlers.get(&job.kind) else {
                    repo::mark_dead(&self.pool, &job, now, MISSING_HANDLER_ERROR_TEXT).await?;
                    log_dead_letter(&job, "permanent");
                    continue;
                };
                let tenant = job
                    .tenant_id
                    .and_then(|id| tenants.iter().find(|tenant| tenant.id == id));
                if job.tenant_id.is_some() && tenant.is_none() {
                    // The claim query admits only caller-supplied tenant ids; keep this defensive path fail-closed.
                    repo::mark_retry(&self.pool, &job, now, "tenant metadata is unavailable").await?;
                    continue;
                }
                if let Some(tenant) = tenant
                    && !self.enabled.is_enabled(tenant.id, handler.module()).await?
                {
                    repo::mark_done(&self.pool, job.id, now, Some("skipped: module disabled")).await?;
                    continue;
                }

                let context = JobCtx {
                    tenant: tenant.cloned().map(Arc::new),
                    pool: self.pool.clone(),
                    clock: Arc::clone(&self.clock),
                    now,
                    policy_store: self.policy_store.clone(),
                };
                match handler.run(&context, &job.payload).await {
                    Ok(()) => repo::mark_done(&self.pool, job.id, now, None).await?,
                    Err(JobError::Permanent(_)) => {
                        repo::mark_dead(&self.pool, &job, now, PERMANENT_ERROR_TEXT).await?;
                        log_dead_letter(&job, "permanent");
                    }
                    Err(JobError::Retryable(_)) if job.attempts >= job.max_attempts => {
                        repo::mark_dead(&self.pool, &job, now, RETRYABLE_ERROR_TEXT).await?;
                        log_dead_letter(&job, "retryable");
                    }
                    Err(JobError::Retryable(_)) => {
                        repo::mark_retry(&self.pool, &job, now, RETRYABLE_ERROR_TEXT).await?;
                    }
                }
            }
        }

        Ok(claimed)
    }
}

fn log_dead_letter(job: &repo::ClaimedJob, error_class: &'static str) {
    tracing::error!(
        job_id = %job.id,
        tenant_id = ?job.tenant_id.map(|tenant| tenant.0),
        actor_id = "system",
        failure_code = "handler_failed",
        error_class,
        outcome = "dead_lettered",
        jobs_dead_lettered = 1_u64,
        "job dead-lettered"
    );
}

/// ARC-09: plans one latest due occurrence for each active tenant and enabled module schedule.
pub struct CronPlanner {
    pool: PgPool,
    schedules: Vec<CronSpec>,
    enabled: Arc<dyn EnabledModuleSet>,
}

impl fmt::Debug for CronPlanner {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CronPlanner")
            .field("pool", &"[configured]")
            .field("schedule_count", &self.schedules.len())
            .finish_non_exhaustive()
    }
}

impl CronPlanner {
    /// MOD-04/ARC-09: gathers the schedule declarations from compiled modules.
    pub fn new(pool: PgPool, modules: &ModuleRegistry, enabled: Arc<dyn EnabledModuleSet>) -> Self {
        let schedules = modules
            .modules()
            .iter()
            .flat_map(|module| module.cron())
            .collect();
        Self {
            pool,
            schedules,
            enabled,
        }
    }

    /// ARC-09: plans schedules only for active tenants supplied by a trusted caller.
    pub async fn plan_due(&self, tenants: &[TenantInfo], now: Timestamp) -> Result<usize, AppError> {
        let mut inserted = 0;
        for tenant in tenants
            .iter()
            .filter(|tenant| tenant.status == crate::model::TenantStatus::Active)
        {
            let timezone = Tz::from_str(&tenant.timezone)
                .map_err(|_| AppError::Internal(anyhow!("invalid timezone in trusted tenant metadata")))?;
            for spec in &self.schedules {
                if !self.enabled.is_enabled(tenant.id, spec.module).await? {
                    continue;
                }
                let schedule = parse_schedule(spec.schedule)?;
                let mut tx = db::begin_system(&self.pool, tenant.id).await?;
                let lookback = now
                    .checked_sub_signed(ChronoDuration::seconds(CRON_INITIAL_LOOKBACK_SECONDS))
                    .unwrap_or(now);
                let last_fire = repo::cron_last_fire(&mut tx, tenant.id, spec.kind)
                    .await?
                    .unwrap_or(lookback);
                if let Some(fire_at) = latest_local_fire(&schedule, timezone, last_fire, now) {
                    let dedupe_key = (spec.dedupe)(tenant.id, fire_at);
                    let fire_at_utc = fire_at.with_timezone(&Utc);
                    if repo::enqueue_cron(&mut tx, tenant.id, spec.kind, fire_at_utc, &dedupe_key).await? {
                        inserted += 1;
                    }
                }
                repo::upsert_cron_state(&mut tx, tenant.id, spec.kind, now).await?;
                tx.commit().await?;
            }
        }
        Ok(inserted)
    }
}

fn parse_schedule(expression: &str) -> Result<Schedule, AppError> {
    let fields = expression.split_whitespace().collect::<Vec<_>>();
    if fields.len() != 5 {
        return Err(AppError::Internal(anyhow!("cron schedule must have five fields")));
    }
    let expanded = format!("0 {} *", fields.join(" "));
    Schedule::from_str(&expanded).map_err(|_| AppError::Internal(anyhow!("invalid module cron schedule")))
}

fn latest_local_fire(
    schedule: &Schedule,
    timezone: Tz,
    last_fire: Timestamp,
    now: Timestamp,
) -> Option<DateTime<Tz>> {
    let now_wall = latest_planning_wall_time(now, timezone)?;
    let wall_now = DateTime::<Utc>::from_naive_utc_and_offset(now_wall, Utc);
    let scheduled_wall = schedule.after(&wall_now).next_back()?.naive_utc();
    if scheduled_wall > now_wall {
        return None;
    }
    let fire_at = localize_wall_time(scheduled_wall, timezone)?;
    let fire_at_utc = fire_at.with_timezone(&Utc);
    (fire_at_utc > last_fire && fire_at_utc <= now).then_some(fire_at)
}

fn latest_planning_wall_time(now: Timestamp, timezone: Tz) -> Option<NaiveDateTime> {
    let local_now = now.with_timezone(&timezone);
    let wall_now = local_now.naive_local();
    if let chrono::LocalResult::Ambiguous(_, later) = timezone.from_local_datetime(&wall_now)
        && local_now == later
    {
        let mut candidate = wall_now;
        loop {
            candidate = candidate.checked_add_signed(ChronoDuration::minutes(1))?;
            match timezone.from_local_datetime(&candidate) {
                chrono::LocalResult::Ambiguous(_, _) => continue,
                chrono::LocalResult::Single(_) => {
                    return candidate.checked_sub_signed(ChronoDuration::seconds(1));
                }
                chrono::LocalResult::None => continue,
            }
        }
    }
    Some(wall_now)
}

fn localize_wall_time(wall_time: NaiveDateTime, timezone: Tz) -> Option<DateTime<Tz>> {
    let mut candidate = wall_time;
    loop {
        match timezone.from_local_datetime(&candidate) {
            chrono::LocalResult::Single(instant) => return Some(instant),
            chrono::LocalResult::Ambiguous(earlier, _) => return Some(earlier),
            chrono::LocalResult::None => {
                candidate = candidate.checked_add_signed(ChronoDuration::minutes(1))?;
            }
        }
    }
}
