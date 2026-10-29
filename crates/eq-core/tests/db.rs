use anyhow::Result;
use eq_core::actor::{ActorCtx, ActorRole};
use eq_core::db::{begin, begin_system, run_migrations};
use eq_core::ids::{PersonId, TenantId};
use secrecy::ExposeSecret;
use sqlx::Row;
use std::net::IpAddr;
use uuid::Uuid;

#[path = "../../eq-testkit/src/db.rs"]
#[expect(
    clippy::panic,
    reason = "the shared database fixture reports only redacted setup failures"
)]
mod shared_test_db;

use shared_test_db::TestDb;

#[tokio::test]
async fn bootstrap_idempotent() -> Result<()> {
    let db = TestDb::new().await;
    assert!(db.app_url.expose_secret().starts_with("postgres://eq_app:"));

    run_migrations(db.url.expose_secret()).await?;
    run_migrations(db.url.expose_secret()).await?;

    Ok(())
}

#[tokio::test]
async fn rls_scopes_rows() -> Result<()> {
    let db = TestDb::new().await;

    sqlx::query("CREATE TABLE tenant_rls_probe (tenant_id uuid NOT NULL, label text NOT NULL)")
        .execute(&db.pool)
        .await?;
    sqlx::query("SELECT eq_enable_rls('public.tenant_rls_probe'::regclass)")
        .fetch_one(&db.pool)
        .await?;
    sqlx::query("GRANT SELECT, INSERT ON tenant_rls_probe TO eq_app")
        .execute(&db.pool)
        .await?;

    let tenant_a = Uuid::from_u128(1);
    let tenant_b = Uuid::from_u128(2);
    sqlx::query("INSERT INTO tenant_rls_probe (tenant_id, label) VALUES ($1, $2), ($3, $4)")
        .bind(tenant_a)
        .bind("tenant-a")
        .bind(tenant_b)
        .bind("tenant-b")
        .execute(&db.pool)
        .await?;

    let role = sqlx::query("SELECT rolsuper, rolbypassrls FROM pg_roles WHERE rolname = $1")
        .bind("eq_app")
        .fetch_one(&db.pool)
        .await?;
    assert!(!role.get::<bool, _>("rolsuper"));
    assert!(!role.get::<bool, _>("rolbypassrls"));

    let actor = ActorCtx {
        person_id: Some(PersonId(Uuid::from_u128(3))),
        role: ActorRole::Owner,
        ip: None::<IpAddr>,
        request_id: None,
    };
    let mut transaction = begin(&db.app_pool, TenantId(tenant_a), &actor).await?;
    let visible = sqlx::query("SELECT label FROM tenant_rls_probe ORDER BY label")
        .fetch_all(transaction.conn())
        .await?;
    assert_eq!(visible.len(), 1);
    assert_eq!(visible[0].get::<String, _>("label"), "tenant-a");
    transaction.commit().await?;

    let mut connection = db.app_pool.acquire().await?;
    let visible_without_tenant = sqlx::query("SELECT label FROM tenant_rls_probe ORDER BY label")
        .fetch_all(&mut *connection)
        .await?;
    assert!(visible_without_tenant.is_empty());
    Ok(())
}

#[tokio::test]
async fn settings_do_not_leak() -> Result<()> {
    let db = TestDb::new().await;
    let tenant = TenantId(Uuid::from_u128(4));

    let mut transaction = begin_system(&db.app_pool, tenant).await?;
    let actor_id: Option<String> = sqlx::query_scalar("SELECT current_setting('app.actor_id', true)")
        .fetch_one(transaction.conn())
        .await?;
    assert_eq!(actor_id.as_deref(), Some(""));
    transaction.commit().await?;

    let mut connection = db.app_pool.acquire().await?;
    let settings: (Option<String>, Option<String>) = sqlx::query_as(
        "SELECT current_setting('app.tenant_id', true) AS tenant_id, \
                current_setting('app.actor_id', true) AS actor_id",
    )
    .fetch_one(&mut *connection)
    .await?;
    assert!(settings.0.as_deref().unwrap_or_default().is_empty());
    assert!(settings.1.as_deref().unwrap_or_default().is_empty());
    Ok(())
}
