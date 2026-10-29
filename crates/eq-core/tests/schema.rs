use anyhow::{Result, anyhow};
use eq_core::ids::{PersonId, TenantId};
use eq_core::model::{Closure, Tenant, TenantStatus};
use secrecy::ExposeSecret;
use sqlx::Row;
use uuid::Uuid;

#[path = "../../eq-testkit/src/db.rs"]
#[expect(
    clippy::panic,
    reason = "the shared database fixture reports only redacted setup failures"
)]
mod shared_test_db;

use shared_test_db::TestDb;

const TENANT_RLS_TABLES: &[&str] = &[
    "locations",
    "resources",
    "persons",
    "memberships",
    "person_links",
    "tenant_modules",
    "closures",
    "policies",
    "files",
    "drafts",
    "attention_items",
    "consents",
    "idempotency_keys",
];

fn test_uuid() -> Uuid {
    Uuid::now_v7()
}

async fn create_tenant(db: &TestDb, id: TenantId, slug: &str) -> Result<()> {
    sqlx::query(
        "INSERT INTO tenants (id, slug, name, country, preset_id, preset_version) \
         VALUES ($1, $2, 'Test School', 'ID', 'test', 1)",
    )
    .bind(id)
    .bind(slug)
    .execute(&db.pool)
    .await?;
    Ok(())
}

#[tokio::test]
async fn migrations_apply() -> Result<()> {
    let db = TestDb::new().await;
    assert!(db.url.expose_secret().starts_with("postgres://postgres:"));
    assert!(db.app_url.expose_secret().starts_with("postgres://eq_app:"));
    let expected_tables = [
        "tenants",
        "locations",
        "resources",
        "persons",
        "memberships",
        "person_links",
        "tenant_modules",
        "closures",
        "policies",
        "files",
        "drafts",
        "attention_items",
        "consents",
        "tenant_keys",
        "idempotency_keys",
    ];

    for table in expected_tables {
        let exists: bool = sqlx::query_scalar("SELECT to_regclass($1) IS NOT NULL")
            .bind(format!("public.{table}"))
            .fetch_one(&db.pool)
            .await?;
        assert!(exists, "missing table {table}");
    }

    let migrated_ics_column: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM pg_catalog.pg_attribute \
         WHERE attrelid = 'public.persons'::regclass AND attname = 'ics_secret_version' \
           AND NOT attisdropped)",
    )
    .fetch_one(&db.pool)
    .await?;
    assert!(!migrated_ics_column, "T-211 owns the later ics migration");

    let tenant_id = TenantId(test_uuid());
    create_tenant(&db, tenant_id, &format!("model{}", Uuid::now_v7().simple())).await?;
    let tenant = sqlx::query_as::<_, Tenant>(
        "SELECT id, slug, name, country, preset_id, preset_version, setup_step, \
                setup_completed_at, status, locale, currency, timezone, created_at \
         FROM tenants WHERE id = $1",
    )
    .bind(tenant_id)
    .fetch_one(&db.pool)
    .await?;
    assert_eq!(tenant.id, tenant_id);
    assert_eq!(tenant.status, TenantStatus::Active);

    let closure_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO closures (id, tenant_id, dates, name) \
         VALUES ($1, $2, daterange('2026-10-25', '2026-10-26', '[)'), '{}'::jsonb)",
    )
    .bind(closure_id)
    .bind(tenant_id)
    .execute(&db.pool)
    .await?;
    let closure = sqlx::query_as::<_, Closure>(
        "SELECT id, tenant_id, location_id, dates, name, kind, created_at \
         FROM closures WHERE id = $1",
    )
    .bind(closure_id)
    .fetch_one(&db.pool)
    .await?;
    let serialized = serde_json::to_vec(&closure)?;
    let deserialized: Closure = serde_json::from_slice(&serialized)?;
    assert_eq!(closure, deserialized);
    Ok(())
}

#[tokio::test]
async fn rls_enabled_on_tenant_tables() -> Result<()> {
    let db = TestDb::new().await;
    let rows = sqlx::query(
        "SELECT c.relname, c.relrowsecurity, c.relforcerowsecurity \
         FROM pg_catalog.pg_class AS c \
         JOIN pg_catalog.pg_namespace AS n ON n.oid = c.relnamespace \
         WHERE n.nspname = 'public' AND c.relname = ANY($1) ORDER BY c.relname",
    )
    .bind(
        TENANT_RLS_TABLES
            .iter()
            .map(|name| (*name).to_owned())
            .collect::<Vec<_>>(),
    )
    .fetch_all(&db.pool)
    .await?;
    assert_eq!(rows.len(), TENANT_RLS_TABLES.len());
    for row in rows {
        let table: String = row.try_get("relname")?;
        let enabled: bool = row.try_get("relrowsecurity")?;
        let forced: bool = row.try_get("relforcerowsecurity")?;
        assert!(enabled, "RLS is not enabled on {table}");
        assert!(forced, "RLS is not forced on {table}");
    }
    let no_rls_tables = ["tenants", "tenant_keys"].map(str::to_owned).to_vec();
    let rows = sqlx::query(
        "SELECT c.relname, c.relrowsecurity, c.relforcerowsecurity \
         FROM pg_catalog.pg_class AS c \
         JOIN pg_catalog.pg_namespace AS n ON n.oid = c.relnamespace \
         WHERE n.nspname = 'public' AND c.relname = ANY($1) ORDER BY c.relname",
    )
    .bind(no_rls_tables)
    .fetch_all(&db.pool)
    .await?;
    assert_eq!(rows.len(), 2);
    for row in rows {
        let table: String = row.try_get("relname")?;
        let enabled: bool = row.try_get("relrowsecurity")?;
        let forced: bool = row.try_get("relforcerowsecurity")?;
        assert!(!enabled, "RLS is not part of the contract for {table}");
        assert!(!forced, "RLS is not part of the contract for {table}");
    }

    let tenant_id = TenantId(test_uuid());
    create_tenant(&db, tenant_id, &format!("scope{}", Uuid::now_v7().simple())).await?;
    let location_id = Uuid::now_v7();
    sqlx::query("INSERT INTO locations (id, tenant_id, name) VALUES ($1, $2, 'Private test location')")
        .bind(location_id)
        .bind(tenant_id)
        .execute(&db.pool)
        .await?;
    let visible: i64 = sqlx::query_scalar("SELECT count(*) FROM locations WHERE id = $1")
        .bind(location_id)
        .fetch_one(&db.app_pool)
        .await?;
    assert_eq!(visible, 0, "unscoped eq_app read must be hidden by RLS");
    Ok(())
}

#[tokio::test]
async fn one_primary_guardian() -> Result<()> {
    let db = TestDb::new().await;
    let tenant_id = TenantId(test_uuid());
    create_tenant(&db, tenant_id, &format!("guardian{}", Uuid::now_v7().simple())).await?;
    let guardian_a = PersonId(test_uuid());
    let guardian_b = PersonId(test_uuid());
    let student = PersonId(test_uuid());
    for (person_id, name) in [
        (guardian_a, "Guardian A"),
        (guardian_b, "Guardian B"),
        (student, "Student"),
    ] {
        sqlx::query("INSERT INTO persons (id, tenant_id, display_name) VALUES ($1, $2, $3)")
            .bind(person_id)
            .bind(tenant_id)
            .bind(name)
            .execute(&db.pool)
            .await?;
    }
    sqlx::query("INSERT INTO person_links (tenant_id, guardian_id, student_id) VALUES ($1, $2, $3)")
        .bind(tenant_id)
        .bind(guardian_a)
        .bind(student)
        .execute(&db.pool)
        .await?;

    let duplicate =
        sqlx::query("INSERT INTO person_links (tenant_id, guardian_id, student_id) VALUES ($1, $2, $3)")
            .bind(tenant_id)
            .bind(guardian_b)
            .bind(student)
            .execute(&db.pool)
            .await;
    match duplicate {
        Err(error) => {
            let code = error.as_database_error().and_then(|db_error| db_error.code());
            assert_eq!(code.as_deref(), Some("23505"));
        }
        Ok(_) => return Err(anyhow!("a second primary guardian link unexpectedly succeeded")),
    }
    Ok(())
}

#[tokio::test]
async fn single_tenant_guard() -> Result<()> {
    let db = TestDb::new().await;
    let direct_tenant_read = sqlx::query("SELECT id FROM public.tenants")
        .fetch_all(&db.app_pool)
        .await;
    match direct_tenant_read {
        Err(error) => {
            let code = error.as_database_error().and_then(|db_error| db_error.code());
            assert_eq!(code.as_deref(), Some("42501"));
        }
        Ok(_) => return Err(anyhow!("eq_app unexpectedly read tenants directly")),
    }

    let empty = sqlx::query("SELECT id, slug, status FROM public.resolve_single_tenant()")
        .fetch_all(&db.app_pool)
        .await?;
    assert!(empty.is_empty());

    let first = TenantId(test_uuid());
    let first_slug = format!("single{}", Uuid::now_v7().simple());
    create_tenant(&db, first, &first_slug).await?;
    let resolved = sqlx::query("SELECT id, slug, status FROM public.resolve_single_tenant()")
        .fetch_one(&db.app_pool)
        .await?;
    assert_eq!(resolved.try_get::<Uuid, _>("id")?, first.0);
    assert_eq!(resolved.try_get::<String, _>("slug")?, first_slug);
    assert_eq!(resolved.try_get::<String, _>("status")?, "active");

    let by_slug = sqlx::query("SELECT id, slug, status FROM public.resolve_tenant_by_slug($1)")
        .bind(&first_slug)
        .fetch_one(&db.app_pool)
        .await?;
    assert_eq!(by_slug.try_get::<Uuid, _>("id")?, first.0);

    create_tenant(
        &db,
        TenantId(test_uuid()),
        &format!("second{}", Uuid::now_v7().simple()),
    )
    .await?;
    let multiple = sqlx::query("SELECT id, slug, status FROM public.resolve_single_tenant()")
        .fetch_all(&db.app_pool)
        .await;
    match multiple {
        Err(error) => {
            let code = error.as_database_error().and_then(|db_error| db_error.code());
            assert_eq!(code.as_deref(), Some("21000"));
        }
        Ok(_) => return Err(anyhow!("single-tenant resolver accepted multiple tenants")),
    }
    Ok(())
}

#[tokio::test]
async fn policies_append_only_for_app_role() -> Result<()> {
    let db = TestDb::new().await;
    let tenant_id = TenantId(test_uuid());
    create_tenant(&db, tenant_id, &format!("policy{}", Uuid::now_v7().simple())).await?;
    sqlx::query(
        "INSERT INTO policies (tenant_id, module_id, version, schema_version, effective_from, document) \
         VALUES ($1, 'core', 1, 1, now(), '{}'::jsonb)",
    )
    .bind(tenant_id)
    .execute(&db.pool)
    .await?;

    let update = sqlx::query(
        "UPDATE policies SET note = 'must fail' WHERE tenant_id = $1 AND module_id = 'core' AND version = 1",
    )
    .bind(tenant_id)
    .execute(&db.app_pool)
    .await;
    match update {
        Err(error) => {
            let code = error.as_database_error().and_then(|db_error| db_error.code());
            assert_eq!(code.as_deref(), Some("42501"));
        }
        Ok(_) => return Err(anyhow!("eq_app unexpectedly updated append-only policy data")),
    }
    Ok(())
}
