use anyhow::{Result, anyhow};
use axum::Router;
use axum::body::{Body, to_bytes};
use axum::extract::{ConnectInfo, FromRef, State};
use axum::http::header::HOST;
use axum::http::{Request, StatusCode};
use axum::routing::get;
use eq_core::actor::ActorRole;
use eq_core::config::{BackupSchedule, Config, FilesBackend, LogFormat, Mode};
use eq_core::db::begin;
use eq_core::error::AppError;
use eq_core::ids::TenantId;
use eq_core::tenant::{RequestActor, RequireTenant, TenantCtx, TenantResolver};
use secrecy::{ExposeSecret, SecretSlice, SecretString};
use serde_json::{Value, json};
use sqlx::PgPool;
use sqlx::Row;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

#[path = "../../eq-testkit/src/db.rs"]
mod shared_test_db;

use shared_test_db::TestDb;

fn config(mode: Mode, base_domain: Option<&str>) -> Result<Config> {
    Ok(Config {
        database_url: SecretString::from("postgres://eq_app:test-secret-app@localhost/ritma"),
        database_owner_url: SecretString::from("postgres://eq:test-secret-owner@localhost/ritma"),
        secret_key: SecretSlice::from(vec![0; 32]),
        secret_key_previous: None,
        public_url: url::Url::parse("https://example.invalid")?,
        mode,
        base_domain: base_domain.map(String::from),
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

async fn insert_tenant(db: &TestDb, slug: &str, status: &str) -> Result<TenantId> {
    let tenant_id = TenantId(Uuid::now_v7());
    sqlx::query(
        "INSERT INTO tenants (id, slug, name, country, preset_id, preset_version, status) \
         VALUES ($1, $2, 'Test School', 'ID', 'test', 1, $3)",
    )
    .bind(tenant_id)
    .bind(slug)
    .bind(status)
    .execute(&db.pool)
    .await?;
    Ok(tenant_id)
}

fn parts(host: &str) -> Result<axum::http::request::Parts> {
    let request = Request::builder()
        .uri("/")
        .header(HOST, host)
        .body(Body::empty())?;
    Ok(request.into_parts().0)
}

#[tokio::test]
async fn single_mode_resolves() -> Result<()> {
    let db = TestDb::new().await;
    let tenant_id = insert_tenant(&db, "ayu", "active").await?;
    let resolver = TenantResolver::new(db.app_pool.clone(), Arc::new(config(Mode::Single, None)?));

    let resolved = resolver.resolve(&parts("localhost")?).await?;
    let TenantCtx::Resolved(info) = resolved else {
        return Err(anyhow!("a provisioned single-mode tenant was not resolved"));
    };

    assert_eq!(info.id, tenant_id);
    assert_eq!(info.slug, "ayu");
    assert_eq!(info.timezone, "UTC");
    assert_eq!(info.locale, eq_core::policy::types::Locale::En);
    assert_eq!(info.currency, eq_core::money::Currency::USD);
    Ok(())
}

#[tokio::test]
async fn multi_mode_by_host() -> Result<()> {
    let db = TestDb::new().await;
    let tenant_id = insert_tenant(&db, "ayu", "active").await?;
    let mut multi_config = config(Mode::Multi, Some("app.test"))?;
    multi_config.trusted_proxies = vec!["127.0.0.0/8".parse()?];
    let resolver = TenantResolver::new(db.app_pool.clone(), Arc::new(multi_config));

    let resolved = resolver.resolve(&parts("AYU.App.Test:8080")?).await?;
    let TenantCtx::Resolved(info) = resolved else {
        return Err(anyhow!("multi-mode Host did not resolve a tenant"));
    };
    assert_eq!(info.id, tenant_id);

    let outside = resolver.resolve(&parts("x.other.test")?).await;
    assert!(matches!(outside, Err(AppError::NotFound { entity: "tenant" })));

    let forged = Request::builder()
        .uri("/")
        .header(HOST, "x.other.test")
        .header("x-forwarded-host", "ayu.app.test")
        .body(Body::empty())?;
    let forged = resolver.resolve(&forged.into_parts().0).await;
    assert!(matches!(forged, Err(AppError::NotFound { entity: "tenant" })));

    let mut trusted_forward = Request::builder()
        .uri("/")
        .header(HOST, "x.other.test")
        .header("x-forwarded-host", "ayu.app.test")
        .body(Body::empty())?;
    trusted_forward
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 8080))));
    let trusted = resolver.resolve(&trusted_forward.into_parts().0).await?;
    assert!(matches!(trusted, TenantCtx::Resolved(_)));
    Ok(())
}

#[derive(Clone)]
struct TestState {
    pool: PgPool,
    resolver: TenantResolver,
}

impl FromRef<TestState> for TenantResolver {
    fn from_ref(state: &TestState) -> Self {
        state.resolver.clone()
    }
}

async fn scoped_handler(
    RequireTenant(tenant): RequireTenant,
    RequestActor(actor): RequestActor,
    State(state): State<TestState>,
) -> Result<axum::Json<Value>, AppError> {
    let mut transaction = begin(&state.pool, tenant.id, &actor).await?;
    let row = sqlx::query("SELECT current_setting('app.tenant_id') AS tenant_id")
        .fetch_one(transaction.conn())
        .await?;
    let resolved_tenant_id: String = row.try_get("tenant_id")?;
    transaction.commit().await?;
    Ok(axum::Json(json!({
        "tenant_id": resolved_tenant_id,
        "actor_role": format!("{:?}", actor.role),
        "request_id": actor.request_id.map(|id| id.to_string()),
    })))
}

#[tokio::test]
async fn handler_tx_scoped() -> Result<()> {
    let db = TestDb::new().await;
    let tenant_id = insert_tenant(&db, "ayu", "active").await?;
    let state = TestState {
        pool: db.app_pool.clone(),
        resolver: TenantResolver::new(db.app_pool.clone(), Arc::new(config(Mode::Single, None)?)),
    };
    let app = Router::new().route("/", get(scoped_handler)).with_state(state);
    let request_id = Uuid::now_v7();
    let mut request = Request::builder()
        .uri("/")
        .header(HOST, "localhost")
        .body(Body::empty())?;
    request.extensions_mut().insert(request_id);
    let response = app
        .oneshot(request)
        .await
        .map_err(|error| anyhow!(error.to_string()))?;

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await?)?;
    assert_eq!(body["tenant_id"], tenant_id.0.to_string());
    assert_eq!(body["actor_role"], format!("{:?}", ActorRole::Anonymous));
    assert_eq!(body["request_id"], request_id.to_string());
    Ok(())
}

#[tokio::test]
async fn app_role_can_resolve_metadata_without_table_select() -> Result<()> {
    let db = TestDb::new().await;
    assert!(db.url.expose_secret().starts_with("postgres://postgres:"));
    assert!(db.app_url.expose_secret().starts_with("postgres://eq_app:"));
    let _tenant_id = insert_tenant(&db, "ayu", "active").await?;
    let resolved = sqlx::query(
        "SELECT id, slug, status, timezone, locale, currency FROM public.resolve_tenant_info_by_slug($1)",
    )
    .bind("ayu")
    .fetch_one(&db.app_pool)
    .await?;
    assert_eq!(resolved.try_get::<String, _>("slug")?, "ayu");

    let denied = sqlx::query("SELECT id FROM public.tenants")
        .fetch_all(&db.app_pool)
        .await;
    assert!(denied.is_err(), "eq_app unexpectedly read tenants directly");
    Ok(())
}
