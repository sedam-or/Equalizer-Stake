use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Request, Response, StatusCode};
use axum::routing::get;
use eq_api::{AppState, middleware::request_id, router};
use eq_core::clock::SystemClock;
use eq_core::config::{BackupSchedule, Config, FilesBackend, LogFormat, Mode};
use eq_core::db::run_migrations;
use eq_core::error::{AppError, codes};
use secrecy::{SecretSlice, SecretString};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use testcontainers::core::{IntoContainerPort, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage, ImageExt};
use tokio::net::TcpListener;
use tower::ServiceExt;
use uuid::Uuid;

const TEST_POSTGRES_PASSWORD: &str = "test-secret-postgres";

fn app_state(database_url: String) -> anyhow::Result<AppState> {
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect_lazy(&database_url)?;
    let config = Config {
        database_url: SecretString::from(database_url),
        database_owner_url: SecretString::from("postgres://eq:test-secret-owner@localhost/ritma"),
        secret_key: SecretSlice::from(vec![0; 32]),
        secret_key_previous: None,
        public_url: url::Url::parse("http://127.0.0.1:8080")?,
        mode: Mode::Single,
        listen: SocketAddr::from(([127, 0, 0, 1], 8080)),
        data_dir: PathBuf::from("./data"),
        files_backend: FilesBackend::Local,
        setup_token: None,
        trusted_proxies: Vec::new(),
        log: String::from("info"),
        log_format: LogFormat::Pretty,
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
    };
    Ok(AppState {
        pool,
        config: Arc::new(config),
        clock: Arc::new(SystemClock),
    })
}

async fn send_get(
    app: Router,
    path: &str,
    client_request_id: Option<&str>,
) -> anyhow::Result<Response<Body>> {
    let mut request = Request::builder().uri(path);
    if let Some(request_id) = client_request_id {
        request = request.header("x-request-id", request_id);
    }
    let response = app
        .oneshot(request.body(Body::empty())?)
        .await
        .map_err(|error| match error {})?;
    Ok(response)
}

#[tokio::test]
async fn healthz_ok() -> anyhow::Result<()> {
    let app = router(app_state(String::from(
        "postgres://eq_app:test-secret-app@127.0.0.1:1/ritma",
    ))?);
    let response = send_get(app, "/healthz", Some("test-secret-forged-request-id")).await?;

    assert_eq!(response.status(), StatusCode::OK);
    let request_id = response
        .headers()
        .get("x-request-id")
        .ok_or_else(|| anyhow::anyhow!("response is missing X-Request-Id"))?
        .to_str()?;
    let request_id = Uuid::parse_str(request_id)?;
    assert_eq!(request_id.get_version_num(), 7);
    assert_ne!(request_id.to_string(), "test-secret-forged-request-id");

    let body: serde_json::Value = serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await?)?;
    assert_eq!(body["status"], "ok");
    assert_eq!(body["version"], env!("CARGO_PKG_VERSION"));
    Ok(())
}

#[tokio::test]
async fn readyz_db_down() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let unused_address = listener.local_addr()?;
    drop(listener);
    let database_url = format!("postgres://eq_app:test-secret-app@{unused_address}/ritma");
    let app = router(app_state(database_url)?);
    let started_at = Instant::now();
    let response = send_get(app, "/readyz", None).await?;

    assert!(started_at.elapsed() < Duration::from_secs(1));
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body: serde_json::Value = serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await?)?;
    assert_eq!(body["status"], "not_ready");
    assert_eq!(body["reason"], "db_unreachable");
    Ok(())
}

#[tokio::test]
async fn readyz_migrations_pending() -> anyhow::Result<()> {
    let (container, owner_url, pool) = start_postgres().await?;
    let app = router(app_state(owner_url)?);
    let response = send_get(app, "/readyz", None).await?;

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body: serde_json::Value = serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await?)?;
    assert_eq!(body["reason"], "migrations_pending");
    pool.close().await;
    drop(container);
    Ok(())
}

#[tokio::test]
async fn readyz_ready() -> anyhow::Result<()> {
    let (container, owner_url, owner_pool) = start_postgres().await?;
    run_migrations(&owner_url).await?;
    sqlx::query("ALTER ROLE eq_app LOGIN PASSWORD 'test-secret-app' NOSUPERUSER NOBYPASSRLS")
        .execute(&owner_pool)
        .await?;
    let app_url = owner_url.replacen(
        &format!("postgres:{TEST_POSTGRES_PASSWORD}@"),
        "eq_app:test-secret-app@",
        1,
    );
    let app = router(app_state(app_url)?);
    let response = send_get(app, "/readyz", None).await?;

    assert_eq!(response.status(), StatusCode::OK);
    let body: serde_json::Value = serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await?)?;
    assert_eq!(body["status"], "ready");
    owner_pool.close().await;
    drop(container);
    Ok(())
}

async fn start_postgres() -> anyhow::Result<(ContainerAsync<GenericImage>, String, sqlx::PgPool)> {
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
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&owner_url)
        .await?;
    Ok((container, owner_url, pool))
}

async fn test_problem() -> AppError {
    AppError::BadRequest {
        code: &codes::BAD_REQUEST,
        detail: String::from("test request failure"),
    }
}

#[tokio::test]
async fn problem_has_request_id() -> anyhow::Result<()> {
    let app = request_id::apply(Router::new().route("/problem", get(test_problem)));
    let response = send_get(app, "/problem", None).await?;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let header_id = response
        .headers()
        .get("x-request-id")
        .ok_or_else(|| anyhow::anyhow!("response is missing X-Request-Id"))?
        .to_str()?
        .to_owned();
    let body: serde_json::Value = serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await?)?;
    assert_eq!(body["request_id"], header_id);
    Ok(())
}
