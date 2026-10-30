use anyhow::{Result, anyhow};
use chrono::{DateTime, Months, Utc};
use eq_audit::policy::AuditPolicy;
use eq_audit::{AuditEvent, AuditModule, AuditRetentionJob, AuditVerifyRecentJob, Category, Sealer, record};
use eq_core::actor::{ActorCtx, ActorRole};
use eq_core::clock::SystemClock;
use eq_core::db;
use eq_core::ids::TenantId;
use eq_core::jobs::{JobCtx, JobHandler};
use eq_core::model::TenantStatus;
use eq_core::module::{ModuleId, ModuleRegistry};
use eq_core::money::Currency;
use eq_core::policy::types::Locale;
use eq_core::policy::{PolicyStore, PresetPatchSource, PresetSourceError};
use eq_core::ports::sinks::{AuditSink, PolicyChange};
use eq_core::tenant::TenantInfo;
use eq_testkit::module_sinks::ModuleSinkDoubles;
use serde_json::{Value, json};
use sqlx::{Error, PgPool, Row, postgres::PgPoolOptions};
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

const SEALER_LOCK_WAIT_TIMEOUT: Duration = Duration::from_secs(10); // technical: allow PostgreSQL activity state to expose the test lock wait.
const SEALER_ACTIVITY_POLL_INTERVAL: Duration = Duration::from_millis(10); // technical: observe lock state without busy polling.

struct TestDb {
    pool: PgPool,
    app_pool: PgPool,
}

impl TestDb {
    async fn new() -> Result<Self> {
        let owner_url = test_database_url("RITMA_TEST_DATABASE_URL")?;
        let app_url = test_database_url("RITMA_TEST_DATABASE_APP_URL")?;
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&owner_url)
            .await?;
        let app_pool = PgPoolOptions::new().max_connections(5).connect(&app_url).await?;
        Ok(Self { pool, app_pool })
    }
}

fn test_database_url(name: &str) -> Result<String> {
    std::env::var_os(name)
        .ok_or_else(|| anyhow!("{name} is required for the fixed PostgreSQL test database"))?
        .into_string()
        .map_err(|_| anyhow!("{name} must be UTF-8"))
}

async fn tenant(db: &TestDb) -> Result<TenantId> {
    let id = TenantId(Uuid::now_v7());
    let slug = format!("audit-{}", id.0.simple());
    sqlx::query(
        "INSERT INTO public.tenants \
             (id, slug, name, country, preset_id, preset_version, status, timezone, locale, currency) \
         VALUES ($1, $2, 'Audit Test', 'ID', 'test', 1, 'active', 'UTC', 'en', 'IDR')",
    )
    .bind(id.0)
    .bind(slug)
    .execute(&db.pool)
    .await?;
    Ok(id)
}

#[tokio::test]
async fn audit_failure_rolls_back() -> Result<()> {
    let db = TestDb::new().await?;
    let tenant = tenant(&db).await?;
    let row_id = Uuid::now_v7();

    sqlx::raw_sql(
        "CREATE TABLE public.audit_business_rows (id uuid PRIMARY KEY); \
         GRANT INSERT, SELECT ON public.audit_business_rows TO eq_app; \
         CREATE FUNCTION public.audit_reject_marker() RETURNS trigger \
         LANGUAGE plpgsql SET search_path = pg_catalog AS $$ \
         BEGIN \
           IF NEW.action = 'test.audit.fail' THEN \
             RAISE EXCEPTION 'audit fixture rejected marker'; \
           END IF; \
           RETURN NEW; \
         END $$; \
         CREATE TRIGGER audit_reject_marker BEFORE INSERT ON audit.entries \
         FOR EACH ROW EXECUTE FUNCTION public.audit_reject_marker();",
    )
    .execute(&db.pool)
    .await?;

    let actor = ActorCtx::system("audit-test");
    let mut tx = db::begin(&db.app_pool, tenant, &actor).await?;
    sqlx::query("INSERT INTO public.audit_business_rows (id) VALUES ($1)")
        .bind(row_id)
        .execute(tx.conn())
        .await?;

    let result = record(
        &mut tx,
        AuditEvent {
            action: "test.audit.fail",
            category: Category::Operational,
            entity_type: "business_row",
            entity_id: Some(row_id),
            before: None,
            after: Some(json!({ "id": row_id })),
            actor: &actor,
        },
    )
    .await;
    assert!(result.is_err());
    tx.rollback().await?;

    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM public.audit_business_rows WHERE id = $1)",
    )
    .bind(row_id)
    .fetch_one(&db.pool)
    .await?;
    assert!(!exists);
    Ok(())
}

#[tokio::test]
async fn audit_sink_records_changes_in_the_caller_transaction() -> Result<()> {
    let db = TestDb::new().await?;
    let tenant = tenant(&db).await?;
    let actor = ActorCtx::system("audit-sink-test");
    let sink = AuditModule;
    let mut tx = db::begin(&db.app_pool, tenant, &actor).await?;

    sink.record_module_toggle(&mut tx, tenant, &actor, ModuleId::Scheduling, true)
        .await?;
    sink.record_policy_change(
        &mut tx,
        tenant,
        &actor,
        PolicyChange {
            module: ModuleId::Audit,
            version: 1,
            previous: None,
            document: json!({
                "retention_years": 7,
                "anonymize_pii_after_years": 2,
                "admin_categories": ["operational"]
            }),
        },
    )
    .await?;

    let actions =
        sqlx::query_scalar::<_, String>("SELECT action FROM audit.entries WHERE tenant_id = $1 ORDER BY seq")
            .bind(tenant.0)
            .fetch_all(tx.conn())
            .await?;
    assert_eq!(
        actions,
        vec!["core.module.enabled".to_owned(), "core.policy.changed".to_owned()]
    );
    tx.rollback().await?;

    let persisted = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM audit.entries WHERE tenant_id = $1 AND action IN ('core.module.enabled', 'core.policy.changed')",
    )
    .bind(tenant.0)
    .fetch_one(&db.pool)
    .await?;
    assert_eq!(persisted, 0);
    Ok(())
}

#[tokio::test]
async fn audit_sink_rejects_mismatched_tenant_context() -> Result<()> {
    let db = TestDb::new().await?;
    let tenant_id = tenant(&db).await?;
    let other_tenant = tenant(&db).await?;
    let actor = ActorCtx::system("audit-sink-test");
    let sink = AuditModule;
    let mut tx = db::begin(&db.app_pool, tenant_id, &actor).await?;

    let result = sink
        .record_module_toggle(&mut tx, other_tenant, &actor, ModuleId::Scheduling, true)
        .await;
    assert!(result.is_err());
    tx.rollback().await?;

    let persisted = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM audit.entries WHERE tenant_id = $1 AND action = 'core.module.enabled'",
    )
    .bind(tenant_id.0)
    .fetch_one(&db.pool)
    .await?;
    assert_eq!(persisted, 0);
    Ok(())
}

#[tokio::test]
async fn app_role_cannot_mutate() -> Result<()> {
    let db = TestDb::new().await?;

    let role = sqlx::query_scalar::<_, String>("SELECT current_user")
        .fetch_one(&db.app_pool)
        .await?;
    assert_eq!(role, "eq_app");

    let update = sqlx::query("UPDATE audit.entries SET after = '{}'::jsonb")
        .execute(&db.app_pool)
        .await;
    assert!(update.as_ref().is_err_and(is_permission_denied));

    let delete = sqlx::query("DELETE FROM audit.entries")
        .execute(&db.app_pool)
        .await;
    assert!(delete.as_ref().is_err_and(is_permission_denied));

    Ok(())
}

#[tokio::test]
async fn tamper_detected() -> Result<()> {
    let db = TestDb::new().await?;
    let tenant = tenant(&db).await?;
    let seq = record_entry(&db, tenant, None, Some(json!({ "status": "initial" })), None).await?;
    let sealer = Sealer::new(db.app_pool.clone());
    sealer.seal_tenant(tenant).await?;

    sqlx::query("UPDATE audit.entries SET after = $1 WHERE tenant_id = $2 AND seq = $3")
        .bind(sqlx::types::Json(json!({ "status": "tampered" })))
        .bind(tenant.0)
        .bind(seq)
        .execute(&db.pool)
        .await?;

    let report = sealer.verify(tenant, seq, seq).await?;
    assert_eq!(report.checked, 1);
    assert_eq!(report.first_mismatch, Some(seq));
    Ok(())
}

#[tokio::test]
async fn anonymize_keeps_chain_valid() -> Result<()> {
    let db = TestDb::new().await?;
    let tenant = tenant(&db).await?;
    let before = json!({
        "person": { "display_name": "test-person-109f", "contacts": [{ "phone": "test-phone-109f" }] }
    });
    let after = json!({ "notes": [{ "note": "test-minor-note-109f" }] });
    let ip: IpAddr = "192.0.2.10".parse()?;
    let seq = record_entry(&db, tenant, Some(before), Some(after), Some(ip)).await?;
    let sealer = Sealer::new(db.app_pool.clone());
    sealer.seal_tenant(tenant).await?;

    let cutoff: DateTime<Utc> = sqlx::query_scalar("SELECT pg_catalog.clock_timestamp()")
        .fetch_one(&db.pool)
        .await?;
    let actor = ActorCtx::system("audit-test");
    let mut tx = db::begin(&db.app_pool, tenant, &actor).await?;
    sqlx::query("SELECT audit.anonymize($1, $2, NULL::uuid)")
        .bind(tenant.0)
        .bind(cutoff)
        .execute(tx.conn())
        .await?;
    tx.commit().await?;

    let row = sqlx::query_as::<_, (Value, Value, Option<String>, Option<Vec<u8>>)>(
        "SELECT \"before\", \"after\", ip::text, redaction_proof \
         FROM audit.entries WHERE tenant_id = $1 AND seq = $2",
    )
    .bind(tenant.0)
    .bind(seq)
    .fetch_one(&db.pool)
    .await?;
    let before_text = row.0.to_string();
    let after_text = row.1.to_string();
    assert!(!before_text.contains("test-person-109f"));
    assert!(!before_text.contains("test-phone-109f"));
    assert!(!after_text.contains("test-minor-note-109f"));
    assert!(before_text.contains("REDACTED-"));
    assert!(after_text.contains("REDACTED-"));
    assert!(row.2.is_none());
    assert!(row.3.is_some());

    let report = sealer.verify(tenant, seq, seq).await?;
    assert_eq!(report.checked, 1);
    assert_eq!(report.anonymized, 1);
    assert_eq!(report.first_mismatch, None);
    Ok(())
}

#[tokio::test]
async fn purge_with_checkpoint() -> Result<()> {
    let db = TestDb::new().await?;
    let tenant = tenant(&db).await?;
    let expired_seq = record_entry(&db, tenant, None, Some(json!({ "kind": "expired" })), None).await?;
    let cutoff: DateTime<Utc> = sqlx::query_scalar("SELECT pg_catalog.clock_timestamp()")
        .fetch_one(&db.pool)
        .await?;
    let retained_seq = record_entry(&db, tenant, None, Some(json!({ "kind": "retained" })), None).await?;
    let sealer = Sealer::new(db.app_pool.clone());
    sealer.seal_tenant(tenant).await?;
    let expected_hash: Vec<u8> =
        sqlx::query_scalar("SELECT hash FROM audit.entries WHERE tenant_id = $1 AND seq = $2")
            .bind(tenant.0)
            .bind(expired_seq)
            .fetch_one(&db.pool)
            .await?;

    let actor = ActorCtx::system("audit-test");
    let mut tx = db::begin(&db.app_pool, tenant, &actor).await?;
    sqlx::query("SELECT audit.purge_expired($1, $2)")
        .bind(tenant.0)
        .bind(cutoff)
        .execute(tx.conn())
        .await?;
    tx.commit().await?;

    let remaining: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM audit.entries WHERE tenant_id = $1 AND seq = $2)")
            .bind(tenant.0)
            .bind(retained_seq)
            .fetch_one(&db.pool)
            .await?;
    let deleted: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM audit.entries WHERE tenant_id = $1 AND seq = $2)")
            .bind(tenant.0)
            .bind(expired_seq)
            .fetch_one(&db.pool)
            .await?;
    let checkpoint = sqlx::query_as::<_, (i64, Vec<u8>)>(
        "SELECT seq, hash FROM audit.checkpoints WHERE tenant_id = $1 ORDER BY seq DESC LIMIT 1",
    )
    .bind(tenant.0)
    .fetch_one(&db.pool)
    .await?;
    assert!(remaining);
    assert!(!deleted);
    assert_eq!(checkpoint.0, expired_seq);
    assert_eq!(checkpoint.1, expected_hash);

    let report = sealer.verify(tenant, retained_seq, retained_seq).await?;
    assert_eq!(report.checked, 1);
    assert_eq!(report.first_mismatch, None);
    Ok(())
}

#[tokio::test]
async fn concurrent_sealers() -> Result<()> {
    let db = TestDb::new().await?;
    let tenant = tenant(&db).await?;
    let first_seq = record_entry(&db, tenant, None, Some(json!({ "order": 1 })), None).await?;
    let second_seq = record_entry(&db, tenant, None, Some(json!({ "order": 2 })), None).await?;
    assert!(first_seq < second_seq);

    let first = Sealer::new(db.app_pool.clone());
    let second = Sealer::new(db.app_pool.clone());
    let (first_result, second_result) = tokio::join!(first.seal_tenant(tenant), second.seal_tenant(tenant));
    first_result?;
    second_result?;

    let report = first.verify(tenant, first_seq, second_seq).await?;
    assert_eq!(report.checked, 2);
    assert_eq!(report.first_mismatch, None);
    Ok(())
}

#[tokio::test]
async fn concurrent_writer_and_sealer_preserve_sequence_order() -> Result<()> {
    let db = TestDb::new().await?;
    let tenant = tenant(&db).await?;
    let request_id = Uuid::now_v7();
    let actor = ActorCtx {
        person_id: None,
        role: ActorRole::System,
        ip: None,
        request_id: Some(request_id),
    };
    let mut writer_tx = db::begin(&db.app_pool, tenant, &actor).await?;
    record(
        &mut writer_tx,
        AuditEvent {
            action: "test.audit.writer_sealer",
            category: Category::Operational,
            entity_type: "test",
            entity_id: None,
            before: None,
            after: Some(json!({ "sequence": "writer" })),
            actor: &actor,
        },
    )
    .await?;

    let sequence = sqlx::query_scalar::<_, i64>(
        "SELECT seq FROM audit.entries WHERE tenant_id = $1 AND request_id = $2",
    )
    .bind(tenant.0)
    .bind(request_id)
    .fetch_one(writer_tx.conn())
    .await?;
    let lock_key = format!("audit.seal:{}", tenant.0);
    let lock_available = sqlx::query_scalar::<_, bool>(
        "SELECT pg_catalog.pg_try_advisory_xact_lock(pg_catalog.hashtext($1)::bigint)",
    )
    .bind(lock_key)
    .fetch_one(&db.pool)
    .await?;
    assert!(!lock_available, "record must hold the tenant lock until commit");

    let sealer = Sealer::new(db.app_pool.clone());
    let seal_task = tokio::spawn(async move { sealer.seal_tenant(tenant).await });

    tokio::time::timeout(SEALER_LOCK_WAIT_TIMEOUT, async {
        loop {
            let sealer_waiting = sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS (\
                     SELECT 1 FROM pg_catalog.pg_stat_activity \
                     WHERE datname = pg_catalog.current_database() \
                       AND pid <> pg_catalog.pg_backend_pid() \
                       AND wait_event_type = 'Lock' \
                       AND query LIKE '%pg_catalog.pg_advisory_xact_lock%'\
                 )",
            )
            .fetch_one(&db.pool)
            .await?;
            if sealer_waiting {
                break;
            }
            tokio::time::sleep(SEALER_ACTIVITY_POLL_INTERVAL).await;
        }
        Ok::<(), Error>(())
    })
    .await??;

    writer_tx.commit().await?;
    seal_task.await??;

    let report = Sealer::new(db.app_pool.clone())
        .verify(tenant, sequence, sequence)
        .await?;
    assert_eq!(report.checked, 1);
    assert_eq!(report.first_mismatch, None);
    Ok(())
}

#[tokio::test]
async fn audit_functions_reject_cross_tenant_arguments() -> Result<()> {
    let db = TestDb::new().await?;
    let tenant_id = tenant(&db).await?;
    let other_tenant = tenant(&db).await?;
    let statements = [
        "SELECT audit.seal($1, '[]'::jsonb)",
        "SELECT audit.anonymize($1, pg_catalog.clock_timestamp(), NULL::uuid)",
        "SELECT audit.purge_expired($1, pg_catalog.clock_timestamp())",
    ];

    for statement in statements {
        let actor = ActorCtx::system("audit-test");
        let mut tx = db::begin(&db.app_pool, tenant_id, &actor).await?;
        let result = sqlx::query(statement)
            .bind(other_tenant.0)
            .execute(tx.conn())
            .await;
        assert!(result.as_ref().is_err_and(is_permission_denied));
        tx.rollback().await?;
    }
    Ok(())
}

#[tokio::test]
async fn audit_retention_job_uses_policy_store() -> Result<()> {
    let db = TestDb::new().await?;
    let tenant = tenant(&db).await?;
    let purged_seq = record_entry(&db, tenant, None, Some(json!({ "value": "old-entry" })), None).await?;
    let retained_seq = record_entry(
        &db,
        tenant,
        None,
        Some(json!({
            "person": { "display_name": "test-person-109g", "reason": "test-minor-note-109g" }
        })),
        Some("192.0.2.10".parse()?),
    )
    .await?;
    let now =
        sqlx::query_scalar::<_, DateTime<Utc>>("SELECT pg_catalog.clock_timestamp() + interval '1 second'")
            .fetch_one(&db.pool)
            .await?;
    let retained_at = now
        .checked_sub_months(Months::new(18))
        .ok_or_else(|| anyhow::anyhow!("test retention timestamp is in range"))?;
    let purged_at = now
        .checked_sub_months(Months::new(72))
        .ok_or_else(|| anyhow::anyhow!("test purge timestamp is in range"))?;
    sqlx::query("UPDATE audit.entries SET at = $3 WHERE tenant_id = $1 AND seq = $2")
        .bind(tenant.0)
        .bind(retained_seq)
        .bind(retained_at)
        .execute(&db.pool)
        .await?;
    sqlx::query("UPDATE audit.entries SET at = $3 WHERE tenant_id = $1 AND seq = $2")
        .bind(tenant.0)
        .bind(purged_seq)
        .bind(purged_at)
        .execute(&db.pool)
        .await?;
    Sealer::new(db.app_pool.clone()).seal_tenant(tenant).await?;

    let policy_store = policy_store(&db).await?;
    run_retention_job(&db, tenant, now, policy_store).await?;

    let retained = sqlx::query(
        "SELECT \"after\", ip::text AS ip, redaction_proof \
         FROM audit.entries WHERE tenant_id = $1 AND seq = $2",
    )
    .bind(tenant.0)
    .bind(retained_seq)
    .fetch_one(&db.pool)
    .await?;
    let after = retained
        .try_get::<Option<Value>, _>("after")?
        .ok_or_else(|| anyhow::anyhow!("retained test entry has after JSON"))?;
    let after_text = after.to_string();
    assert!(!after_text.contains("test-person-109g"));
    assert!(!after_text.contains("test-minor-note-109g"));
    assert!(after_text.contains("REDACTED-"));
    assert!(retained.try_get::<Option<String>, _>("ip")?.is_none());
    assert!(
        retained
            .try_get::<Option<Vec<u8>>, _>("redaction_proof")?
            .is_some()
    );

    let purged = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM audit.entries WHERE tenant_id = $1 AND seq = $2)",
    )
    .bind(tenant.0)
    .bind(purged_seq)
    .fetch_one(&db.pool)
    .await?;
    assert!(!purged);
    let checkpoint_seq = sqlx::query_scalar::<_, i64>(
        "SELECT seq FROM audit.checkpoints WHERE tenant_id = $1 ORDER BY seq DESC LIMIT 1",
    )
    .bind(tenant.0)
    .fetch_one(&db.pool)
    .await?;
    assert_eq!(checkpoint_seq, purged_seq);

    let report = Sealer::new(db.app_pool.clone())
        .verify(tenant, retained_seq, retained_seq)
        .await?;
    assert_eq!(report.first_mismatch, None);
    Ok(())
}

async fn run_retention_job(
    db: &TestDb,
    tenant: TenantId,
    now: DateTime<Utc>,
    policy_store: Arc<PolicyStore>,
) -> Result<()> {
    let actor = ActorCtx::system("audit-test");
    let policy = AuditPolicy::new(5, 1, vec![Category::Operational])
        .map_err(|_| anyhow::anyhow!("test audit policy is valid"))?;
    let mut policy_tx = db::begin(&db.app_pool, tenant, &actor).await?;
    policy_store
        .put(&mut policy_tx, tenant, policy, now, &actor, None)
        .await?;
    policy_tx.commit().await?;

    let ctx = JobCtx {
        tenant: Some(Arc::new(tenant_info(tenant))),
        pool: db.app_pool.clone(),
        clock: Arc::new(SystemClock),
        now,
        policy_store: Some(policy_store),
    };
    AuditRetentionJob
        .run(&ctx, &Value::Null)
        .await
        .map_err(anyhow::Error::new)
}

#[tokio::test]
async fn audit_verify_job_raises_attention_on_mismatch() -> Result<()> {
    let db = TestDb::new().await?;
    let tenant = tenant(&db).await?;
    let seq = record_entry(&db, tenant, None, Some(json!({ "value": "initial" })), None).await?;
    Sealer::new(db.app_pool.clone()).seal_tenant(tenant).await?;
    sqlx::query("UPDATE audit.entries SET after = $1 WHERE tenant_id = $2 AND seq = $3")
        .bind(sqlx::types::Json(json!({ "value": "tampered" })))
        .bind(tenant.0)
        .bind(seq)
        .execute(&db.pool)
        .await?;
    let now =
        sqlx::query_scalar::<_, DateTime<Utc>>("SELECT pg_catalog.clock_timestamp() + interval '1 second'")
            .fetch_one(&db.pool)
            .await?;

    let ctx = JobCtx {
        tenant: Some(Arc::new(tenant_info(tenant))),
        pool: db.app_pool.clone(),
        clock: Arc::new(SystemClock),
        now,
        policy_store: None,
    };
    AuditVerifyRecentJob
        .run(&ctx, &Value::Null)
        .await
        .map_err(anyhow::Error::new)?;

    let item = sqlx::query(
        "SELECT module_id, kind, severity, payload, dedupe_key \
         FROM public.attention_items WHERE tenant_id = $1 AND kind = 'audit_chain_mismatch'",
    )
    .bind(tenant.0)
    .fetch_one(&db.pool)
    .await?;
    assert_eq!(item.try_get::<String, _>("module_id")?, "audit");
    assert_eq!(item.try_get::<String, _>("kind")?, "audit_chain_mismatch");
    assert_eq!(item.try_get::<String, _>("severity")?, "high");
    assert_eq!(item.try_get::<Value, _>("payload")?["sequence"], seq);
    assert_eq!(
        item.try_get::<String, _>("dedupe_key")?,
        format!("audit_chain_mismatch:{seq}")
    );
    Ok(())
}

#[derive(Debug)]
struct NoPresetPatches;

impl PresetPatchSource for NoPresetPatches {
    fn policy_patch(
        &self,
        _preset_id: &str,
        _preset_version: i32,
        _module: ModuleId,
    ) -> std::result::Result<Option<Value>, PresetSourceError> {
        Ok(None)
    }
}

async fn policy_store(db: &TestDb) -> Result<Arc<PolicyStore>> {
    let sinks = ModuleSinkDoubles::default();
    let modules = Arc::new(ModuleRegistry::new(vec![
        Arc::new(AuditModule) as Arc<dyn eq_core::module::Module>
    ]));
    let store = PolicyStore::new(
        db.app_pool.clone(),
        modules,
        Arc::new(NoPresetPatches),
        sinks.audit_sink(),
        sinks.event_sink(),
        Arc::new(SystemClock),
    )
    .await?;
    Ok(Arc::new(store))
}

fn tenant_info(tenant: TenantId) -> TenantInfo {
    TenantInfo {
        id: tenant,
        slug: String::from("audit-test"),
        status: TenantStatus::Active,
        timezone: String::from("UTC"),
        locale: Locale::En,
        currency: Currency::IDR,
    }
}

async fn record_entry(
    db: &TestDb,
    tenant: TenantId,
    before: Option<Value>,
    after: Option<Value>,
    ip: Option<IpAddr>,
) -> Result<i64> {
    let request_id = Uuid::now_v7();
    let actor = ActorCtx {
        person_id: None,
        role: ActorRole::System,
        ip,
        request_id: Some(request_id),
    };
    let mut tx = db::begin(&db.app_pool, tenant, &actor).await?;
    record(
        &mut tx,
        AuditEvent {
            action: "test.audit.chain",
            category: Category::Operational,
            entity_type: "test",
            entity_id: Some(Uuid::now_v7()),
            before,
            after,
            actor: &actor,
        },
    )
    .await?;
    tx.commit().await?;
    let seq = sqlx::query_scalar::<_, i64>(
        "SELECT seq FROM audit.entries WHERE tenant_id = $1 AND request_id = $2",
    )
    .bind(tenant.0)
    .bind(request_id)
    .fetch_one(&db.pool)
    .await?;
    Ok(seq)
}

fn is_permission_denied(error: &Error) -> bool {
    error
        .as_database_error()
        .and_then(|database_error| database_error.code())
        .as_deref()
        == Some("42501")
}
