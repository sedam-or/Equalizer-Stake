use std::fmt;
use std::sync::Arc;

use anyhow::{Context, Result, ensure};
use async_trait::async_trait;
use axum::http::StatusCode;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use eq_core::app_state::AppState;
use eq_core::clock::Clock;
use eq_core::config::Config;
use eq_core::crypto::{Keys, bidx_email};
use eq_core::error::AppError;
use eq_core::fields::{Email, FieldCtx, FieldKind, Password, PasswordInput};
use eq_core::ids::{PersonId, TenantId};
use eq_core::model::TenantStatus;
use eq_core::module::gate::{EnabledModuleSet, ModuleGate};
use eq_core::module::{CoreModule, Module, ModuleId, ModuleRegistry};
use eq_core::money::Currency;
use eq_core::policy::PolicyDoc;
use eq_core::policy::types::Locale;
use eq_core::tenant::TenantCtx;
use eq_identity::api_auth::LoginHttpState;
use eq_identity::module::IdentityModule;
use eq_identity::password::hash_password;
use eq_identity::policy::IdentityPolicy;
use reqwest::Client;
use secrecy::ExposeSecret;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};
use uuid::Uuid;

const EMAIL: &str = "reset-person@example.test";
const CURRENT_PASSWORD: &str = "test-secret-current-password";
const NEW_PASSWORD: &str = "test-secret-new-password-strong";

struct Fixture {
    owner_pool: PgPool,
    client: Client,
    address: std::net::SocketAddr,
    server: tokio::task::JoinHandle<Result<(), std::io::Error>>,
    clock: Arc<eq_testkit::clock::ManualClock>,
    tenant: TenantId,
    person: PersonId,
    policy: IdentityPolicy,
}

impl Fixture {
    async fn new() -> Result<Self> {
        let config = Arc::new(Config::from_env().context("load T-110e task database configuration")?);
        ensure!(database_name(config.database_url.expose_secret()) == "ritma_test_T-110e");
        ensure!(database_name(config.database_owner_url.expose_secret()) == "ritma_test_T-110e");
        let app_pool = PgPoolOptions::new()
            .max_connections(6)
            .connect(config.database_url.expose_secret())
            .await
            .context("connect to T-110e application database")?;
        let owner_pool = PgPoolOptions::new()
            .max_connections(2)
            .connect(config.database_owner_url.expose_secret())
            .await
            .context("connect to T-110e migration and seed owner")?;
        verify_app_pool(&app_pool).await?;

        let tenant = TenantId(Uuid::now_v7());
        let person = PersonId(Uuid::now_v7());
        let clock = Arc::new(eq_testkit::clock::test_clock());
        let policy = IdentityPolicy::default();
        seed_identity(&owner_pool, config.as_ref(), tenant, person, &policy, clock.now()).await?;

        let state = AppState::new(app_pool.clone(), Arc::clone(&config), clock.clone());
        let modules = Arc::new(ModuleRegistry::new(vec![
            Arc::new(CoreModule) as Arc<dyn Module>,
            Arc::new(IdentityModule) as Arc<dyn Module>,
        ]));
        let enabled: Arc<dyn EnabledModuleSet> = Arc::new(AlwaysEnabled);
        let runtime = Arc::new(LoginHttpState::new(
            app_pool.clone(),
            clock.clone(),
            Arc::clone(&modules),
            Arc::clone(&enabled),
            Arc::clone(&state.auth),
            state.tenant_resolver.clone(),
        ));
        let (routes, _openapi) = IdentityModule.routes().split_for_parts();
        let routes = routes
            .layer(axum::middleware::from_fn_with_state(
                Arc::clone(&runtime),
                eq_identity::api_auth::actor_auth_middleware,
            ))
            .layer(ModuleGate::new(ModuleId::Identity, enabled))
            .layer(axum::Extension(runtime));
        let tenant_ctx = TenantCtx::Resolved(Arc::new(eq_core::tenant::TenantInfo {
            id: tenant,
            slug: format!("reset-{}", tenant.0.simple()),
            status: TenantStatus::Active,
            timezone: "UTC".to_owned(),
            locale: Locale::En,
            currency: Currency::USD,
        }));

        let router = routes.with_state(state).layer(axum::Extension(tenant_ctx));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let address = listener.local_addr()?;
        let server = tokio::spawn(async move { axum::serve(listener, router).await });

        Ok(Self {
            owner_pool,
            client: Client::new(),
            address,
            server,
            clock,
            tenant,
            person,
            policy,
        })
    }

    async fn post(&self, uri: &str, body: Value) -> Result<(StatusCode, Vec<u8>)> {
        let response = self
            .client
            .post(format!("http://{}{}", self.address, uri))
            .json(&body)
            .send()
            .await
            .context("dispatch password-reset request")?;
        let status = StatusCode::from_u16(response.status().as_u16())?;
        let body = response
            .bytes()
            .await
            .context("read password-reset response")?
            .to_vec();
        Ok((status, body))
    }

    async fn post_problem(&self, uri: &str, body: Value) -> Result<(StatusCode, Value)> {
        let (status, bytes) = self.post(uri, body).await?;
        let problem = serde_json::from_slice(&bytes).context("decode password-reset problem")?;
        Ok((status, problem))
    }

    async fn seed_reset(&self, raw_token: &str, expires_at: chrono::DateTime<chrono::Utc>) -> Result<Uuid> {
        let token_hash = Sha256::digest(raw_token.as_bytes()).to_vec();
        let id = Uuid::now_v7();
        sqlx::query!(
            "INSERT INTO public.password_resets (id, tenant_id, person_id, token_hash, expires_at, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6)",
            id,
            self.tenant.0,
            self.person.0,
            token_hash,
            expires_at,
            self.clock.now()
        )
        .execute(&self.owner_pool)
        .await?;
        Ok(id)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        self.server.abort();
    }
}

struct AlwaysEnabled;

impl fmt::Debug for AlwaysEnabled {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("AlwaysEnabled")
    }
}

#[async_trait]
impl EnabledModuleSet for AlwaysEnabled {
    async fn is_enabled(&self, _tenant: TenantId, _module: ModuleId) -> std::result::Result<bool, AppError> {
        Ok(true)
    }
}

fn database_name(url: &str) -> &str {
    url.rsplit('/').next().unwrap_or_default()
}

async fn verify_app_pool(pool: &PgPool) -> Result<()> {
    let role = sqlx::query(
        "SELECT current_database() AS database, current_user AS username, rolsuper, rolbypassrls \
         FROM pg_roles WHERE rolname = current_user",
    )
    .fetch_one(pool)
    .await?;
    ensure!(role.try_get::<String, _>("database")? == "ritma_test_T-110e");
    ensure!(role.try_get::<String, _>("username")? == "eq_app");
    ensure!(!role.try_get::<bool, _>("rolsuper")? && !role.try_get::<bool, _>("rolbypassrls")?);
    Ok(())
}

async fn seed_identity(
    owner_pool: &PgPool,
    config: &Config,
    tenant: TenantId,
    person: PersonId,
    policy: &IdentityPolicy,
    now: chrono::DateTime<chrono::Utc>,
) -> Result<()> {
    sqlx::query!(
        "INSERT INTO public.tenants (id, slug, name, country, preset_id, preset_version) \
         VALUES ($1, $2, 'Test School', 'ID', 'test', 1)",
        tenant.0,
        format!("reset-{}", tenant.0.simple())
    )
    .execute(owner_pool)
    .await?;
    let field_ctx = field_ctx(now)?;
    let email = Email::parse(EMAIL.to_owned(), &field_ctx)
        .map_err(|_| anyhow::anyhow!("fake test email is valid"))?;
    let keys = Keys::derive(config.secret_key.expose_secret(), None);
    let email_bidx = bidx_email(&keys, tenant, email.as_ref())?;
    sqlx::query!(
        "INSERT INTO public.persons (id, tenant_id, display_name, email_bidx) VALUES ($1, $2, 'Test Person', $3)",
        person.0,
        tenant.0,
        email_bidx.as_slice()
    )
    .execute(owner_pool)
    .await?;
    sqlx::query!(
        "INSERT INTO public.memberships (tenant_id, person_id, role, status) \
         VALUES ($1, $2, 'admin', 'active')",
        tenant.0,
        person.0
    )
    .execute(owner_pool)
    .await?;
    let password_input = serde_json::from_value::<PasswordInput>(json!(CURRENT_PASSWORD))?;
    let password = Password::parse(password_input, &field_ctx)
        .map_err(|_| anyhow::anyhow!("fake current test password is valid"))?;
    let password_hash = hash_password(&password)?;
    sqlx::query!(
        "INSERT INTO public.credentials (tenant_id, person_id, password_hash, password_changed_at, failed_count) \
         VALUES ($1, $2, $3, $4, 0)",
        tenant.0,
        person.0,
        password_hash.expose_secret(),
        now
    )
    .execute(owner_pool)
    .await?;
    sqlx::query!(
        "INSERT INTO public.policies (tenant_id, module_id, version, schema_version, effective_from, document) \
         VALUES ($1, 'identity', 1, $2, $3, $4)",
        tenant.0,
        i32::try_from(IdentityPolicy::SCHEMA_VERSION)?,
        now - chrono::Duration::days(1),
        serde_json::to_value(policy)?
    )
    .execute(owner_pool)
    .await?;
    Ok(())
}

fn field_ctx(now: chrono::DateTime<chrono::Utc>) -> Result<FieldCtx> {
    Ok(FieldCtx {
        country: None,
        tenant_currency: Currency::USD,
        tenant_locale: Locale::En,
        today: now.date_naive(),
        password_min_length: None,
    })
}

#[tokio::test]
async fn forgot_no_enumeration() -> Result<()> {
    let fixture = Fixture::new().await?;
    let known = fixture
        .post("/api/v1/auth/forgot", json!({"email": EMAIL}))
        .await?;
    let unknown = fixture
        .post("/api/v1/auth/forgot", json!({"email": "unknown@example.test"}))
        .await?;
    assert_eq!(known.0, StatusCode::ACCEPTED);
    assert_eq!(unknown.0, StatusCode::ACCEPTED);
    assert_eq!(known.1, unknown.1);
    assert!(known.1.is_empty());

    let reset_count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM public.password_resets WHERE tenant_id = $1 AND person_id = $2",
        fixture.tenant.0,
        fixture.person.0
    )
    .fetch_one(&fixture.owner_pool)
    .await?
    .context("password reset count is non-null")?;
    assert_eq!(reset_count, 1);
    let events = sqlx::query!(
        "SELECT event_type, payload FROM public.outbox WHERE tenant_id = $1 AND aggregate_id = $2",
        fixture.tenant.0,
        fixture.person.0
    )
    .fetch_all(&fixture.owner_pool)
    .await?;
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, "identity.password_reset_requested.v1");
    let payload = events[0].payload["payload"]
        .as_object()
        .context("event has typed payload")?;
    assert_eq!(payload.len(), 2);
    assert!(payload.contains_key("person_id"));
    assert!(payload.contains_key("token_id"));
    assert!(!events[0].payload.to_string().contains("test-secret"));
    Ok(())
}

#[tokio::test]
async fn forgot_rate_limited_silently() -> Result<()> {
    let fixture = Fixture::new().await?;
    let now = fixture.clock.now();
    let cap = fixture.policy.reset_max_per_hour();
    for index in 0..cap {
        let id = Uuid::now_v7();
        let token_hash =
            Sha256::digest(format!("test-secret-rate-limit-{}-{index}", fixture.tenant.0).as_bytes())
                .to_vec();
        sqlx::query!(
            "INSERT INTO public.password_resets (id, tenant_id, person_id, token_hash, expires_at, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6)",
            id,
            fixture.tenant.0,
            fixture.person.0,
            token_hash,
            now + chrono::Duration::hours(1),
            now - chrono::Duration::minutes(5)
        )
        .execute(&fixture.owner_pool)
        .await?;
    }
    let (status, body) = fixture
        .post("/api/v1/auth/forgot", json!({"email": EMAIL}))
        .await?;
    assert_eq!(status, StatusCode::ACCEPTED);
    assert!(body.is_empty());
    let count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM public.password_resets WHERE tenant_id = $1 AND person_id = $2",
        fixture.tenant.0,
        fixture.person.0
    )
    .fetch_one(&fixture.owner_pool)
    .await?
    .context("password reset count is non-null")?;
    assert_eq!(count, i64::from(cap));
    let events = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM public.outbox WHERE tenant_id = $1 AND aggregate_id = $2",
        fixture.tenant.0,
        fixture.person.0
    )
    .fetch_one(&fixture.owner_pool)
    .await?
    .context("outbox count is non-null")?;
    assert_eq!(events, 0);
    Ok(())
}

#[tokio::test]
async fn reset_revokes_sessions() -> Result<()> {
    let fixture = Fixture::new().await?;
    let raw_token = URL_SAFE_NO_PAD.encode(Sha256::digest(
        format!("test-secret-reset-{}", fixture.tenant.0).as_bytes(),
    ));
    let reset_id = fixture
        .seed_reset(&raw_token, fixture.clock.now() + chrono::Duration::minutes(30))
        .await?;
    let before_password_changed_at = sqlx::query_scalar!(
        "SELECT password_changed_at AS \"password_changed_at!\" FROM public.credentials \
         WHERE tenant_id = $1 AND person_id = $2",
        fixture.tenant.0,
        fixture.person.0
    )
    .fetch_one(&fixture.owner_pool)
    .await?;
    let (weak_status, weak_problem) = fixture
        .post_problem(
            "/api/v1/auth/reset",
            json!({"token": raw_token.as_str(), "new_password": "short"}),
        )
        .await?;
    assert_eq!(weak_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(weak_problem["code"], "password_weak");
    let (malformed_status, malformed_problem) = fixture
        .post_problem(
            "/api/v1/auth/reset",
            json!({"token": "test-secret-malformed-token", "new_password": NEW_PASSWORD}),
        )
        .await?;
    assert_eq!(malformed_status, StatusCode::BAD_REQUEST);
    assert_eq!(malformed_problem["code"], "token_invalid");
    seed_refresh_sessions(&fixture).await?;

    let (status, body) = fixture
        .post(
            "/api/v1/auth/reset",
            json!({"token": raw_token.as_str(), "new_password": NEW_PASSWORD}),
        )
        .await?;
    assert_eq!(
        status,
        StatusCode::NO_CONTENT,
        "{}",
        String::from_utf8_lossy(&body)
    );
    assert!(body.is_empty());
    assert_reset_effects(&fixture, reset_id, before_password_changed_at).await?;

    let (replay_status, replay_problem) = fixture
        .post_problem(
            "/api/v1/auth/reset",
            json!({"token": raw_token.as_str(), "new_password": "test-secret-another-password"}),
        )
        .await?;
    assert_eq!(replay_status, StatusCode::BAD_REQUEST);
    assert_eq!(replay_problem["code"], "token_invalid");
    Ok(())
}

async fn seed_refresh_sessions(fixture: &Fixture) -> Result<()> {
    for index in 0..2 {
        sqlx::query!(
            "INSERT INTO public.refresh_sessions \
             (id, tenant_id, person_id, family_id, token_hash, active_role, idle_expires_at, absolute_expires_at) \
             VALUES ($1, $2, $3, $4, $5, 'admin', $6, $7)",
            Uuid::now_v7(),
            fixture.tenant.0,
            fixture.person.0,
            Uuid::now_v7(),
            Sha256::digest(
                format!("test-secret-session-{}-{index}", fixture.tenant.0).as_bytes(),
            )
            .to_vec(),
            fixture.clock.now() + chrono::Duration::hours(1),
            fixture.clock.now() + chrono::Duration::days(1)
        )
        .execute(&fixture.owner_pool)
        .await?;
    }
    Ok(())
}

async fn assert_reset_effects(
    fixture: &Fixture,
    reset_id: Uuid,
    before_password_changed_at: chrono::DateTime<chrono::Utc>,
) -> Result<()> {
    let reset = sqlx::query!(
        "SELECT used_at FROM public.password_resets WHERE tenant_id = $1 AND id = $2",
        fixture.tenant.0,
        reset_id
    )
    .fetch_one(&fixture.owner_pool)
    .await?;
    assert!(reset.used_at.is_some());
    let credentials = sqlx::query!(
        "SELECT password_hash, password_changed_at FROM public.credentials WHERE tenant_id = $1 AND person_id = $2",
        fixture.tenant.0,
        fixture.person.0
    )
    .fetch_one(&fixture.owner_pool)
    .await?;
    assert!(credentials.password_hash.is_some());
    let password_changed_at = credentials
        .password_changed_at
        .context("reset stores the new password-version timestamp")?;
    assert!(password_changed_at.timestamp() > before_password_changed_at.timestamp());
    let sessions = sqlx::query!(
        "SELECT revoked_at FROM public.refresh_sessions WHERE tenant_id = $1 AND person_id = $2",
        fixture.tenant.0,
        fixture.person.0
    )
    .fetch_all(&fixture.owner_pool)
    .await?;
    assert_eq!(sessions.len(), 2);
    assert!(sessions.iter().all(|session| session.revoked_at.is_some()));
    let audit = sqlx::query!(
        "SELECT actor_id, actor_role, action, entity_type, entity_id FROM audit.entries \
         WHERE tenant_id = $1 AND entity_id = $2 AND action = 'security.password.reset'",
        fixture.tenant.0,
        fixture.person.0
    )
    .fetch_one(&fixture.owner_pool)
    .await?;
    assert!(audit.actor_id.is_none());
    assert_eq!(audit.actor_role, "system");
    assert_eq!(audit.entity_type, "credential");
    assert_eq!(audit.entity_id, Some(fixture.person.0));
    Ok(())
}
