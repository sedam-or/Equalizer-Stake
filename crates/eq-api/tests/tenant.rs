use anyhow::Result;
use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, Response, StatusCode};
use eq_api::{AppState, router};
use eq_core::clock::SystemClock;
use eq_core::config::{BackupSchedule, Config, FilesBackend, LogFormat, Mode};
use secrecy::{ExposeSecret, SecretSlice, SecretString};
use serde_json::Value;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

#[path = "../../eq-testkit/src/db.rs"]
mod shared_test_db;

use shared_test_db::TestDb;

fn config() -> Result<Config> {
    Ok(Config {
        database_url: SecretString::from("postgres://eq_app:test-secret-app@localhost/ritma"),
        database_owner_url: SecretString::from("postgres://eq:test-secret-owner@localhost/ritma"),
        secret_key: SecretSlice::from(vec![0; 32]),
        secret_key_previous: None,
        public_url: url::Url::parse("https://example.invalid")?,
        mode: Mode::Single,
        base_domain: None,
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
    })
}

async fn test_app() -> Result<(axum::Router, TestDb)> {
    let db = TestDb::new().await;
    let state = AppState::new(db.app_pool.clone(), Arc::new(config()?), Arc::new(SystemClock));
    Ok((router(state), db))
}

async fn send(app: axum::Router, method: Method, path: &str) -> Result<Response<Body>> {
    let request = Request::builder()
        .method(method)
        .uri(path)
        .header("host", "localhost")
        .body(Body::empty())?;
    app.oneshot(request)
        .await
        .map_err(|error| anyhow::anyhow!(error.to_string()))
}

async fn insert_tenant(db: &TestDb, slug: &str, status: &str) -> Result<Uuid> {
    let id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO tenants (id, slug, name, country, preset_id, preset_version, status) \
         VALUES ($1, $2, 'Test School', 'ID', 'test', 1, $3)",
    )
    .bind(id)
    .bind(slug)
    .bind(status)
    .execute(&db.pool)
    .await?;
    Ok(id)
}

async fn problem_code(response: Response<Body>) -> Result<String> {
    let body: Value = serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await?)?;
    Ok(body["code"].as_str().unwrap_or_default().to_owned())
}

#[tokio::test]
async fn suspended_is_read_only() -> Result<()> {
    let (app, db) = test_app().await?;
    assert!(db.url.expose_secret().starts_with("postgres://postgres:"));
    assert!(db.app_url.expose_secret().starts_with("postgres://eq_app:"));
    insert_tenant(&db, "ayu", "suspended").await?;

    let write = send(app.clone(), Method::POST, "/api/v1/people").await?;
    assert_eq!(write.status(), StatusCode::LOCKED);
    assert_eq!(problem_code(write).await?, "locked");

    let read = send(app, Method::GET, "/api/v1/people").await?;
    assert_eq!(read.status(), StatusCode::NOT_FOUND);
    assert_eq!(problem_code(read).await?, "not_found");
    Ok(())
}

#[tokio::test]
async fn unprovisioned_gate() -> Result<()> {
    let (app, _db) = test_app().await?;

    let people = send(app.clone(), Method::GET, "/api/v1/people").await?;
    assert_eq!(people.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(problem_code(people).await?, "setup_required");

    let root = send(app.clone(), Method::GET, "/").await?;
    assert_eq!(root.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(problem_code(root).await?, "setup_required");

    let meta = send(app.clone(), Method::GET, "/api/v1/meta/app").await?;
    assert_eq!(meta.status(), StatusCode::NOT_FOUND);
    assert_eq!(problem_code(meta).await?, "not_found");

    let setup = send(app.clone(), Method::GET, "/api/v1/setup/status").await?;
    assert_eq!(setup.status(), StatusCode::NOT_FOUND);
    assert_eq!(problem_code(setup).await?, "not_found");

    let ready = send(app, Method::GET, "/readyz").await?;
    assert_eq!(ready.status(), StatusCode::OK);
    Ok(())
}
