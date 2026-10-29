use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use eq_core::actor::ActorCtx;
use eq_core::app_state::AppState;
use eq_core::clock::{Clock, Timestamp};
use eq_core::db;
use eq_core::error::AppError;
use eq_core::ids::TenantId;
use eq_core::jobs::{CronPlanner, CronSpec, JobCtx, JobError, JobHandler, JobRunner, enqueue};
use eq_core::model::TenantStatus;
use eq_core::module::gate::EnabledModuleSet;
use eq_core::module::{
    FieldError, Module, ModuleId, ModuleManifest, ModuleRegistry, NavItem, PermissionDef, SummaryLine,
};
use eq_core::money::Currency;
use eq_core::policy::types::Locale;
use eq_core::ports::PortRegistry;
use eq_core::tenant::TenantInfo;
use serde_json::{Value, json};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use utoipa_axum::router::OpenApiRouter;
use uuid::Uuid;

#[path = "../../eq-testkit/src/db.rs"]
#[expect(
    clippy::panic,
    reason = "the shared database fixture reports only redacted setup failures"
)]
#[expect(
    dead_code,
    reason = "the shared test fixture exposes credential-bearing URLs for suites that build extra pools"
)]
mod shared_test_db;

use shared_test_db::TestDb;

const JOB_KIND: &str = "billing.daily";

static BILLING_MANIFEST: ModuleManifest = ModuleManifest {
    id: ModuleId::Billing,
    name_key: "module.billing.name",
    description_key: "module.billing.description",
    icon: "receipt",
    always_on: false,
    requires_all: &[],
    requires_any: &[],
    policy_schema_version: 1,
};

#[derive(Clone)]
struct FixedClock(Timestamp);

impl std::fmt::Debug for FixedClock {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("FixedClock")
    }
}

impl Clock for FixedClock {
    fn now(&self) -> Timestamp {
        self.0
    }
}

struct FixedEnabled(bool);

#[async_trait]
impl EnabledModuleSet for FixedEnabled {
    async fn is_enabled(&self, _tenant: TenantId, _module: ModuleId) -> Result<bool, AppError> {
        Ok(self.0)
    }
}

#[derive(Clone)]
enum Behavior {
    Succeed,
    Retry,
    Permanent,
}

#[derive(Clone)]
struct TestHandler {
    behavior: Behavior,
    calls: Arc<AtomicUsize>,
}

#[async_trait]
impl JobHandler for TestHandler {
    fn kind(&self) -> &'static str {
        JOB_KIND
    }

    fn module(&self) -> ModuleId {
        ModuleId::Billing
    }

    async fn run(&self, _ctx: &JobCtx, _payload: &Value) -> Result<(), JobError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        match &self.behavior {
            Behavior::Succeed => Ok(()),
            Behavior::Retry => Err(JobError::Retryable(anyhow::anyhow!(
                "test-secret-handler-message"
            ))),
            Behavior::Permanent => Err(JobError::Permanent(anyhow::anyhow!(
                "test-secret-handler-message"
            ))),
        }
    }
}

struct TestModule {
    handler: Option<TestHandler>,
    schedules: Vec<CronSpec>,
}

#[async_trait]
impl Module for TestModule {
    fn manifest(&self) -> &'static ModuleManifest {
        &BILLING_MANIFEST
    }

    fn policy_schema(&self) -> Value {
        Value::Null
    }

    fn default_policy(&self) -> Value {
        Value::Null
    }

    fn validate_policy(&self, _document: &Value) -> Result<(), Vec<FieldError>> {
        Ok(())
    }

    fn summarize_policy(&self, _document: &Value) -> Vec<SummaryLine> {
        Vec::new()
    }

    fn migrate_policy(&self, _from_version: u32, document: Value) -> Result<Value> {
        Ok(document)
    }

    fn routes(&self) -> OpenApiRouter<AppState> {
        OpenApiRouter::new()
    }

    fn event_handlers(&self) -> Vec<Box<dyn eq_core::outbox::EventHandler>> {
        Vec::new()
    }

    fn jobs(&self) -> Vec<Box<dyn JobHandler>> {
        self.handler
            .as_ref()
            .map(|handler| Box::new(handler.clone()) as Box<dyn JobHandler>)
            .into_iter()
            .collect()
    }

    fn cron(&self) -> Vec<CronSpec> {
        self.schedules.clone()
    }

    fn register_ports(&self, _ports: &mut PortRegistry) {}

    fn permissions(&self) -> &'static [PermissionDef] {
        &[]
    }

    fn nav(&self) -> &'static [NavItem] {
        &[]
    }
}

fn registry(handler: Option<TestHandler>, schedules: Vec<CronSpec>) -> ModuleRegistry {
    ModuleRegistry::new(vec![Arc::new(TestModule { handler, schedules })])
}

fn tenant_info(id: TenantId, timezone: &str) -> TenantInfo {
    TenantInfo {
        id,
        slug: String::from("jobs-test"),
        status: TenantStatus::Active,
        timezone: timezone.to_owned(),
        locale: Locale::En,
        currency: Currency::IDR,
    }
}

#[expect(
    clippy::expect_used,
    reason = "fixed timestamp literals in the database tests are valid fixtures"
)]
fn instant(value: &str) -> Timestamp {
    DateTime::parse_from_rfc3339(value)
        .expect("fixed test timestamp is valid")
        .with_timezone(&Utc)
}

fn daily_dedupe(tenant: TenantId, fire_at: DateTime<chrono_tz::Tz>) -> String {
    format!("x:{}:{}", tenant.0, fire_at.format("%Y-%m-%d"))
}

fn daily_schedule() -> CronSpec {
    CronSpec {
        kind: JOB_KIND,
        module: ModuleId::Billing,
        schedule: "0 1 * * *",
        dedupe: daily_dedupe,
    }
}

#[expect(
    clippy::expect_used,
    reason = "a database fixture that cannot insert its tenant must abort the test"
)]
async fn test_db() -> (TestDb, TenantInfo) {
    let db = TestDb::new().await;
    let id = TenantId(Uuid::now_v7());
    let slug = format!("jobs-{}", id.0.simple());
    sqlx::query!(
        "INSERT INTO public.tenants (id, slug, name, country, preset_id, preset_version, status, timezone, locale, currency) \
         VALUES ($1, $2, 'Jobs Test', 'ID', 'test', 1, 'active', 'UTC', 'en', 'IDR')",
        id.0,
        slug
    )
    .execute(&db.pool)
    .await
    .expect("tenant fixture inserts");
    (db, tenant_info(id, "UTC"))
}

async fn enqueue_test_job(db: &TestDb, tenant: TenantId, run_at: Timestamp) -> Result<Uuid> {
    let mut tx = db::begin(&db.app_pool, tenant, &ActorCtx::system("jobs-test")).await?;
    let id = enqueue(
        &mut tx,
        Some(tenant),
        JOB_KIND,
        json!({ "v": 1, "person_id": Uuid::now_v7() }),
        run_at,
        Some(format!("test:{}", Uuid::now_v7())),
    )
    .await?;
    tx.commit().await?;
    Ok(id)
}

fn runner(db: &TestDb, behavior: Behavior, enabled: bool, calls: Arc<AtomicUsize>) -> JobRunner {
    let module = TestHandler { behavior, calls };
    JobRunner::new(
        db.app_pool.clone(),
        &registry(Some(module), Vec::new()),
        Arc::new(FixedEnabled(enabled)),
        Arc::new(FixedClock(DateTime::<Utc>::UNIX_EPOCH)),
    )
}

#[tokio::test]
async fn job_retry_and_dead() -> Result<()> {
    let (db, tenant) = test_db().await;
    let calls = Arc::new(AtomicUsize::new(0));
    let runner = runner(&db, Behavior::Retry, true, Arc::clone(&calls));
    let first = instant("2026-01-05T18:00:00Z");
    let id = enqueue_test_job(&db, tenant.id, first).await?;
    let mut now = first;

    for attempt in 1..=5 {
        assert_eq!(runner.run_due(std::slice::from_ref(&tenant), now).await?, 1);
        let row = sqlx::query!(
            "SELECT status, attempts, run_at, last_error FROM public.jobs WHERE id = $1",
            id
        )
        .fetch_one(&db.app_pool)
        .await?;
        assert_eq!(row.attempts, attempt);
        if attempt == 5 {
            assert_eq!(row.status, "dead");
            let last_error = row.last_error.unwrap_or_default();
            assert!(last_error.contains("retryable"));
            assert!(!last_error.contains("test-secret-handler-message"));
            let dedupe_key = format!("job_dead_letter:{id}");
            let mut attention_tx = db::begin(&db.app_pool, tenant.id, &ActorCtx::system("jobs-test")).await?;
            let attention = sqlx::query!(
                "SELECT kind, ref_type, ref_id, payload, severity, dedupe_key \
                 FROM public.attention_items WHERE tenant_id = $1 AND dedupe_key = $2",
                tenant.id.0,
                dedupe_key
            )
            .fetch_one(attention_tx.conn())
            .await?;
            attention_tx.commit().await?;
            assert_eq!(attention.kind, "job_dead_letter");
            assert_eq!(attention.ref_type.as_deref(), Some("job"));
            assert_eq!(attention.ref_id, Some(id));
            assert_eq!(attention.payload, json!({ "job_id": id }));
            assert_eq!(attention.severity, "high");
            assert_eq!(attention.dedupe_key.as_deref(), Some(dedupe_key.as_str()));
        } else {
            assert_eq!(row.status, "queued");
            let delay = 30_i64 * (1_i64 << (attempt - 1));
            assert_eq!(row.run_at, now + Duration::seconds(delay));
            now = row.run_at;
        }
    }

    assert_eq!(calls.load(Ordering::SeqCst), 5);
    let debug = format!(
        "{:?}",
        JobError::Retryable(anyhow::anyhow!("test-secret-handler-message"))
    );
    assert!(!debug.contains("test-secret-handler-message"));
    Ok(())
}

#[tokio::test]
async fn job_permanent_failure_is_dead_lettered() -> Result<()> {
    let (db, tenant) = test_db().await;
    let calls = Arc::new(AtomicUsize::new(0));
    let runner = runner(&db, Behavior::Permanent, true, Arc::clone(&calls));
    let now = instant("2026-01-05T18:00:00Z");
    let id = enqueue_test_job(&db, tenant.id, now).await?;

    assert_eq!(runner.run_due(std::slice::from_ref(&tenant), now).await?, 1);
    let row = sqlx::query!(
        "SELECT status, attempts, last_error FROM public.jobs WHERE id = $1",
        id
    )
    .fetch_one(&db.app_pool)
    .await?;
    assert_eq!(row.status, "dead");
    assert_eq!(row.attempts, 1);
    assert_eq!(row.last_error.as_deref(), Some("job failed (permanent)"));
    assert!(
        !row.last_error
            .unwrap_or_default()
            .contains("test-secret-handler-message")
    );
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    Ok(())
}

#[tokio::test]
async fn lease_reaped() -> Result<()> {
    let (db, tenant) = test_db().await;
    let calls = Arc::new(AtomicUsize::new(0));
    let runner = runner(&db, Behavior::Succeed, true, Arc::clone(&calls));
    let now = instant("2026-01-05T18:00:00Z");
    let id = Uuid::now_v7();
    sqlx::query!(
        "INSERT INTO public.jobs (id, tenant_id, kind, payload, run_at, status, attempts, locked_until) \
         VALUES ($1, $2, $3, '{}'::jsonb, $4::timestamptz, 'running', 1, $4::timestamptz - INTERVAL '1 second')",
        id,
        tenant.id.0,
        JOB_KIND,
        now
    )
    .execute(&db.pool)
    .await?;

    assert_eq!(runner.run_due(std::slice::from_ref(&tenant), now).await?, 1);
    let row = sqlx::query!(
        "SELECT status, attempts, locked_until FROM public.jobs WHERE id = $1",
        id
    )
    .fetch_one(&db.app_pool)
    .await?;
    assert_eq!(row.status, "done");
    assert_eq!(row.attempts, 2);
    assert!(row.locked_until.is_none());
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    Ok(())
}

#[tokio::test]
async fn disabled_module_job_is_skipped() -> Result<()> {
    let (db, tenant) = test_db().await;
    let calls = Arc::new(AtomicUsize::new(0));
    let runner = runner(&db, Behavior::Succeed, false, Arc::clone(&calls));
    let now = instant("2026-01-05T18:00:00Z");
    let id = enqueue_test_job(&db, tenant.id, now).await?;

    assert_eq!(runner.run_due(std::slice::from_ref(&tenant), now).await?, 1);
    let row = sqlx::query!("SELECT status, last_error FROM public.jobs WHERE id = $1", id)
        .fetch_one(&db.app_pool)
        .await?;
    assert_eq!(row.status, "done");
    assert_eq!(row.last_error.as_deref(), Some("skipped: module disabled"));
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    Ok(())
}

#[tokio::test]
async fn runner_processes_due_jobs_across_batches() -> Result<()> {
    let (db, tenant) = test_db().await;
    let calls = Arc::new(AtomicUsize::new(0));
    let runner = runner(&db, Behavior::Succeed, true, Arc::clone(&calls));
    let now = instant("2026-01-05T18:00:00Z");

    for _ in 0..101 {
        enqueue_test_job(&db, tenant.id, now).await?;
    }

    assert_eq!(runner.run_due(std::slice::from_ref(&tenant), now).await?, 101);
    assert_eq!(calls.load(Ordering::SeqCst), 101);
    let count = sqlx::query_scalar!(
        "SELECT count(*) FROM public.jobs WHERE tenant_id = $1 AND kind = $2",
        tenant.id.0,
        JOB_KIND
    )
    .fetch_one(&db.app_pool)
    .await?;
    assert_eq!(count, Some(101));
    Ok(())
}

#[tokio::test]
async fn planner_dedupe_across_instances() -> Result<()> {
    let (db, tenant) = test_db().await;
    let mut tenant = tenant;
    tenant.timezone = String::from("Asia/Jakarta");
    let registry = registry(None, vec![daily_schedule()]);
    let enabled: Arc<dyn EnabledModuleSet> = Arc::new(FixedEnabled(true));
    let first = CronPlanner::new(db.app_pool.clone(), &registry, Arc::clone(&enabled));
    let second = CronPlanner::new(db.app_pool.clone(), &registry, enabled);
    let now = instant("2026-01-05T18:00:30Z");

    let (first_count, second_count) = tokio::join!(
        first.plan_due(std::slice::from_ref(&tenant), now),
        second.plan_due(std::slice::from_ref(&tenant), now)
    );
    assert_eq!(first_count? + second_count?, 1);
    let count = sqlx::query_scalar!(
        "SELECT count(*) FROM public.jobs WHERE tenant_id = $1 AND kind = $2",
        tenant.id.0,
        JOB_KIND
    )
    .fetch_one(&db.app_pool)
    .await?;
    assert_eq!(count, Some(1));
    Ok(())
}

#[tokio::test]
async fn catch_up_single_occurrence() -> Result<()> {
    let (db, mut tenant) = test_db().await;
    tenant.timezone = String::from("Asia/Jakarta");
    let previous_tick = instant("2026-01-01T00:00:00Z");
    sqlx::query!(
        "INSERT INTO public.cron_state (tenant_id, kind, last_fire_at) VALUES ($1, $2, $3)",
        tenant.id.0,
        JOB_KIND,
        previous_tick
    )
    .execute(&db.pool)
    .await?;
    let planner = CronPlanner::new(
        db.app_pool.clone(),
        &registry(None, vec![daily_schedule()]),
        Arc::new(FixedEnabled(true)),
    );

    let now = instant("2026-01-04T18:01:00Z");
    assert_eq!(planner.plan_due(std::slice::from_ref(&tenant), now).await?, 1);
    let rows = sqlx::query!(
        "SELECT dedupe_key FROM public.jobs WHERE tenant_id = $1 AND kind = $2",
        tenant.id.0,
        JOB_KIND
    )
    .fetch_all(&db.app_pool)
    .await?;
    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0].dedupe_key.as_deref(),
        Some(format!("x:{}:2026-01-05", tenant.id.0).as_str())
    );
    Ok(())
}

mod cron {
    use super::*;

    mod tests {
        use super::*;

        #[tokio::test]
        async fn fires_in_tenant_tz() -> Result<()> {
            let (db, mut tenant) = test_db().await;
            tenant.timezone = String::from("Asia/Jakarta");
            let planner = CronPlanner::new(
                db.app_pool.clone(),
                &registry(None, vec![daily_schedule()]),
                Arc::new(FixedEnabled(true)),
            );

            let before = instant("2026-01-05T17:59:00Z");
            let after = instant("2026-01-05T18:01:00Z");
            assert_eq!(planner.plan_due(std::slice::from_ref(&tenant), before).await?, 0);
            assert_eq!(planner.plan_due(std::slice::from_ref(&tenant), after).await?, 1);
            let row = sqlx::query!(
                "SELECT run_at, dedupe_key FROM public.jobs WHERE tenant_id = $1 AND kind = $2",
                tenant.id.0,
                JOB_KIND
            )
            .fetch_one(&db.app_pool)
            .await?;
            assert_eq!(row.run_at, instant("2026-01-05T18:00:00Z"));
            assert_eq!(
                row.dedupe_key.as_deref(),
                Some(format!("x:{}:2026-01-06", tenant.id.0).as_str())
            );
            Ok(())
        }
    }
}

#[tokio::test]
async fn dst_spring_forward() -> Result<()> {
    let (db, mut tenant) = test_db().await;
    tenant.timezone = String::from("America/New_York");
    let dst_schedule = CronSpec {
        kind: JOB_KIND,
        module: ModuleId::Billing,
        schedule: "30 2 * * *",
        dedupe: daily_dedupe,
    };
    let planner = CronPlanner::new(
        db.app_pool.clone(),
        &registry(None, vec![dst_schedule]),
        Arc::new(FixedEnabled(true)),
    );
    let now = instant("2026-03-08T07:00:00Z");

    assert_eq!(planner.plan_due(std::slice::from_ref(&tenant), now).await?, 1);
    let row = sqlx::query!(
        "SELECT run_at FROM public.jobs WHERE tenant_id = $1 AND kind = $2",
        tenant.id.0,
        JOB_KIND
    )
    .fetch_one(&db.app_pool)
    .await?;
    assert_eq!(row.run_at, now);
    let timezone: chrono_tz::Tz = "America/New_York".parse()?;
    assert_eq!(
        row.run_at.with_timezone(&timezone).format("%H:%M").to_string(),
        "03:00"
    );
    Ok(())
}

#[tokio::test]
async fn dst_fall_back_first_occurrence() -> Result<()> {
    let (db, mut tenant) = test_db().await;
    tenant.timezone = String::from("America/New_York");
    let previous_tick = instant("2026-11-01T05:14:00Z");
    sqlx::query!(
        "INSERT INTO public.cron_state (tenant_id, kind, last_fire_at) VALUES ($1, $2, $3)",
        tenant.id.0,
        JOB_KIND,
        previous_tick
    )
    .execute(&db.pool)
    .await?;
    let fold_schedule = CronSpec {
        kind: JOB_KIND,
        module: ModuleId::Billing,
        schedule: "30 1 * * *",
        dedupe: daily_dedupe,
    };
    let planner = CronPlanner::new(
        db.app_pool.clone(),
        &registry(None, vec![fold_schedule]),
        Arc::new(FixedEnabled(true)),
    );
    let now = instant("2026-11-01T06:45:00Z");

    assert_eq!(planner.plan_due(std::slice::from_ref(&tenant), now).await?, 1);
    assert_eq!(
        planner
            .plan_due(std::slice::from_ref(&tenant), now + Duration::minutes(1))
            .await?,
        0
    );
    let row = sqlx::query!(
        "SELECT run_at FROM public.jobs WHERE tenant_id = $1 AND kind = $2",
        tenant.id.0,
        JOB_KIND
    )
    .fetch_one(&db.app_pool)
    .await?;
    assert_eq!(row.run_at, instant("2026-11-01T05:30:00Z"));
    Ok(())
}
