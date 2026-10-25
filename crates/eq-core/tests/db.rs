use eq_core::actor::{ActorCtx, ActorRole};
use eq_core::config::{BackupSchedule, Config, FilesBackend, LogFormat, Mode};
use eq_core::db::{begin, begin_system, connect, run_migrations};
use eq_core::ids::{PersonId, TenantId};
use secrecy::{SecretSlice, SecretString};
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use testcontainers::core::{IntoContainerPort, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage, ImageExt};
use uuid::Uuid;

const TEST_POSTGRES_PASSWORD: &str = "test-secret-postgres";
const TEST_APP_PASSWORD: &str = "test-secret-app";

async fn start_postgres() -> anyhow::Result<(ContainerAsync<GenericImage>, String, PgPool)> {
    let image = GenericImage::new("postgres", "16-alpine")
        .with_exposed_port(5432.tcp())
        .with_wait_for(WaitFor::message_on_stderr(
            "database system is ready to accept connections",
        ))
        .with_wait_for(WaitFor::message_on_stdout(
            "database system is ready to accept connections",
        ))
        .with_env_var("POSTGRES_USER", "postgres")
        .with_env_var("POSTGRES_PASSWORD", TEST_POSTGRES_PASSWORD)
        .with_env_var("POSTGRES_DB", "postgres");
    let container = image.start().await?;
    let host = container.get_host().await?.to_string();
    let port = container.get_host_port_ipv4(5432).await?;
    let owner_url = format!("postgres://postgres:{TEST_POSTGRES_PASSWORD}@{host}:{port}/postgres");
    let owner_pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&owner_url)
        .await?;

    Ok((container, owner_url, owner_pool))
}

fn app_config(database_url: String) -> anyhow::Result<Config> {
    Ok(Config {
        database_url: SecretString::from(database_url),
        database_owner_url: SecretString::from("postgres://eq:test-secret-owner@localhost/ritma"),
        secret_key: SecretSlice::from(vec![0; 32]),
        secret_key_previous: None,
        public_url: url::Url::parse("https://example.invalid")?,
        mode: Mode::Single,
        listen: SocketAddr::from(([127, 0, 0, 1], 8080)),
        data_dir: PathBuf::from("./data"),
        files_backend: FilesBackend::Local,
        setup_token: None,
        trusted_proxies: Vec::new(),
        log: String::from("info"),
        log_format: LogFormat::Json,
        backup_schedule: BackupSchedule::Off,
        backup_keep: 14,
        smtp_url: None,
        captcha: None,
        tile_url: String::from("https://example.invalid/{z}/{x}/{y}.png"),
        telemetry: false,
        update_check: false,
        metrics_token: None,
        db_pool: 1,
        demo: false,
    })
}

async fn enable_app_role(owner_pool: &PgPool, owner_url: &str) -> anyhow::Result<PgPool> {
    run_migrations(owner_url).await?;
    sqlx::query("ALTER ROLE eq_app LOGIN PASSWORD 'test-secret-app' NOSUPERUSER NOBYPASSRLS")
        .execute(owner_pool)
        .await?;

    let app_url = owner_url.replacen(
        &format!("postgres:{TEST_POSTGRES_PASSWORD}@"),
        &format!("eq_app:{TEST_APP_PASSWORD}@"),
        1,
    );
    connect(&app_config(app_url)?).await
}

#[tokio::test]
async fn bootstrap_idempotent() -> anyhow::Result<()> {
    let (_container, owner_url, _owner_pool) = start_postgres().await?;

    run_migrations(&owner_url).await?;
    run_migrations(&owner_url).await?;
    Ok(())
}

#[tokio::test]
async fn rls_scopes_rows() -> anyhow::Result<()> {
    let (_container, owner_url, owner_pool) = start_postgres().await?;
    let app_pool = enable_app_role(&owner_pool, &owner_url).await?;

    sqlx::query("CREATE TABLE tenant_rls_probe (tenant_id uuid NOT NULL, label text NOT NULL)")
        .execute(&owner_pool)
        .await?;
    sqlx::query("SELECT eq_enable_rls('public.tenant_rls_probe'::regclass)")
        .execute(&owner_pool)
        .await?;
    sqlx::query("GRANT SELECT, INSERT ON tenant_rls_probe TO eq_app")
        .execute(&owner_pool)
        .await?;

    let tenant_a = Uuid::from_u128(1);
    let tenant_b = Uuid::from_u128(2);
    sqlx::query("INSERT INTO tenant_rls_probe (tenant_id, label) VALUES ($1, $2), ($3, $4)")
        .bind(tenant_a)
        .bind("tenant-a")
        .bind(tenant_b)
        .bind("tenant-b")
        .execute(&owner_pool)
        .await?;

    let role = sqlx::query("SELECT rolsuper, rolbypassrls FROM pg_roles WHERE rolname = $1")
        .bind("eq_app")
        .fetch_one(&owner_pool)
        .await?;
    assert!(!role.get::<bool, _>("rolsuper"));
    assert!(!role.get::<bool, _>("rolbypassrls"));

    let actor = ActorCtx {
        person_id: Some(PersonId(Uuid::from_u128(3))),
        role: ActorRole::Owner,
        ip: None::<IpAddr>,
        request_id: None,
    };
    let mut transaction = begin(&app_pool, TenantId(tenant_a), &actor).await?;
    let visible = sqlx::query("SELECT label FROM tenant_rls_probe ORDER BY label")
        .fetch_all(transaction.conn())
        .await?;
    assert_eq!(visible.len(), 1);
    assert_eq!(visible[0].get::<String, _>("label"), "tenant-a");
    transaction.commit().await?;

    let mut connection = app_pool.acquire().await?;
    let visible_without_tenant = sqlx::query("SELECT label FROM tenant_rls_probe ORDER BY label")
        .fetch_all(&mut *connection)
        .await?;
    assert!(visible_without_tenant.is_empty());
    Ok(())
}

#[tokio::test]
async fn settings_do_not_leak() -> anyhow::Result<()> {
    let (_container, owner_url, owner_pool) = start_postgres().await?;
    let app_pool = enable_app_role(&owner_pool, &owner_url).await?;
    let tenant = TenantId(Uuid::from_u128(4));

    let mut transaction = begin_system(&app_pool, tenant).await?;
    let actor_id: Option<String> = sqlx::query_scalar("SELECT current_setting('app.actor_id', true)")
        .fetch_one(transaction.conn())
        .await?;
    assert_eq!(actor_id.as_deref(), Some(""));
    transaction.commit().await?;

    let mut connection = app_pool.acquire().await?;
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
