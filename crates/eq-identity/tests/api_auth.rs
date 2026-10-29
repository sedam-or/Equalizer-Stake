use std::fmt;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, ensure};
use async_trait::async_trait;
use axum::body::{Body, to_bytes};
use axum::http::header::{CONTENT_TYPE, SET_COOKIE};
use axum::http::{Request, StatusCode};
use eq_core::app_state::AppState;
use eq_core::clock::SystemClock;
use eq_core::config::Config;
use eq_core::crypto::{Keys, bidx_email};
use eq_core::error::AppError;
use eq_core::fields::{Email, FieldCtx, FieldKind, Password, PasswordInput};
use eq_core::ids::{PersonId, TenantId};
use eq_core::model::{Role, TenantStatus};
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
use secrecy::ExposeSecret;
use serde_json::{Value, json};
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};
use tower::ServiceExt;
use uuid::Uuid;

const EMAIL: &str = "teacher@example.test";
const CORRECT_PASSWORD: &str = "test-secret-correct-password";
const WRONG_PASSWORD: &str = "test-secret-incorrect-password";

fn task_database(config: &Config) -> &str {
    config
        .database_url
        .expose_secret()
        .rsplit('/')
        .next()
        .unwrap_or_default()
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

struct Fixture {
    owner_pool: PgPool,
    router: axum::Router,
    tenant: TenantId,
    person: PersonId,
    policy: IdentityPolicy,
}

impl Fixture {
    async fn post(&self, email: &str, password: &str) -> Result<(StatusCode, axum::http::HeaderMap, Value)> {
        self.post_with_token(email, password, None).await
    }

    async fn post_with_token(
        &self,
        email: &str,
        password: &str,
        token: Option<&str>,
    ) -> Result<(StatusCode, axum::http::HeaderMap, Value)> {
        let mut builder = Request::builder()
            .method("POST")
            .uri("/api/v1/auth/login")
            .header(CONTENT_TYPE, "application/json");
        if let Some(token) = token {
            builder = builder.header(axum::http::header::AUTHORIZATION, format!("Bearer {token}"));
        }
        let request = builder.body(Body::from(
            json!({"email": email, "password": password}).to_string(),
        ))?;
        let response = self
            .router
            .clone()
            .oneshot(request)
            .await
            .context("dispatch login request")?;
        let status = response.status();
        let headers = response.headers().clone();
        let body = to_bytes(response.into_body(), 1024 * 1024)
            .await
            .context("read login response")?;
        let body = serde_json::from_slice(&body).context("decode login response JSON")?;
        Ok((status, headers, body))
    }

    async fn get_protected_actor(&self, token: &str) -> Result<(StatusCode, Value)> {
        self.get_protected_actor_with_authorization(&format!("Bearer {token}"))
            .await
    }

    async fn get_protected_actor_with_authorization(
        &self,
        authorization: &str,
    ) -> Result<(StatusCode, Value)> {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let address = listener.local_addr()?;
        let router = self.router.clone();
        let server = tokio::spawn(async move {
            axum::serve(
                listener,
                router.into_make_service_with_connect_info::<SocketAddr>(),
            )
            .await
        });
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()?;
        let response = client
            .get(format!("http://{address}/__test/actor"))
            .header(axum::http::header::AUTHORIZATION, authorization)
            .send()
            .await;
        server.abort();
        let _ = server.await;
        let response = response?;
        let status = response.status();
        let body = response.bytes().await.context("read protected Actor response")?;
        let body = serde_json::from_slice(&body).context("decode protected Actor response JSON")?;
        Ok((status, body))
    }
}

async fn protected_actor(actor: eq_core::auth::Actor) -> axum::Json<Value> {
    axum::Json(json!({"has_write": actor.has("identity.people.write")}))
}

async fn fixture() -> Result<Fixture> {
    fixture_for_role(Role::Admin, false).await
}

async fn fixture_for_role(role: Role, seed_pii_ciphertext: bool) -> Result<Fixture> {
    fixture_for_role_and_admin(role, seed_pii_ciphertext, false).await
}

async fn fixture_for_role_and_admin(
    role: Role,
    seed_pii_ciphertext: bool,
    admin_present: bool,
) -> Result<Fixture> {
    let config = Arc::new(Config::from_env().context("load T-110a4 test configuration")?);
    let task_database = task_database(config.as_ref());
    ensure!(
        task_database.starts_with("ritma_test_"),
        "identity tests require the disposable task database"
    );
    ensure!(
        config.database_owner_url.expose_secret().rsplit('/').next() == Some(task_database),
        "identity test owner connection must target the disposable task database"
    );
    let (app_pool, owner_pool) = connect_task_database(config.as_ref()).await?;
    let clock = Arc::new(SystemClock);
    let state = AppState::new(app_pool.clone(), Arc::clone(&config), clock.clone());
    let (tenant, person, policy) = seed_login_fixture(
        &owner_pool,
        config.as_ref(),
        role,
        seed_pii_ciphertext,
        admin_present,
    )
    .await?;

    let modules = Arc::new(ModuleRegistry::new(vec![
        Arc::new(CoreModule) as Arc<dyn Module>,
        Arc::new(IdentityModule) as Arc<dyn Module>,
    ]));
    let enabled: Arc<dyn EnabledModuleSet> = Arc::new(AlwaysEnabled);
    let runtime = Arc::new(LoginHttpState::new(
        app_pool.clone(),
        clock,
        Arc::clone(&modules),
        Arc::clone(&enabled),
        Arc::clone(&state.auth),
        state.tenant_resolver.clone(),
    ));
    let (routes, _openapi) = IdentityModule.routes().split_for_parts();
    let routes = routes
        .route("/__test/actor", axum::routing::get(protected_actor))
        .layer(axum::middleware::from_fn_with_state(
            Arc::clone(&runtime),
            eq_identity::api_auth::actor_auth_middleware,
        ))
        .layer(ModuleGate::new(ModuleId::Identity, enabled))
        .layer(axum::Extension(runtime));
    let tenant_ctx = TenantCtx::Resolved(Arc::new(eq_core::tenant::TenantInfo {
        id: tenant,
        slug: format!("auth-{}", tenant.0.simple()),
        status: TenantStatus::Active,
        timezone: "UTC".to_owned(),
        locale: Locale::En,
        currency: Currency::USD,
    }));
    let router = routes.with_state(state).layer(axum::Extension(tenant_ctx));

    Ok(Fixture {
        owner_pool,
        router,
        tenant,
        person,
        policy,
    })
}

async fn connect_task_database(config: &Config) -> Result<(PgPool, PgPool)> {
    let app_pool = PgPoolOptions::new()
        .max_connections(8)
        .connect(config.database_url.expose_secret())
        .await
        .map_err(|_| anyhow::anyhow!("connect to identity application test database"))?;
    let owner_pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(config.database_owner_url.expose_secret())
        .await
        .map_err(|_| anyhow::anyhow!("connect to identity test database owner"))?;
    let role = sqlx::query(
        "SELECT current_database() AS database, current_user AS username, rolsuper, rolbypassrls \
         FROM pg_roles WHERE rolname = current_user",
    )
    .fetch_one(&app_pool)
    .await?;
    ensure!(role.try_get::<String, _>("database")? == task_database(config));
    ensure!(role.try_get::<String, _>("username")? == "eq_app");
    ensure!(!role.try_get::<bool, _>("rolsuper")? && !role.try_get::<bool, _>("rolbypassrls")?);
    Ok((app_pool, owner_pool))
}

async fn seed_login_fixture(
    owner_pool: &PgPool,
    config: &Config,
    role: Role,
    seed_pii_ciphertext: bool,
    admin_present: bool,
) -> Result<(TenantId, PersonId, IdentityPolicy)> {
    let tenant = TenantId(Uuid::now_v7());
    let person = PersonId(Uuid::now_v7());
    let slug = format!("auth-{}", tenant.0.simple());
    sqlx::query(
        "INSERT INTO public.tenants (id, slug, name, country, preset_id, preset_version) \
         VALUES ($1, $2, 'Test School', 'ID', 'test', 1)",
    )
    .bind(tenant.0)
    .bind(slug)
    .execute(owner_pool)
    .await?;
    let field_ctx = field_ctx()?;
    let email = Email::parse(EMAIL.to_owned(), &field_ctx)
        .map_err(|_| anyhow::anyhow!("fixed test email is valid"))?;
    let keys = Keys::derive(config.secret_key.expose_secret(), None);
    let email_bidx = bidx_email(&keys, tenant, email.as_ref())?;
    let pii_sentinels = seed_pii_ciphertext.then(|| vec![0_u8]);
    sqlx::query(
        "INSERT INTO public.persons (id, tenant_id, display_name, email_enc, phone_enc, email_bidx) \
         VALUES ($1, $2, 'Test Person', $3, $4, $5)",
    )
    .bind(person.0)
    .bind(tenant.0)
    .bind(pii_sentinels.clone())
    .bind(pii_sentinels)
    .bind(email_bidx.as_slice())
    .execute(owner_pool)
    .await?;
    if admin_present && role != Role::Admin {
        sqlx::query(
            "INSERT INTO public.memberships (tenant_id, person_id, role, status) VALUES ($1, $2, 'admin', 'active')",
        )
        .bind(tenant.0)
        .bind(person.0)
        .execute(owner_pool)
        .await?;
    }
    sqlx::query(
        "INSERT INTO public.memberships (tenant_id, person_id, role, status) VALUES ($1, $2, $3, 'active')",
    )
    .bind(tenant.0)
    .bind(person.0)
    .bind(role)
    .execute(owner_pool)
    .await?;

    let password_input = serde_json::from_value::<PasswordInput>(json!(CORRECT_PASSWORD))?;
    let password = Password::parse(password_input, &field_ctx)
        .map_err(|_| anyhow::anyhow!("fixed test password is valid"))?;
    let password_hash = hash_password(&password)?;
    sqlx::query(
        "INSERT INTO public.credentials (tenant_id, person_id, password_hash, failed_count) VALUES ($1, $2, $3, 0)",
    )
    .bind(tenant.0)
    .bind(person.0)
    .bind(password_hash.expose_secret())
    .execute(owner_pool)
    .await?;

    let policy = IdentityPolicy::default();
    let schema_version = i32::try_from(IdentityPolicy::SCHEMA_VERSION)?;
    sqlx::query(
        "INSERT INTO public.policies (tenant_id, module_id, version, schema_version, effective_from, document) \
         VALUES ($1, 'identity', 1, $2, now() - interval '1 day', $3)",
    )
    .bind(tenant.0)
    .bind(schema_version)
    .bind(serde_json::to_value(&policy)?)
    .execute(owner_pool)
    .await?;
    Ok((tenant, person, policy))
}

fn field_ctx() -> Result<FieldCtx> {
    Ok(FieldCtx {
        country: None,
        tenant_currency: Currency::USD,
        tenant_locale: Locale::En,
        today: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).context("fixed test date is valid")?,
        password_min_length: None,
    })
}

#[tokio::test]
async fn login_success() -> Result<()> {
    let fixture = fixture().await?;
    let (status, headers, body) = fixture.post(EMAIL, CORRECT_PASSWORD).await?;

    assert_eq!(status, StatusCode::OK);
    assert!(body["access_token"].as_str().is_some());
    assert_eq!(body["expires_in"], 600);
    assert_eq!(body["person"]["id"], fixture.person.0.to_string());
    assert_eq!(body["person"]["display_name"], "Test Person");
    assert_eq!(body["person"]["roles"], json!(["admin"]));
    assert_eq!(body["person"]["active_role"], "admin");
    assert_eq!(body["roles"], json!(["admin"]));
    assert_eq!(body["active_role"], "admin");
    assert_eq!(body["person"]["email"], Value::Null);
    assert_eq!(body["person"]["phone"], Value::Null);

    let cookie = headers
        .get(SET_COOKIE)
        .context("login response must set the refresh cookie")?
        .to_str()?;
    assert!(cookie.starts_with("eq_rt="));
    assert!(cookie.contains("HttpOnly; Secure; SameSite=Strict; Path=/api/v1/auth"));
    let max_age = u64::from(fixture.policy.absolute_session_days()) * 86_400;
    assert!(cookie.ends_with(&format!("Max-Age={max_age}")));
    assert!(
        !body
            .to_string()
            .contains(cookie.split(';').next().unwrap_or_default())
    );
    assert_eq!(
        headers
            .get(axum::http::header::CACHE_CONTROL)
            .and_then(|value| value.to_str().ok()),
        Some("no-store")
    );

    let audit = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM audit.entries WHERE tenant_id = $1 AND entity_id = $2 AND action = 'security.login.success'",
    )
    .bind(fixture.tenant.0)
    .bind(fixture.person.0)
    .fetch_one(&fixture.owner_pool)
    .await?;
    assert_eq!(audit, 1);
    Ok(())
}

#[tokio::test]
async fn login_omits_pii_without_permission() -> Result<()> {
    let fixture = fixture_for_role(Role::Instructor, true).await?;
    let (status, _headers, body) = fixture.post(EMAIL, CORRECT_PASSWORD).await?;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["person"]["active_role"], "instructor");
    assert_eq!(body["person"]["email"], Value::Null);
    assert_eq!(body["person"]["phone"], Value::Null);
    let permissions = body["person"]["permissions"]
        .as_array()
        .context("login response must contain its permission snapshot")?;
    assert!(
        !permissions
            .iter()
            .any(|permission| permission == "identity.people.read_pii")
    );
    Ok(())
}

#[tokio::test]
async fn owner_gets_admin_perms_without_admin() -> Result<()> {
    let owner_without_admin = fixture_for_role_and_admin(Role::Owner, false, false).await?;
    let (status, _headers, body) = owner_without_admin.post(EMAIL, CORRECT_PASSWORD).await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["person"]["active_role"], "owner");
    assert!(
        body["person"]["permissions"]
            .as_array()
            .context("owner permission set is an array")?
            .iter()
            .any(|permission| permission == "identity.people.write")
    );

    let owner_with_admin = fixture_for_role_and_admin(Role::Owner, false, true).await?;
    let (status, _headers, body) = owner_with_admin.post(EMAIL, CORRECT_PASSWORD).await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["person"]["active_role"], "owner");
    assert!(
        !body["person"]["permissions"]
            .as_array()
            .context("owner permission set is an array")?
            .iter()
            .any(|permission| permission == "identity.people.write")
    );
    Ok(())
}

#[tokio::test]
async fn bearer_actor_rejects_cross_tenant_token() -> Result<()> {
    let source = fixture().await?;
    let (status, _headers, body) = source.post(EMAIL, CORRECT_PASSWORD).await?;
    assert_eq!(status, StatusCode::OK);
    let token = body["access_token"]
        .as_str()
        .context("login returns an access token")?;

    let target = fixture().await?;
    let (status, body) = target.get_protected_actor(token).await?;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["code"], "unauthenticated");
    Ok(())
}

#[tokio::test]
async fn bearer_actor_accepts_current_token() -> Result<()> {
    let fixture = fixture().await?;
    let (status, _headers, body) = fixture.post(EMAIL, CORRECT_PASSWORD).await?;
    assert_eq!(status, StatusCode::OK);
    let token = body["access_token"]
        .as_str()
        .context("login returns an access token")?;

    let (status, body) = fixture.get_protected_actor(token).await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["has_write"], true);
    Ok(())
}

#[tokio::test]
async fn bearer_actor_rejects_malformed_transport() -> Result<()> {
    let fixture = fixture().await?;
    let (status, body) = fixture
        .get_protected_actor_with_authorization("Bearer  test-secret-invalid-token")
        .await?;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["code"], "unauthenticated");
    Ok(())
}

#[tokio::test]
async fn bearer_actor_rejects_inactive_role() -> Result<()> {
    let fixture = fixture().await?;
    let (status, _headers, body) = fixture.post(EMAIL, CORRECT_PASSWORD).await?;
    assert_eq!(status, StatusCode::OK);
    let token = body["access_token"]
        .as_str()
        .context("login returns an access token")?;
    sqlx::query(
        "UPDATE public.memberships SET status = 'deactivated' WHERE tenant_id = $1 AND person_id = $2 AND role = 'admin'",
    )
    .bind(fixture.tenant.0)
    .bind(fixture.person.0)
    .execute(&fixture.owner_pool)
    .await?;

    let (status, body) = fixture.get_protected_actor(token).await?;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["code"], "unauthenticated");
    Ok(())
}

#[tokio::test]
async fn bearer_actor_rejects_stale_password_version() -> Result<()> {
    let fixture = fixture().await?;
    let (status, _headers, body) = fixture.post(EMAIL, CORRECT_PASSWORD).await?;
    assert_eq!(status, StatusCode::OK);
    let token = body["access_token"]
        .as_str()
        .context("login returns an access token")?;
    sqlx::query(
        "UPDATE public.credentials SET password_changed_at = now() WHERE tenant_id = $1 AND person_id = $2",
    )
    .bind(fixture.tenant.0)
    .bind(fixture.person.0)
    .execute(&fixture.owner_pool)
    .await?;

    let (status, body) = fixture.get_protected_actor(token).await?;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["code"], "unauthenticated");
    Ok(())
}

#[tokio::test]
async fn lockout_flow() -> Result<()> {
    let fixture = fixture().await?;
    let threshold = fixture.policy.lockout().max_failed();
    for _ in 0..threshold {
        let (status, _headers, body) = fixture.post(EMAIL, WRONG_PASSWORD).await?;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(body["code"], "invalid_credentials");
    }

    let (locked_status, _locked_headers, locked_body) = fixture.post(EMAIL, CORRECT_PASSWORD).await?;
    assert_eq!(locked_status, StatusCode::LOCKED);
    assert!(locked_body["until"].is_string());

    let (wrong_status, _wrong_headers, wrong_body) = fixture.post(EMAIL, WRONG_PASSWORD).await?;
    assert_eq!(wrong_status, StatusCode::UNAUTHORIZED);
    assert_eq!(wrong_body["code"], "invalid_credentials");
    assert!(wrong_body.get("until").is_none());

    let audit = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM audit.entries WHERE tenant_id = $1 AND entity_id = $2 AND action = 'security.account.locked'",
    )
    .bind(fixture.tenant.0)
    .bind(fixture.person.0)
    .fetch_one(&fixture.owner_pool)
    .await?;
    let event = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM public.outbox WHERE tenant_id = $1 AND event_type = 'identity.account_locked.v1'",
    )
    .bind(fixture.tenant.0)
    .fetch_one(&fixture.owner_pool)
    .await?;
    assert_eq!(audit, 1);
    assert_eq!(event, 1);
    Ok(())
}

#[tokio::test]
async fn login_constant_time() -> Result<()> {
    let fixture = fixture().await?;
    let mut unknown = Vec::with_capacity(20);
    let mut known_wrong = Vec::with_capacity(20);

    for index in 0..20 {
        let start = Instant::now();
        let (status, _headers, body) = fixture
            .post(&format!("missing-{index}@example.test"), WRONG_PASSWORD)
            .await?;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(body["code"], "invalid_credentials");
        unknown.push(start.elapsed());

        let start = Instant::now();
        let (status, _headers, body) = fixture.post(EMAIL, WRONG_PASSWORD).await?;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(body["code"], "invalid_credentials");
        known_wrong.push(start.elapsed());
    }

    unknown.sort_unstable();
    known_wrong.sort_unstable();
    let unknown_p50 = unknown[unknown.len() / 2];
    let known_p50 = known_wrong[known_wrong.len() / 2];
    let delta = unknown_p50.abs_diff(known_p50);
    assert!(
        delta <= Duration::from_millis(50),
        "p50 timing delta exceeded the acceptance bound"
    );
    Ok(())
}
