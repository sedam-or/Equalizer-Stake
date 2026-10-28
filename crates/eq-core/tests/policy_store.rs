use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use eq_core::actor::ActorCtx;
use eq_core::clock::Clock;
use eq_core::db::{self, Tx};
use eq_core::error::{AppError, FieldError};
use eq_core::errors;
use eq_core::ids::TenantId;
use eq_core::module::{CoreModule, Module, ModuleId, ModuleRegistry, SummaryLine};
use eq_core::policy::types::DayOfMonth;
use eq_core::policy::{PolicyDoc, PolicyStore, PresetPatchSource, PresetSourceError};
use eq_core::ports::sinks::{AuditSink, EventSink, PolicyChange, PolicyInit};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use tokio::task::JoinSet;
use uuid::Uuid;

#[path = "../../eq-testkit/src/db.rs"]
mod shared_test_db;
use shared_test_db::TestDb;

#[path = "../../eq-testkit/src/module_sinks.rs"]
mod shared_module_sinks;
use shared_module_sinks::ModuleSinkDoubles;

#[derive(Clone, Debug)]
struct TestClock(Arc<Mutex<DateTime<Utc>>>);

impl TestClock {
    fn new(now: DateTime<Utc>) -> Self {
        Self(Arc::new(Mutex::new(now)))
    }

    fn set(&self, now: DateTime<Utc>) {
        *self.0.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = now;
    }
}

impl Clock for TestClock {
    fn now(&self) -> DateTime<Utc> {
        *self.0.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

#[derive(Debug)]
struct TestPatchSource(Option<Value>);

impl PresetPatchSource for TestPatchSource {
    fn policy_patch(
        &self,
        preset_id: &str,
        preset_version: i32,
        module: ModuleId,
    ) -> Result<Option<Value>, PresetSourceError> {
        assert_eq!(preset_id, "test");
        assert_eq!(preset_version, 1);
        assert_eq!(module, ModuleId::Core);
        Ok(self.0.clone())
    }
}

#[derive(Debug)]
struct FailingAuditSink;

#[async_trait]
impl AuditSink for FailingAuditSink {
    async fn record_module_toggle(
        &self,
        _tx: &mut Tx,
        _tenant: TenantId,
        _actor: &ActorCtx,
        _module: ModuleId,
        _enabled: bool,
    ) -> std::result::Result<(), AppError> {
        Ok(())
    }

    async fn record_policy_change(
        &self,
        _tx: &mut Tx,
        _tenant: TenantId,
        _actor: &ActorCtx,
        _change: PolicyChange,
    ) -> std::result::Result<(), AppError> {
        Err(AppError::Internal(anyhow::anyhow!("fake audit sink failure")))
    }
}

#[derive(Debug)]
struct FailingEventSink;

#[async_trait]
impl EventSink for FailingEventSink {
    async fn emit_module_toggled(
        &self,
        _tx: &mut Tx,
        _tenant: TenantId,
        _actor: &ActorCtx,
        _module: ModuleId,
        _enabled: bool,
    ) -> std::result::Result<(), AppError> {
        Ok(())
    }

    async fn emit_policy_changed(
        &self,
        _tx: &mut Tx,
        _tenant: TenantId,
        _actor: &ActorCtx,
        _module: ModuleId,
        _version: i32,
        _effective_from: DateTime<Utc>,
    ) -> std::result::Result<(), AppError> {
        Err(AppError::Internal(anyhow::anyhow!("fake event sink failure")))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct TestCycle {
    issue_day: DayOfMonth,
    due_day: DayOfMonth,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct TestPolicy {
    cycle: TestCycle,
}

impl Default for TestPolicy {
    fn default() -> Self {
        Self {
            cycle: TestCycle {
                issue_day: DayOfMonth::from_const(1),
                due_day: DayOfMonth::from_const(2),
            },
        }
    }
}

impl PolicyDoc for TestPolicy {
    const MODULE: ModuleId = ModuleId::Core;
    const SCHEMA_VERSION: u32 = 1;

    fn validate(&self) -> Result<(), Vec<FieldError>> {
        if self.cycle.due_day.get() < self.cycle.issue_day.get() {
            return Err(vec![FieldError {
                pointer: "/cycle/due_day".to_owned(),
                code: errors::MUST_BE_GTE.to_owned(),
                params: Map::new(),
            }]);
        }
        Ok(())
    }

    fn summarize(&self) -> Vec<SummaryLine> {
        Vec::new()
    }
}

async fn fixture(
    patch: Option<Value>,
) -> Result<(TestDb, TenantId, PolicyStore, TestClock, ModuleSinkDoubles)> {
    let db = TestDb::new().await;
    let tenant = TenantId(Uuid::now_v7());
    insert_tenant(&db, tenant, "test", 1).await?;
    let clock = TestClock::new(at(2026, 1, 1));
    let sinks = ModuleSinkDoubles::default();
    let modules = Arc::new(ModuleRegistry::new(vec![Arc::new(CoreModule) as Arc<dyn Module>]));
    let store = PolicyStore::new(
        db.app_pool.clone(),
        modules,
        Arc::new(TestPatchSource(patch)),
        sinks.audit_sink(),
        sinks.event_sink(),
        Arc::new(clock.clone()),
    )
    .await?;
    Ok((db, tenant, store, clock, sinks))
}

async fn insert_tenant(db: &TestDb, tenant: TenantId, preset_id: &str, preset_version: i32) -> Result<()> {
    let slug = format!("school-{}", tenant.0.simple());
    sqlx::query!(
        "INSERT INTO public.tenants (id, slug, name, country, preset_id, preset_version) VALUES ($1, $2, 'Test School', 'ID', $3, $4)",
        tenant.0,
        slug,
        preset_id,
        preset_version
    )
    .execute(&db.pool)
    .await?;
    Ok(())
}

#[expect(
    clippy::expect_used,
    reason = "test timestamps use fixed valid UTC calendar dates"
)]
fn at(year: i32, month: u32, day: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(year, month, day, 0, 0, 0)
        .single()
        .expect("valid test timestamp")
}

fn doc(issue_day: u8, due_day: u8) -> TestPolicy {
    TestPolicy {
        cycle: TestCycle {
            issue_day: DayOfMonth::from_const(issue_day),
            due_day: DayOfMonth::from_const(due_day),
        },
    }
}

async fn put(
    db: &TestDb,
    store: &PolicyStore,
    tenant: TenantId,
    document: TestPolicy,
    effective_from: DateTime<Utc>,
) -> Result<i32> {
    let actor = ActorCtx::system("policy-test");
    let mut tx = db::begin_system(&db.app_pool, tenant).await?;
    let version = store
        .put::<TestPolicy>(&mut tx, tenant, document, effective_from, &actor, None)
        .await?
        .version;
    tx.commit().await?;
    Ok(version)
}

mod policy {
    mod tests {
        use super::super::*;

        #[tokio::test]
        async fn effective_dating() -> Result<()> {
            let (db, tenant, store, _, _) = fixture(None).await?;
            assert_eq!(put(&db, &store, tenant, doc(1, 5), at(2026, 1, 1)).await?, 1);
            assert_eq!(put(&db, &store, tenant, doc(1, 10), at(2026, 3, 1)).await?, 2);

            let mut tx = db::begin_system(&db.app_pool, tenant).await?;
            assert_eq!(
                store
                    .get::<TestPolicy>(&mut tx, tenant, at(2026, 2, 15))
                    .await?
                    .doc,
                doc(1, 5)
            );
            assert_eq!(
                store
                    .get::<TestPolicy>(&mut tx, tenant, at(2026, 3, 1))
                    .await?
                    .doc,
                doc(1, 10)
            );
            tx.commit().await?;
            Ok(())
        }
    }
}

#[tokio::test]
async fn future_version_applies_later() -> Result<()> {
    let (db, tenant, store, clock, _) = fixture(None).await?;
    put(&db, &store, tenant, doc(1, 5), at(2026, 1, 1)).await?;
    put(&db, &store, tenant, doc(1, 10), at(2026, 3, 1)).await?;

    let mut tx = db::begin_system(&db.app_pool, tenant).await?;
    assert_eq!(store.current::<TestPolicy>(&mut tx, tenant).await?.doc, doc(1, 5));
    tx.commit().await?;

    clock.set(at(2026, 3, 1));
    let mut tx = db::begin_system(&db.app_pool, tenant).await?;
    assert_eq!(
        store.current::<TestPolicy>(&mut tx, tenant).await?.doc,
        doc(1, 10)
    );
    tx.commit().await?;
    Ok(())
}

#[tokio::test]
async fn missing_policy_fallback_uses_requested_instant() -> Result<()> {
    let (db, tenant, store, _, _) = fixture(Some(json!({"cycle":{"due_day":6}}))).await?;
    let requested_at = at(2026, 2, 15);
    let mut tx = db::begin_system(&db.app_pool, tenant).await?;

    let fallback = store.get::<TestPolicy>(&mut tx, tenant, requested_at).await?;

    assert_eq!(fallback.version, 0);
    assert_eq!(fallback.effective_from, requested_at);
    assert_eq!(fallback.doc, doc(1, 6));
    tx.commit().await?;
    Ok(())
}

#[tokio::test]
async fn cross_field_error_pointer() -> Result<()> {
    let (db, tenant, store, _, _) = fixture(None).await?;
    let actor = ActorCtx::system("policy-test");
    let mut tx = db::begin_system(&db.app_pool, tenant).await?;
    let result = store
        .put::<TestPolicy>(&mut tx, tenant, doc(9, 8), at(2026, 1, 1), &actor, None)
        .await;
    let Err(AppError::Validation { errors }) = result else {
        panic!("the cross-field rule must reject an inverted cycle");
    };
    assert_eq!(errors[0].pointer, "/cycle/due_day");
    assert_eq!(errors[0].code, errors::MUST_BE_GTE);
    tx.commit().await?;
    Ok(())
}

#[tokio::test]
async fn concurrent_put_serialized() -> Result<()> {
    let (db, tenant, store, _, _) = fixture(None).await?;
    put(&db, &store, tenant, doc(1, 5), at(2026, 1, 1)).await?;
    let store = Arc::new(store);
    let mut writes = JoinSet::new();
    for due_day in [8, 12] {
        let pool = db.app_pool.clone();
        let store = Arc::clone(&store);
        writes.spawn(async move {
            let actor = ActorCtx::system("policy-test");
            let mut tx = db::begin_system(&pool, tenant).await?;
            let version = store
                .put::<TestPolicy>(&mut tx, tenant, doc(1, due_day), at(2026, 4, 1), &actor, None)
                .await?
                .version;
            tx.commit().await?;
            Ok::<_, AppError>(version)
        });
    }
    let mut versions = Vec::new();
    while let Some(result) = writes.join_next().await {
        versions.push(result??);
    }
    versions.sort_unstable();
    assert_eq!(versions, [2, 3]);
    Ok(())
}

#[tokio::test]
async fn append_only() -> Result<()> {
    let (db, tenant, store, _, _) = fixture(None).await?;
    put(&db, &store, tenant, doc(1, 5), at(2026, 1, 1)).await?;
    let mut tx = db::begin_system(&db.app_pool, tenant).await?;
    let result = sqlx::query!(
        "UPDATE public.policies SET document = $1 WHERE tenant_id = $2 AND module_id = 'core'",
        json!({"cycle":{"issue_day":1,"due_day":2}}),
        tenant.0
    )
    .execute(tx.conn())
    .await;
    assert!(result.is_err());
    drop(tx);
    Ok(())
}

#[tokio::test]
async fn put_records_policy_audit_and_event() -> Result<()> {
    let (db, tenant, store, _, sinks) = fixture(None).await?;
    let actor = ActorCtx::system("policy-test");
    let effective_from = at(2026, 1, 1);
    let mut tx = db::begin_system(&db.app_pool, tenant).await?;
    let version = store
        .put::<TestPolicy>(&mut tx, tenant, doc(1, 5), effective_from, &actor, None)
        .await?;
    let records = sinks.records()?;

    assert_eq!(records.policy_audits.len(), 1);
    assert_eq!(records.policy_audits[0].tenant, tenant);
    assert_eq!(records.policy_audits[0].version, version.version);
    assert_eq!(records.policy_audits[0].previous, None);
    assert_eq!(
        records.policy_audits[0].document,
        json!({"cycle":{"issue_day":1,"due_day":5}})
    );
    assert_eq!(records.policy_events.len(), 1);
    assert_eq!(records.policy_events[0].tenant, tenant);
    assert_eq!(records.policy_events[0].version, version.version);
    assert_eq!(records.policy_events[0].effective_from, effective_from);
    tx.commit().await?;
    Ok(())
}

#[tokio::test]
async fn audit_sink_failure_rolls_back_policy_write() -> Result<()> {
    let db = TestDb::new().await;
    let tenant = TenantId(Uuid::now_v7());
    insert_tenant(&db, tenant, "test", 1).await?;
    let clock = TestClock::new(at(2026, 1, 1));
    let modules = Arc::new(ModuleRegistry::new(vec![Arc::new(CoreModule) as Arc<dyn Module>]));
    let sinks = ModuleSinkDoubles::default();
    let store = PolicyStore::new(
        db.app_pool.clone(),
        modules,
        Arc::new(TestPatchSource(None)),
        Arc::new(FailingAuditSink),
        sinks.event_sink(),
        Arc::new(clock),
    )
    .await?;
    let actor = ActorCtx::system("policy-test");
    let mut tx = db::begin_system(&db.app_pool, tenant).await?;
    assert!(
        store
            .put::<TestPolicy>(&mut tx, tenant, doc(1, 5), at(2026, 1, 1), &actor, None)
            .await
            .is_err()
    );
    drop(tx);

    let mut check_tx = db::begin_system(&db.app_pool, tenant).await?;
    assert_eq!(
        store.current::<TestPolicy>(&mut check_tx, tenant).await?.version,
        0
    );
    check_tx.commit().await?;
    Ok(())
}

#[tokio::test]
async fn event_sink_failure_rolls_back_policy_write() -> Result<()> {
    let db = TestDb::new().await;
    let tenant = TenantId(Uuid::now_v7());
    insert_tenant(&db, tenant, "test", 1).await?;
    let clock = TestClock::new(at(2026, 1, 1));
    let modules = Arc::new(ModuleRegistry::new(vec![Arc::new(CoreModule) as Arc<dyn Module>]));
    let sinks = ModuleSinkDoubles::default();
    let store = PolicyStore::new(
        db.app_pool.clone(),
        modules,
        Arc::new(TestPatchSource(None)),
        sinks.audit_sink(),
        Arc::new(FailingEventSink),
        Arc::new(clock),
    )
    .await?;
    let actor = ActorCtx::system("policy-test");
    let mut tx = db::begin_system(&db.app_pool, tenant).await?;
    assert!(
        store
            .put::<TestPolicy>(&mut tx, tenant, doc(1, 5), at(2026, 1, 1), &actor, None)
            .await
            .is_err()
    );
    drop(tx);

    let mut check_tx = db::begin_system(&db.app_pool, tenant).await?;
    assert_eq!(
        store.current::<TestPolicy>(&mut check_tx, tenant).await?.version,
        0
    );
    check_tx.commit().await?;
    Ok(())
}

#[tokio::test]
async fn sink_double_records_policy_initialization() -> Result<()> {
    let (db, tenant, _, _, sinks) = fixture(None).await?;
    let mut tx = db::begin_system(&db.app_pool, tenant).await?;
    sinks
        .policy_init()
        .ensure_initial(&mut tx, tenant, ModuleId::Core)
        .await?;
    assert_eq!(sinks.records()?.policies_initialized, [(tenant, ModuleId::Core)]);
    tx.commit().await?;
    Ok(())
}

#[tokio::test]
async fn policy_init_merges_preset_patch_once() -> Result<()> {
    let (db, tenant, store, _, _) = fixture(Some(json!({"cycle":{"issue_day":1,"due_day":6}}))).await?;
    let mut tx = db::begin_system(&db.app_pool, tenant).await?;
    store.ensure_initial(&mut tx, tenant, ModuleId::Core).await?;
    store.ensure_initial(&mut tx, tenant, ModuleId::Core).await?;
    tx.commit().await?;

    let mut tx = db::begin_system(&db.app_pool, tenant).await?;
    let policy = store.current::<TestPolicy>(&mut tx, tenant).await?;
    assert_eq!(policy.version, 1);
    assert_eq!(policy.doc, doc(1, 6));
    tx.commit().await?;
    Ok(())
}

#[tokio::test]
async fn transaction_context_rejects_another_tenant() -> Result<()> {
    let (db, tenant_a, store, _, _) = fixture(None).await?;
    let tenant_b = TenantId(Uuid::now_v7());
    insert_tenant(&db, tenant_b, "test", 1).await?;
    let mut tx = db::begin_system(&db.app_pool, tenant_a).await?;

    assert!(matches!(
        store.get::<TestPolicy>(&mut tx, tenant_b, at(2026, 1, 1)).await,
        Err(AppError::Internal(_))
    ));
    let actor = ActorCtx::system("policy-test");
    assert!(matches!(
        store
            .put::<TestPolicy>(&mut tx, tenant_b, doc(1, 5), at(2026, 1, 1), &actor, None)
            .await,
        Err(AppError::Internal(_))
    ));
    drop(tx);
    Ok(())
}

#[tokio::test]
async fn cache_is_tenant_scoped() -> Result<()> {
    let (db, tenant_a, store, _, _) = fixture(None).await?;
    let tenant_b = TenantId(Uuid::now_v7());
    insert_tenant(&db, tenant_b, "test", 1).await?;
    put(&db, &store, tenant_a, doc(1, 5), at(2026, 1, 1)).await?;
    put(&db, &store, tenant_b, doc(1, 10), at(2026, 1, 1)).await?;

    let mut tx_a = db::begin_system(&db.app_pool, tenant_a).await?;
    assert_eq!(
        store.current::<TestPolicy>(&mut tx_a, tenant_a).await?.doc,
        doc(1, 5)
    );
    tx_a.commit().await?;
    let mut tx_b = db::begin_system(&db.app_pool, tenant_b).await?;
    assert_eq!(
        store.current::<TestPolicy>(&mut tx_b, tenant_b).await?.doc,
        doc(1, 10)
    );
    tx_b.commit().await?;
    Ok(())
}

#[tokio::test]
async fn test_database_debug_redacts_credentials() -> Result<()> {
    let (db, _, _, _, _) = fixture(None).await?;
    let owner_url_debug = format!("{:?}", db.url);
    let app_url_debug = format!("{:?}", db.app_url);
    assert!(!owner_url_debug.contains("test-secret-"));
    assert!(!app_url_debug.contains("test-secret-"));
    assert!(format!("{db:?}").contains("[REDACTED]"));
    Ok(())
}

#[tokio::test]
async fn cache_invalidated_by_notify() -> Result<()> {
    let db = TestDb::new().await;
    let tenant = TenantId(Uuid::now_v7());
    insert_tenant(&db, tenant, "test", 1).await?;
    let clock = TestClock::new(at(2026, 1, 1));
    let sinks = ModuleSinkDoubles::default();
    let modules = Arc::new(ModuleRegistry::new(vec![Arc::new(CoreModule) as Arc<dyn Module>]));
    let store_a = Arc::new(
        PolicyStore::new(
            db.app_pool.clone(),
            Arc::clone(&modules),
            Arc::new(TestPatchSource(None)),
            sinks.audit_sink(),
            sinks.event_sink(),
            Arc::new(clock.clone()),
        )
        .await?,
    );
    let store_b = PolicyStore::new(
        db.app_pool.clone(),
        modules,
        Arc::new(TestPatchSource(None)),
        sinks.audit_sink(),
        sinks.event_sink(),
        Arc::new(clock),
    )
    .await?;
    put(&db, &store_a, tenant, doc(1, 5), at(2026, 1, 1)).await?;
    let mut tx = db::begin_system(&db.app_pool, tenant).await?;
    assert_eq!(store_a.current::<TestPolicy>(&mut tx, tenant).await?.version, 1);
    tx.commit().await?;
    put(&db, &store_b, tenant, doc(1, 10), at(2026, 1, 1)).await?;

    let deadline = tokio::time::Instant::now() + Duration::from_secs(1);
    loop {
        let mut tx = db::begin_system(&db.app_pool, tenant).await?;
        let version = store_a.current::<TestPolicy>(&mut tx, tenant).await?.version;
        tx.commit().await?;
        if version == 2 {
            break;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "cache notification was not received within 1 second"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    Ok(())
}

#[test]
fn policy_change_debug_redacts_documents() {
    let change = PolicyChange {
        module: ModuleId::Core,
        version: 1,
        previous: Some(json!({"note":"test-secret-policy"})),
        document: json!({"note":"test-secret-policy"}),
    };
    let debug = format!("{change:?}");
    assert!(!debug.contains("test-secret-policy"));
}

#[tokio::test]
async fn resolver_returns_stored_preset_for_eq_app() -> Result<()> {
    let (db, tenant, _, _, _) = fixture(None).await?;
    let mut tx = db::begin_system(&db.app_pool, tenant).await?;
    let row = sqlx::query!(
        "SELECT preset_id, preset_version FROM public.resolve_tenant_preset($1)",
        tenant.0
    )
    .fetch_one(tx.conn())
    .await?;
    assert_eq!(row.preset_id.as_deref(), Some("test"));
    assert_eq!(row.preset_version, Some(1));
    tx.commit().await?;
    Ok(())
}

#[tokio::test]
async fn resolver_unknown_tenant_returns_no_rows() -> Result<()> {
    let (db, _, _, _, _) = fixture(None).await?;
    let unknown_tenant = Uuid::now_v7();
    let row = sqlx::query!(
        "SELECT preset_id, preset_version FROM public.resolve_tenant_preset($1)",
        unknown_tenant
    )
    .fetch_optional(&db.app_pool)
    .await?;
    assert!(row.is_none());
    Ok(())
}

#[tokio::test]
async fn app_cannot_select_preset_identity() -> Result<()> {
    let (db, tenant, _, _, _) = fixture(None).await?;
    let result = sqlx::query!("SELECT preset_id FROM public.tenants WHERE id = $1", tenant.0)
        .fetch_one(&db.app_pool)
        .await;
    let error = result.expect_err("eq_app must not have direct tenant-table access");
    assert!(
        error
            .as_database_error()
            .and_then(|error| error.code())
            .is_some_and(|code| code == "42501")
    );
    Ok(())
}

#[tokio::test]
async fn resolver_works_inside_tenant_transaction() -> Result<()> {
    let (db, tenant, store, _, _) = fixture(Some(json!({"cycle":{"due_day":6}}))).await?;
    let mut tx: Tx = db::begin_system(&db.app_pool, tenant).await?;
    let policy = store.get::<TestPolicy>(&mut tx, tenant, at(2026, 1, 1)).await?;
    assert_eq!(policy.doc, doc(1, 6));
    assert_eq!(policy.version, 0);
    tx.commit().await?;
    Ok(())
}
