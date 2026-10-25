//! TST-03/TST-20: isolated PostgreSQL databases and an RLS-safe app pool.

use anyhow::{Context, Result, anyhow};
use eq_core::db;
use secrecy::{ExposeSecret, SecretString};
use sqlx::{AssertSqlSafe, PgPool, postgres::PgPoolOptions};
use std::fmt;
use testcontainers::core::{IntoContainerPort, WaitFor};
use testcontainers::{ContainerAsync, GenericImage, ImageExt, runners::AsyncRunner};
use tokio::sync::OnceCell;
use url::Url;
use uuid::Uuid;

const TEMPLATE_DATABASE: &str = "eq_template";
const TEST_POSTGRES_PASSWORD: &str = concat!("test-secret-", "postgres");
const TEST_APP_PASSWORD: &str = concat!("test-secret-", "app");
static TEMPLATE: OnceCell<Result<TestTemplate>> = OnceCell::const_new();

/// TST-20: isolated database connections; the owner pool is for setup only.
pub struct TestDb {
    /// Superuser pool reserved for setup, seeding and DDL.
    pub pool: PgPool,
    /// Credential-bearing owner URL; code under test must use `app_pool`.
    pub url: SecretString,
    /// Least-privileged `eq_app` pool for routers and RLS/privilege assertions.
    pub app_pool: PgPool,
    /// Credential-bearing app URL, retained for downstream test helpers.
    pub app_url: SecretString,
}

impl fmt::Debug for TestDb {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TestDb")
            .field("pool", &"[SUPERUSER TEST POOL]")
            .field("url", &"[REDACTED]")
            .field("app_pool", &"[EQ_APP TEST POOL]")
            .field("app_url", &"[REDACTED]")
            .finish()
    }
}

struct TestTemplate {
    _container: Option<ContainerAsync<GenericImage>>,
    owner_url: SecretString,
}

impl fmt::Debug for TestTemplate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TestTemplate")
            .field("container_running", &self._container.is_some())
            .field("owner_url", &"[REDACTED]")
            .finish()
    }
}

impl TestDb {
    /// TST-20: creates a database cloned from the migrated per-process template.
    #[expect(
        clippy::panic,
        reason = "TestDb::new has a fixed task signature and must fail test setup without exposing credential-bearing driver errors"
    )]
    pub async fn new() -> Self {
        let template = test_template().await;
        match create_database(template).await {
            Ok(database) => database,
            Err(_) => panic!("test database creation failed; database details redacted"),
        }
    }
}

#[expect(
    clippy::panic,
    reason = "the OnceCell initializer is test infrastructure; its fixed interface must fail setup while redacting database errors"
)]
async fn test_template() -> &'static TestTemplate {
    match TEMPLATE.get_or_init(initialize_template).await {
        Ok(template) => template,
        Err(_) => panic!("test database template initialization failed; database details redacted"),
    }
}

async fn initialize_template() -> Result<TestTemplate> {
    let (container, owner_url) = match std::env::var_os("RITMA_TEST_DATABASE_URL") {
        Some(value) => {
            let value = value
                .into_string()
                .map_err(|_| anyhow!("RITMA_TEST_DATABASE_URL must be UTF-8"))?;
            (None, SecretString::from(value))
        }
        None => start_postgres().await?,
    };

    let owner_pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(owner_url.expose_secret())
        .await
        .context("connect to the test PostgreSQL owner database")?;

    let template_exists =
        sqlx::query_scalar::<_, bool>("SELECT EXISTS (SELECT 1 FROM pg_database WHERE datname = $1)")
            .bind(TEMPLATE_DATABASE)
            .fetch_one(&owner_pool)
            .await
            .context("check for the test template database")?;
    if !template_exists {
        sqlx::raw_sql("CREATE DATABASE eq_template TEMPLATE template0")
            .execute(&owner_pool)
            .await
            .context("create the test template database")?;
    }

    let template_url = database_url(&owner_url, TEMPLATE_DATABASE, None)?;
    db::run_migrations(template_url.expose_secret())
        .await
        .context("migrate the test template database")?;

    // This statement contains only a compile-time fake password and fixed role name.
    sqlx::query(AssertSqlSafe(format!(
        "ALTER ROLE eq_app LOGIN PASSWORD '{TEST_APP_PASSWORD}' NOSUPERUSER NOBYPASSRLS"
    )))
    .execute(&owner_pool)
    .await
    .context("configure the least-privileged test app role")?;

    sqlx::raw_sql("ALTER DATABASE eq_template WITH IS_TEMPLATE true")
        .execute(&owner_pool)
        .await
        .context("mark the migrated test database as a template")?;
    owner_pool.close().await;

    Ok(TestTemplate {
        _container: container,
        owner_url,
    })
}

async fn start_postgres() -> Result<(Option<ContainerAsync<GenericImage>>, SecretString)> {
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
    let container = image
        .start()
        .await
        .context("start the PostgreSQL 16 test container")?;
    let host = container
        .get_host()
        .await
        .context("get the PostgreSQL test container host")?
        .to_string();
    let port = container
        .get_host_port_ipv4(5432)
        .await
        .context("get the PostgreSQL test container port")?;

    let mut owner_url =
        Url::parse("postgres://postgres@localhost/postgres").context("build the test container owner URL")?;
    owner_url
        .set_host(Some(&host))
        .map_err(|error| anyhow!("invalid test container host: {error}"))?;
    owner_url
        .set_port(Some(port))
        .map_err(|()| anyhow!("invalid test container port"))?;
    owner_url
        .set_password(Some(TEST_POSTGRES_PASSWORD))
        .map_err(|()| anyhow!("invalid test container password"))?;

    Ok((Some(container), SecretString::from(owner_url.to_string())))
}

async fn create_database(template: &TestTemplate) -> Result<TestDb> {
    let database_name = format!("ritma_test_{}", Uuid::now_v7().simple());
    // The identifier is generated from a fixed prefix and UUID hex; SQLx cannot bind identifiers.
    let create_database = AssertSqlSafe(format!(
        "CREATE DATABASE \"{database_name}\" TEMPLATE eq_template"
    ));
    let owner_pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(template.owner_url.expose_secret())
        .await
        .context("connect to the test database owner")?;
    sqlx::raw_sql(create_database)
        .execute(&owner_pool)
        .await
        .context("clone the migrated test template")?;
    owner_pool.close().await;

    let url = database_url(&template.owner_url, &database_name, None)?;
    let app_url = database_url(
        &template.owner_url,
        &database_name,
        Some(("eq_app", TEST_APP_PASSWORD)),
    )?;

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(url.expose_secret())
        .await
        .context("connect to the test database as owner")?;
    let app_pool = PgPoolOptions::new()
        .after_release(|connection, _metadata| {
            Box::pin(async move {
                sqlx::query("RESET ALL")
                    .execute(connection)
                    .await
                    .map(|_| true)
                    .or(Ok(false))
            })
        })
        .connect(app_url.expose_secret())
        .await
        .context("connect to the test database as eq_app")?;

    Ok(TestDb {
        pool,
        url,
        app_pool,
        app_url,
    })
}

fn database_url(
    base_url: &SecretString,
    database: &str,
    credentials: Option<(&str, &str)>,
) -> Result<SecretString> {
    let mut url =
        Url::parse(base_url.expose_secret()).context("parse a credential-bearing test database URL")?;
    url.set_path(&format!("/{database}"));
    if let Some((username, password)) = credentials {
        url.set_username(username)
            .map_err(|()| anyhow!("invalid test database username"))?;
        url.set_password(Some(password))
            .map_err(|()| anyhow!("invalid test database password"))?;
    }
    Ok(SecretString::from(url.to_string()))
}
