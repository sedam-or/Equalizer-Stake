use anyhow::Result;
use eq_audit::{AuditEvent, Category, record};
use eq_core::actor::ActorCtx;
use eq_core::db;
use eq_core::ids::TenantId;
use eq_testkit::TestDb;
use serde_json::json;
use sqlx::Error;
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

fn is_permission_denied(error: &Error) -> bool {
    error
        .as_database_error()
        .and_then(|database_error| database_error.code())
        .as_deref()
        == Some("42501")
}
