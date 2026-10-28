use anyhow::Result;
use chrono::{DateTime, Utc};
use eq_audit::{AuditEvent, Category, Sealer, record};
use eq_core::actor::{ActorCtx, ActorRole};
use eq_core::db;
use eq_core::ids::TenantId;
use eq_testkit::TestDb;
use serde_json::json;
use sqlx::Error;
use std::net::IpAddr;
use uuid::Uuid;

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
    let db = TestDb::new().await;
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
async fn app_role_cannot_mutate() -> Result<()> {
    let db = TestDb::new().await;

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
    let db = TestDb::new().await;
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
    let db = TestDb::new().await;
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

    let row = sqlx::query_as::<
        _,
        (
            serde_json::Value,
            serde_json::Value,
            Option<String>,
            Option<Vec<u8>>,
        ),
    >(
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
    let db = TestDb::new().await;
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
    let db = TestDb::new().await;
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
async fn audit_functions_reject_cross_tenant_arguments() -> Result<()> {
    let db = TestDb::new().await;
    let tenant = tenant(&db).await?;
    let other_tenant = tenant(&db).await?;
    let statements = [
        "SELECT audit.seal($1, '[]'::jsonb)",
        "SELECT audit.anonymize($1, pg_catalog.clock_timestamp(), NULL::uuid)",
        "SELECT audit.purge_expired($1, pg_catalog.clock_timestamp())",
    ];

    for statement in statements {
        let actor = ActorCtx::system("audit-test");
        let mut tx = db::begin(&db.app_pool, tenant, &actor).await?;
        let result = sqlx::query(statement)
            .bind(other_tenant.0)
            .execute(tx.conn())
            .await;
        assert!(result.as_ref().is_err_and(is_permission_denied));
        tx.rollback().await?;
    }
    Ok(())
}

async fn record_entry(
    db: &TestDb,
    tenant: TenantId,
    before: Option<serde_json::Value>,
    after: Option<serde_json::Value>,
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
