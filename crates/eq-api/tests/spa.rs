use axum::body::{Body, to_bytes};
use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE, ETAG, IF_NONE_MATCH};
use axum::http::{Request, Response, StatusCode};
use eq_api::{AppState, router};
use eq_core::clock::SystemClock;
use eq_core::config::{BackupSchedule, Config, FilesBackend, LogFormat, Mode};
use secrecy::{ExposeSecret, SecretSlice, SecretString};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

#[path = "../../eq-testkit/src/db.rs"]
mod shared_test_db;

use shared_test_db::TestDb;

async fn app() -> anyhow::Result<(axum::Router, TestDb)> {
    let db = TestDb::new().await;
    let slug = format!("spa{}", Uuid::now_v7().simple());
    sqlx::query(
        "INSERT INTO tenants (id, slug, name, country, preset_id, preset_version) \
         VALUES ($1, $2, 'Test School', 'ID', 'test', 1)",
    )
    .bind(Uuid::now_v7())
    .bind(slug)
    .execute(&db.pool)
    .await?;
    let database_url = db.app_url.expose_secret();
    let config = Config {
        database_url: SecretString::from(database_url),
        database_owner_url: SecretString::from(db.url.expose_secret()),
        secret_key: SecretSlice::from(vec![0; 32]),
        secret_key_previous: None,
        public_url: url::Url::parse("http://127.0.0.1:8080")?,
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
    };

    Ok((
        router(AppState::new(
            db.app_pool.clone(),
            Arc::new(config),
            Arc::new(SystemClock),
        )),
        db,
    ))
}

async fn send(app: axum::Router, path: &str, etag: Option<&str>) -> anyhow::Result<Response<Body>> {
    let mut request = Request::builder().uri(path);
    if let Some(etag) = etag {
        request = request.header(IF_NONE_MATCH, etag);
    }
    let response = app
        .oneshot(request.body(Body::empty())?)
        .await
        .map_err(|error| match error {})?;
    Ok(response)
}

#[tokio::test]
async fn index_no_cache() -> anyhow::Result<()> {
    let (app, _db) = app().await?;
    let response = send(app, "/", None).await?;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get(CACHE_CONTROL)
            .and_then(|value| value.to_str().ok()),
        Some("no-cache")
    );
    assert_eq!(
        response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()),
        Some("text/html")
    );
    let body = to_bytes(response.into_body(), usize::MAX).await?;
    assert!(std::str::from_utf8(&body)?.contains("id=\"root\""));
    Ok(())
}

#[tokio::test]
async fn asset_immutable() -> anyhow::Result<()> {
    let (app, _db) = app().await?;
    let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../web/dist/assets");
    let asset = std::fs::read_dir(assets_dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .find(|file_name| file_name.ends_with(".js"))
        .ok_or_else(|| anyhow::anyhow!("no generated JavaScript asset exists in web/dist/assets"))?;
    let response = send(app, &format!("/assets/{asset}"), None).await?;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get(CACHE_CONTROL)
            .and_then(|value| value.to_str().ok()),
        Some("public, max-age=31536000, immutable")
    );
    assert!(response.headers().get(ETAG).is_none());
    Ok(())
}

#[tokio::test]
async fn spa_fallback() -> anyhow::Result<()> {
    let (app, _db) = app().await?;
    let response = send(app, "/schedule/week", None).await?;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get(CACHE_CONTROL)
            .and_then(|value| value.to_str().ok()),
        Some("no-cache")
    );
    let body = to_bytes(response.into_body(), usize::MAX).await?;
    assert!(std::str::from_utf8(&body)?.contains("id=\"root\""));
    Ok(())
}

#[tokio::test]
async fn api_404_not_spa() -> anyhow::Result<()> {
    let (app, _db) = app().await?;
    let response = send(app, "/api/v1/nope", None).await?;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(
        response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()),
        Some("application/problem+json")
    );
    let body: serde_json::Value = serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await?)?;
    assert_eq!(body["code"], "not_found");
    Ok(())
}

#[tokio::test]
async fn etag_304() -> anyhow::Result<()> {
    let (app, _db) = app().await?;
    let first = send(app.clone(), "/", None).await?;
    assert_eq!(first.status(), StatusCode::OK);
    let etag = first
        .headers()
        .get(ETAG)
        .ok_or_else(|| anyhow::anyhow!("index response is missing ETag"))?
        .to_str()?
        .to_owned();

    let second = send(app, "/", Some(&etag)).await?;
    assert_eq!(second.status(), StatusCode::NOT_MODIFIED);
    assert_eq!(
        second.headers().get(ETAG).and_then(|value| value.to_str().ok()),
        Some(etag.as_str())
    );
    assert_eq!(
        second
            .headers()
            .get(CACHE_CONTROL)
            .and_then(|value| value.to_str().ok()),
        Some("no-cache")
    );
    assert!(to_bytes(second.into_body(), usize::MAX).await?.is_empty());
    Ok(())
}
