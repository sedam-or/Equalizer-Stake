use anyhow::Result;
use eq_testkit::{TestDb, api::TestApp};
use secrecy::{ExposeSecret, SecretString};
use sqlx::{Row, postgres::PgPoolOptions};

#[tokio::test]
async fn independent_dbs() -> Result<()> {
    let (first, second) = tokio::join!(TestDb::new(), TestDb::new());

    sqlx::query("CREATE TABLE test_db_isolation_probe (label text NOT NULL)")
        .execute(&first.pool)
        .await?;
    sqlx::query("INSERT INTO test_db_isolation_probe (label) VALUES ($1)")
        .bind("first-only")
        .execute(&first.pool)
        .await?;

    let first_count =
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM test_db_isolation_probe WHERE label = $1")
            .bind("first-only")
            .fetch_one(&first.pool)
            .await?;
    let second_has_table = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM pg_tables WHERE schemaname = 'public' AND tablename = $1)",
    )
    .bind("test_db_isolation_probe")
    .fetch_one(&second.pool)
    .await?;

    assert_eq!(first_count, 1);
    assert!(!second_has_table);
    Ok(())
}

#[tokio::test]
async fn template_has_migrations() -> Result<()> {
    let db = TestDb::new().await;
    let migration_count =
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM _sqlx_migrations WHERE success = TRUE")
            .fetch_one(&db.pool)
            .await?;

    assert!(migration_count > 0);
    Ok(())
}

#[tokio::test]
async fn app_pool_enforces_rls() -> Result<()> {
    let db = TestDb::new().await;
    let role =
        sqlx::query("SELECT rolcanlogin, rolsuper, rolbypassrls FROM pg_roles WHERE rolname = 'eq_app'")
            .fetch_one(&db.pool)
            .await?;
    assert!(role.get::<bool, _>("rolcanlogin"));
    assert!(!role.get::<bool, _>("rolsuper"));
    assert!(!role.get::<bool, _>("rolbypassrls"));

    sqlx::query("CREATE TABLE tenant_rls_probe (tenant_id uuid NOT NULL, label text NOT NULL)")
        .execute(&db.pool)
        .await?;
    sqlx::query("SELECT eq_enable_rls('public.tenant_rls_probe'::regclass)")
        .fetch_one(&db.pool)
        .await?;
    sqlx::query("GRANT SELECT ON tenant_rls_probe TO eq_app")
        .execute(&db.pool)
        .await?;
    sqlx::query("INSERT INTO tenant_rls_probe (tenant_id, label) VALUES ($1, $2), ($3, $4)")
        .bind(uuid::Uuid::from_u128(1))
        .bind("first-tenant")
        .bind(uuid::Uuid::from_u128(2))
        .bind("second-tenant")
        .execute(&db.pool)
        .await?;

    let app_visible = sqlx::query_scalar::<_, i64>("SELECT count(*) FROM tenant_rls_probe")
        .fetch_one(&db.app_pool)
        .await?;
    let owner_visible = sqlx::query_scalar::<_, i64>("SELECT count(*) FROM tenant_rls_probe")
        .fetch_one(&db.pool)
        .await?;

    assert_eq!(app_visible, 0);
    assert_eq!(owner_visible, 2);
    Ok(())
}

#[tokio::test]
async fn debug_redacts_database_urls() -> Result<()> {
    let url = SecretString::from("postgres://postgres:test-secret-owner@localhost:5432/ritma");
    let app_url = SecretString::from("postgres://eq_app:test-secret-app@localhost:5432/ritma");
    let pool = PgPoolOptions::new().connect_lazy(url.expose_secret())?;
    let app_pool = PgPoolOptions::new().connect_lazy(app_url.expose_secret())?;
    let db = TestDb {
        pool,
        url,
        app_pool,
        app_url,
    };

    let formatted = format!("{db:?}");
    assert!(!formatted.contains("test-secret-owner"));
    assert!(!formatted.contains("test-secret-app"));
    assert!(!formatted.contains("postgres://"));
    Ok(())
}

#[tokio::test]
async fn test_app_uses_app_pool() {
    let app = TestApp::new().await;
    let formatted = format!("{app:?}");
    assert!(!formatted.contains("postgres://"));
    assert!(!formatted.contains("test-secret-app"));
    app.server.get("/readyz").await.assert_status_ok();
}
