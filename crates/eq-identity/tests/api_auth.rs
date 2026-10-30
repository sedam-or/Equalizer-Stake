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
use eq_core::clock::{Clock, SystemClock};
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
use totp_rs::{Algorithm, Builder, Secret};
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
    runtime: Arc<LoginHttpState>,
    clock: Arc<SystemClock>,
    tenant: TenantId,
    person: PersonId,
    policy: IdentityPolicy,
    public_origin: String,
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

    async fn request_http(
        &self,
        method: reqwest::Method,
        path: &str,
        token: Option<&str>,
        body: Option<Value>,
    ) -> Result<(StatusCode, reqwest::header::HeaderMap, Value)> {
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
        let mut request = reqwest::Client::new().request(method, format!("http://{address}{path}"));
        if let Some(token) = token {
            request = request.bearer_auth(token);
        }
        if let Some(body) = body {
            request = request.json(&body);
        }
        let response = request.send().await;
        server.abort();
        let _ = server.await;
        let response = response?;
        let status = StatusCode::from_u16(response.status().as_u16())?;
        let headers = response.headers().clone();
        let bytes = response.bytes().await.context("read auth flow HTTP response")?;
        let body = serde_json::from_slice(&bytes).context("decode auth flow response JSON")?;
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

    async fn post_switch_role_http(
        &self,
        cookie: Option<&str>,
        csrf: Option<&str>,
        origin: Option<&str>,
        role: &str,
    ) -> Result<(StatusCode, axum::http::HeaderMap, Value)> {
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
        let mut request = client
            .post(format!("http://{address}/api/v1/auth/switch-role"))
            .json(&json!({"role": role}));
        if let Some(cookie) = cookie {
            request = request.header(axum::http::header::COOKIE, cookie);
        }
        if let Some(csrf) = csrf {
            request = request.header("x-eq-csrf", csrf);
        }
        if let Some(origin) = origin {
            request = request.header(axum::http::header::ORIGIN, origin);
        }
        let response = request.send().await;
        server.abort();
        let _ = server.await;
        let response = response?;
        let status = response.status();
        let headers = response.headers().clone();
        let body = response.bytes().await.context("read role-switch response")?;
        let body = serde_json::from_slice(&body).context("decode role-switch response JSON")?;
        Ok((status, headers, body))
    }

    async fn post_refresh_http(
        &self,
        cookie: Option<&str>,
        csrf: Option<&str>,
        origin: Option<&str>,
    ) -> Result<(StatusCode, axum::http::HeaderMap, Value)> {
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
        let mut request = client.post(format!("http://{address}/api/v1/auth/refresh"));
        if let Some(cookie) = cookie {
            request = request.header(axum::http::header::COOKIE, cookie);
        }
        if let Some(csrf) = csrf {
            request = request.header("x-eq-csrf", csrf);
        }
        if let Some(origin) = origin {
            request = request.header(axum::http::header::ORIGIN, origin);
        }
        let response = request.send().await;
        server.abort();
        let _ = server.await;
        let response = response?;
        let status = response.status();
        let headers = response.headers().clone();
        let body = response.bytes().await.context("read refresh response")?;
        let body = serde_json::from_slice(&body).context("decode refresh response JSON")?;
        Ok((status, headers, body))
    }

    async fn login_cookie(&self) -> Result<(String, Value)> {
        let (status, headers, body) = self.post(EMAIL, CORRECT_PASSWORD).await?;
        ensure!(status == StatusCode::OK, "test login must succeed");
        let set_cookie = headers
            .get(SET_COOKIE)
            .context("login response must set eq_rt")?
            .to_str()?;
        let cookie = set_cookie
            .split(';')
            .next()
            .context("refresh cookie must have a name and value")?;
        Ok((cookie.to_owned(), body))
    }

    async fn refresh_session_snapshot(&self) -> Result<(Uuid, Uuid, Vec<u8>, String)> {
        let row = sqlx::query(
            "SELECT id, family_id, token_hash, active_role FROM public.refresh_sessions \
             WHERE tenant_id = $1 AND person_id = $2",
        )
        .bind(self.tenant.0)
        .bind(self.person.0)
        .fetch_one(&self.owner_pool)
        .await?;
        Ok((
            row.try_get("id")?,
            row.try_get("family_id")?,
            row.try_get("token_hash")?,
            row.try_get("active_role")?,
        ))
    }

    async fn refresh_family_snapshot(
        &self,
        family_id: Uuid,
    ) -> Result<Vec<Option<chrono::DateTime<chrono::Utc>>>> {
        let rows = sqlx::query(
            "SELECT revoked_at FROM public.refresh_sessions \
             WHERE tenant_id = $1 AND family_id = $2 ORDER BY created_at, id",
        )
        .bind(self.tenant.0)
        .bind(family_id)
        .fetch_all(&self.owner_pool)
        .await?;
        rows.into_iter()
            .map(|row| row.try_get("revoked_at").context("read refresh revocation state"))
            .collect()
    }
}

async fn protected_actor(actor: eq_core::auth::Actor) -> axum::Json<Value> {
    axum::Json(json!({
        "active_role": actor.role(),
        "has_write": actor.has("identity.people.write")
    }))
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
    let public_origin = config.public_url.origin().ascii_serialization();
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
        clock.clone(),
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
        .layer(axum::Extension(Arc::clone(&runtime)));
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
        runtime,
        clock,
        tenant,
        person,
        policy,
        public_origin,
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

fn current_totp_code(secret_base32: &str, now: chrono::DateTime<chrono::Utc>) -> Result<String> {
    let secret = Secret::try_from_base32(secret_base32)
        .map_err(|_| anyhow::anyhow!("generated test TOTP seed is valid Base32"))?;
    let totp = Builder::new()
        .with_algorithm(Algorithm::SHA1)
        .with_digits(6)
        .with_skew(1)
        .with_step_duration(30)
        .with_secret(secret.as_bytes())
        .build()
        .map_err(|_| anyhow::anyhow!("test TOTP configuration is valid"))?;
    let timestamp = u64::try_from(now.timestamp())
        .map_err(|_| anyhow::anyhow!("test timestamp is after the Unix epoch"))?;
    Ok(totp.generate(timestamp).to_string())
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
#[expect(
    clippy::too_many_lines,
    reason = "the named acceptance journey checks TOTP, recovery, replay, audit and tenant isolation together"
)]
async fn totp_flow() -> Result<()> {
    let fixture = fixture().await?;
    let (login_status, _login_headers, login_body) = fixture
        .request_http(
            reqwest::Method::POST,
            "/api/v1/auth/login",
            None,
            Some(json!({"email": EMAIL, "password": CORRECT_PASSWORD})),
        )
        .await?;
    assert_eq!(login_status, StatusCode::OK, "{login_body}");
    let access_token = login_body["access_token"]
        .as_str()
        .context("login returns a bearer token for TOTP setup")?;

    let (setup_status, _setup_headers, setup_body) = fixture
        .request_http(
            reqwest::Method::POST,
            "/api/v1/me/totp/setup",
            Some(access_token),
            None,
        )
        .await?;
    assert_eq!(setup_status, StatusCode::OK, "{setup_body}");
    let seed = setup_body["secret_base32"]
        .as_str()
        .context("setup returns its one-time TOTP seed")?;
    let confirmation_code = current_totp_code(seed, fixture.clock.now())?;
    let (confirm_status, _confirm_headers, confirm_body) = fixture
        .request_http(
            reqwest::Method::POST,
            "/api/v1/me/totp/confirm",
            Some(access_token),
            Some(json!({"code": confirmation_code})),
        )
        .await?;
    assert_eq!(confirm_status, StatusCode::OK, "{confirm_body}");
    let recovery_codes = confirm_body["recovery_codes"]
        .as_array()
        .context("confirmation returns one-time recovery codes")?;
    assert_eq!(recovery_codes.len(), 10);
    let recovery_code = recovery_codes[0]
        .as_str()
        .context("recovery code is a string")?
        .to_owned();

    let (missing_status, _missing_headers, missing_body) = fixture
        .request_http(
            reqwest::Method::POST,
            "/api/v1/auth/login",
            None,
            Some(json!({"email": EMAIL, "password": CORRECT_PASSWORD})),
        )
        .await?;
    assert_eq!(missing_status, StatusCode::UNAUTHORIZED, "{missing_body}");
    assert_eq!(missing_body["code"], "totp_required");

    let totp_code = current_totp_code(seed, fixture.clock.now())?;
    let (totp_status, _totp_headers, totp_body) = fixture
        .request_http(
            reqwest::Method::POST,
            "/api/v1/auth/login",
            None,
            Some(json!({"email": EMAIL, "password": CORRECT_PASSWORD, "totp": totp_code})),
        )
        .await?;
    assert_eq!(totp_status, StatusCode::OK, "{totp_body}");

    let foreign_tenant = TenantId(Uuid::now_v7());
    let foreign_person = PersonId(Uuid::now_v7());
    let foreign_code = "test-code1";
    sqlx::query(
        "INSERT INTO public.tenants (id, slug, name, country, preset_id, preset_version) \
         VALUES ($1, $2, 'Test Foreign Tenant', 'ID', 'test', 1)",
    )
    .bind(foreign_tenant.0)
    .bind(format!("foreign-{}", foreign_tenant.0.simple()))
    .execute(&fixture.owner_pool)
    .await?;
    sqlx::query(
        "INSERT INTO public.persons (id, tenant_id, display_name, locale) VALUES ($1, $2, 'Test Foreign Person', 'en')",
    )
    .bind(foreign_person.0)
    .bind(foreign_tenant.0)
    .execute(&fixture.owner_pool)
    .await?;
    let context = field_ctx()?;
    let foreign_password_input = serde_json::from_value::<PasswordInput>(json!(foreign_code))?;
    let foreign_password = Password::parse(foreign_password_input, &context)
        .map_err(|_| anyhow::anyhow!("fake foreign recovery value parses"))?;
    let foreign_hash = hash_password(&foreign_password)?;
    sqlx::query(
        "INSERT INTO public.totp_recovery_codes (tenant_id, person_id, code_hash) VALUES ($1, $2, $3)",
    )
    .bind(foreign_tenant.0)
    .bind(foreign_person.0)
    .bind(foreign_hash.expose_secret())
    .execute(&fixture.owner_pool)
    .await?;

    let (foreign_status, _foreign_headers, foreign_body) = fixture
        .request_http(
            reqwest::Method::POST,
            "/api/v1/auth/login",
            None,
            Some(json!({"email": EMAIL, "password": CORRECT_PASSWORD, "totp": foreign_code})),
        )
        .await?;
    assert_eq!(foreign_status, StatusCode::UNAUTHORIZED, "{foreign_body}");
    assert_eq!(foreign_body["code"], "invalid_credentials");

    let (recovery_status, _recovery_headers, recovery_body) = fixture
        .request_http(
            reqwest::Method::POST,
            "/api/v1/auth/login",
            None,
            Some(json!({"email": EMAIL, "password": CORRECT_PASSWORD, "totp": recovery_code})),
        )
        .await?;
    assert_eq!(recovery_status, StatusCode::OK, "{recovery_body}");
    assert!(recovery_body["access_token"].as_str().is_some());

    let (replay_status, _replay_headers, replay_body) = fixture
        .request_http(
            reqwest::Method::POST,
            "/api/v1/auth/login",
            None,
            Some(json!({"email": EMAIL, "password": CORRECT_PASSWORD, "totp": recovery_codes[0]})),
        )
        .await?;
    assert_eq!(replay_status, StatusCode::UNAUTHORIZED, "{replay_body}");
    assert_eq!(replay_body["code"], "invalid_credentials");

    let used_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM public.totp_recovery_codes WHERE tenant_id = $1 AND person_id = $2 AND used_at IS NOT NULL",
    )
    .bind(fixture.tenant.0)
    .bind(fixture.person.0)
    .fetch_one(&fixture.owner_pool)
    .await?;
    assert_eq!(used_count, 1, "one successful recovery login consumes one code");
    let foreign_used_at = sqlx::query_scalar::<_, Option<chrono::DateTime<chrono::Utc>>>(
        "SELECT used_at FROM public.totp_recovery_codes WHERE tenant_id = $1 AND person_id = $2",
    )
    .bind(foreign_tenant.0)
    .bind(foreign_person.0)
    .fetch_one(&fixture.owner_pool)
    .await?;
    assert!(
        foreign_used_at.is_none(),
        "foreign recovery codes are not consumed"
    );
    let login_success_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM audit.entries WHERE tenant_id = $1 AND entity_id = $2 AND action = 'security.login.success'",
    )
    .bind(fixture.tenant.0)
    .bind(fixture.person.0)
    .fetch_one(&fixture.owner_pool)
    .await?;
    assert_eq!(
        login_success_count, 3,
        "normal password, TOTP and recovery logins are audited"
    );
    Ok(())
}

#[tokio::test]
async fn switch_role() -> Result<()> {
    let fixture = fixture_for_role_and_admin(Role::Guardian, false, true).await?;
    let (cookie, login_body) = fixture.login_cookie().await?;
    assert_eq!(login_body["active_role"], "admin");

    let (missing_status, _missing_headers, missing_body) = fixture
        .post_switch_role_http(None, Some("1"), Some(&fixture.public_origin), "guardian")
        .await?;
    assert_eq!(missing_status, StatusCode::UNAUTHORIZED);
    assert_eq!(missing_body["code"], "unauthenticated");

    let (malformed_status, _malformed_headers, malformed_body) = fixture
        .post_switch_role_http(
            Some("eq_rt=test-secret-invalid-cookie"),
            Some("1"),
            Some(&fixture.public_origin),
            "guardian",
        )
        .await?;
    assert_eq!(malformed_status, StatusCode::UNAUTHORIZED);
    assert_eq!(malformed_body["code"], "unauthenticated");

    let (status, headers, body) = fixture
        .post_switch_role_http(Some(&cookie), Some("1"), Some(&fixture.public_origin), "guardian")
        .await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["active_role"], "guardian");
    assert_eq!(body["person"]["active_role"], "guardian");
    assert_eq!(body["person"]["roles"], json!(["admin", "guardian"]));
    assert_eq!(body["roles"], json!(["admin", "guardian"]));
    assert_ne!(body["access_token"], login_body["access_token"]);
    assert!(!headers.contains_key(SET_COOKIE));
    let permissions = body["person"]["permissions"]
        .as_array()
        .context("role-switch response must contain guardian permissions")?;
    assert!(
        !permissions
            .iter()
            .any(|permission| permission == "identity.people.write")
    );

    let access_token = body["access_token"]
        .as_str()
        .context("role-switch response must contain an access token")?;
    let (actor_status, actor_body) = fixture.get_protected_actor(access_token).await?;
    assert_eq!(actor_status, StatusCode::OK);
    assert_eq!(actor_body["active_role"], "guardian");
    assert_eq!(actor_body["has_write"], false);
    Ok(())
}

#[tokio::test]
async fn switch_role_requires_csrf() -> Result<()> {
    let fixture = fixture_for_role_and_admin(Role::Guardian, false, true).await?;
    let (cookie, _) = fixture.login_cookie().await?;
    let before = fixture.refresh_session_snapshot().await?;
    let invalid_headers = [
        (None, Some(fixture.public_origin.as_str())),
        (Some("0"), Some(fixture.public_origin.as_str())),
        (Some("1"), None),
        (Some("1"), Some("https://attacker.example")),
    ];

    for (csrf, origin) in invalid_headers {
        let (status, _headers, body) = fixture
            .post_switch_role_http(Some(&cookie), csrf, origin, "guardian")
            .await?;
        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(body["code"], "forbidden");
    }

    let after = fixture.refresh_session_snapshot().await?;
    assert_eq!(before, after);
    let audit = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM audit.entries WHERE tenant_id = $1 AND action = 'security.role_switched'",
    )
    .bind(fixture.tenant.0)
    .fetch_one(&fixture.owner_pool)
    .await?;
    assert_eq!(audit, 0);
    Ok(())
}

#[tokio::test]
async fn refresh_reuse_revokes_family() -> Result<()> {
    let fixture = fixture().await?;
    let (r1_cookie, _) = fixture.login_cookie().await?;
    let (_, family_id, _, _) = fixture.refresh_session_snapshot().await?;

    let (rotated_status, rotated_headers, rotated_body) = fixture
        .post_refresh_http(Some(&r1_cookie), Some("1"), Some(&fixture.public_origin))
        .await?;
    assert_eq!(rotated_status, StatusCode::OK);
    assert!(rotated_body["access_token"].as_str().is_some());
    let r2_cookie = rotated_headers
        .get(SET_COOKIE)
        .context("refresh response must set the rotated eq_rt cookie")?
        .to_str()?
        .split(';')
        .next()
        .context("rotated refresh cookie must have a name and value")?
        .to_owned();
    assert_ne!(r1_cookie, r2_cookie);

    let (reuse_status, _reuse_headers, reuse_body) = fixture
        .post_refresh_http(Some(&r1_cookie), Some("1"), Some(&fixture.public_origin))
        .await?;
    assert_eq!(reuse_status, StatusCode::UNAUTHORIZED);
    assert_eq!(reuse_body["code"], "session_expired");
    assert_eq!(fixture.refresh_family_snapshot(family_id).await?.len(), 2);
    assert!(
        fixture
            .refresh_family_snapshot(family_id)
            .await?
            .iter()
            .all(Option::is_some)
    );

    let (successor_status, _successor_headers, successor_body) = fixture
        .post_refresh_http(Some(&r2_cookie), Some("1"), Some(&fixture.public_origin))
        .await?;
    assert_eq!(successor_status, StatusCode::UNAUTHORIZED);
    assert_eq!(successor_body["code"], "session_expired");

    let audit = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM audit.entries WHERE tenant_id = $1 AND action = 'security.refresh_reuse'",
    )
    .bind(fixture.tenant.0)
    .fetch_one(&fixture.owner_pool)
    .await?;
    assert_eq!(audit, 1);
    Ok(())
}

#[tokio::test]
async fn refresh_requires_csrf() -> Result<()> {
    let fixture = fixture().await?;
    let (cookie, _) = fixture.login_cookie().await?;
    let before = fixture.refresh_session_snapshot().await?;

    let (status, _headers, body) = fixture.post_refresh_http(Some(&cookie), None, None).await?;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["code"], "forbidden");
    assert_eq!(before, fixture.refresh_session_snapshot().await?);
    Ok(())
}

#[tokio::test]
async fn password_change_invalidates_access() -> Result<()> {
    let fixture = fixture().await?;
    let (status, _headers, body) = fixture.post(EMAIL, CORRECT_PASSWORD).await?;
    assert_eq!(status, StatusCode::OK);
    let token = body["access_token"]
        .as_str()
        .context("login returns an access token")?;

    let (before_status, _before_body) = fixture.get_protected_actor(token).await?;
    assert_eq!(before_status, StatusCode::OK);
    sqlx::query(
        "UPDATE public.credentials SET password_changed_at = now() WHERE tenant_id = $1 AND person_id = $2",
    )
    .bind(fixture.tenant.0)
    .bind(fixture.person.0)
    .execute(&fixture.owner_pool)
    .await?;
    fixture
        .runtime
        .invalidate_password_version(fixture.tenant, fixture.person)
        .await;

    let (after_status, after_body) = fixture.get_protected_actor(token).await?;
    assert_eq!(after_status, StatusCode::UNAUTHORIZED);
    assert_eq!(after_body["code"], "unauthenticated");
    Ok(())
}

#[tokio::test]
async fn switch_role_rejects_inactive_role() -> Result<()> {
    let fixture = fixture().await?;
    let (cookie, _) = fixture.login_cookie().await?;
    let before = fixture.refresh_session_snapshot().await?;

    let (status, _headers, body) = fixture
        .post_switch_role_http(Some(&cookie), Some("1"), Some(&fixture.public_origin), "guardian")
        .await?;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["code"], "forbidden");
    assert_eq!(before, fixture.refresh_session_snapshot().await?);

    let audit = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM audit.entries WHERE tenant_id = $1 AND action = 'security.role_switched'",
    )
    .bind(fixture.tenant.0)
    .fetch_one(&fixture.owner_pool)
    .await?;
    assert_eq!(audit, 0);
    Ok(())
}

#[tokio::test]
async fn switch_role_rejects_expired_session() -> Result<()> {
    let fixture = fixture_for_role_and_admin(Role::Guardian, false, true).await?;
    let (cookie, _) = fixture.login_cookie().await?;
    let before = fixture.refresh_session_snapshot().await?;
    sqlx::query(
        "UPDATE public.refresh_sessions SET idle_expires_at = clock_timestamp() - interval '1 second' \
         WHERE tenant_id = $1 AND id = $2",
    )
    .bind(fixture.tenant.0)
    .bind(before.0)
    .execute(&fixture.owner_pool)
    .await?;

    let (status, _headers, body) = fixture
        .post_switch_role_http(Some(&cookie), Some("1"), Some(&fixture.public_origin), "guardian")
        .await?;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["code"], "unauthenticated");
    assert_eq!(before, fixture.refresh_session_snapshot().await?);

    let audit = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM audit.entries WHERE tenant_id = $1 AND action = 'security.role_switched'",
    )
    .bind(fixture.tenant.0)
    .fetch_one(&fixture.owner_pool)
    .await?;
    assert_eq!(audit, 0);
    Ok(())
}

#[tokio::test]
async fn switch_role_rejects_revoked_session() -> Result<()> {
    let fixture = fixture_for_role_and_admin(Role::Guardian, false, true).await?;
    let (cookie, _) = fixture.login_cookie().await?;
    let before = fixture.refresh_session_snapshot().await?;
    sqlx::query(
        "UPDATE public.refresh_sessions SET revoked_at = clock_timestamp() \
         WHERE tenant_id = $1 AND id = $2",
    )
    .bind(fixture.tenant.0)
    .bind(before.0)
    .execute(&fixture.owner_pool)
    .await?;

    let (status, _headers, body) = fixture
        .post_switch_role_http(Some(&cookie), Some("1"), Some(&fixture.public_origin), "guardian")
        .await?;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["code"], "unauthenticated");
    assert_eq!(before, fixture.refresh_session_snapshot().await?);

    let audit = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM audit.entries WHERE tenant_id = $1 AND action = 'security.role_switched'",
    )
    .bind(fixture.tenant.0)
    .fetch_one(&fixture.owner_pool)
    .await?;
    assert_eq!(audit, 0);
    Ok(())
}

#[tokio::test]
async fn switch_role_keeps_refresh_family() -> Result<()> {
    let fixture = fixture_for_role_and_admin(Role::Guardian, false, true).await?;
    let (cookie, _) = fixture.login_cookie().await?;
    let before = fixture.refresh_session_snapshot().await?;

    let (status, headers, body) = fixture
        .post_switch_role_http(Some(&cookie), Some("1"), Some(&fixture.public_origin), "guardian")
        .await?;
    assert_eq!(status, StatusCode::OK);
    assert!(!headers.contains_key(SET_COOKIE));
    let after = fixture.refresh_session_snapshot().await?;
    assert_eq!(before.0, after.0);
    assert_eq!(before.1, after.1);
    assert_eq!(before.2, after.2);
    assert_eq!(after.3, "guardian");

    let audit = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM audit.entries WHERE tenant_id = $1 AND entity_id = $2 \
         AND action = 'security.role_switched' AND category = 'security'",
    )
    .bind(fixture.tenant.0)
    .bind(before.0)
    .fetch_one(&fixture.owner_pool)
    .await?;
    assert_eq!(audit, 1);
    assert_eq!(body["active_role"], "guardian");
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
