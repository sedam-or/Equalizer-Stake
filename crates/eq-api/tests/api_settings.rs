use anyhow::{Context, Result, ensure};
use async_trait::async_trait;
use chrono::Duration;
use eq_core::app_state::AppState;
use eq_core::clock::{Clock, SystemClock};
use eq_core::config::Config;
use eq_core::ids::{PersonId, TenantId};
use eq_core::model::Role;
use eq_core::model::TenantStatus;
use eq_core::module::ModuleId;
use eq_core::module::gate::EnabledModuleSet;
use eq_core::policy::CorePolicy;
use eq_core::ports::{CurrencyLockCheck, UnlockedCurrencyLockCheck};
use eq_core::tenant::{TenantCtx, TenantInfo};
use eq_core::{db::Tx, error::AppError, money::Currency, policy::types::Locale};
use eq_identity::api_auth::{self, LoginHttpState};
use eq_identity::jwt::{AccessToken, AccessTokenSubject, issue_access_token};
use reqwest::{Client, Method, StatusCode};
use secrecy::ExposeSecret;
use serde_json::{Value, json};
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::task::JoinHandle;
use uuid::Uuid;

const TEST_PASSWORD_VERSION: i64 = 1_700_000_000;

struct Fixture {
    owner_pool: PgPool,
    client: Client,
    address: SocketAddr,
    server: JoinHandle<std::io::Result<()>>,
    config: Arc<Config>,
    clock: Arc<SystemClock>,
    tenant: TenantId,
    owner: PersonId,
    admin: PersonId,
    guardian: PersonId,
}

impl Fixture {
    async fn new(currency_lock: bool) -> Result<Self> {
        let config = Arc::new(Config::from_env().context("load T-113b test configuration")?);
        ensure!(database_name(config.database_url.expose_secret()) == "ritma_test_T-113b");
        ensure!(database_name(config.database_owner_url.expose_secret()) == "ritma_test_T-113b");

        let app_pool = PgPoolOptions::new()
            .max_connections(6)
            .connect(config.database_url.expose_secret())
            .await
            .context("connect to the T-113b application database")?;
        let owner_pool = PgPoolOptions::new()
            .max_connections(2)
            .connect(config.database_owner_url.expose_secret())
            .await
            .context("connect to the T-113b database owner")?;
        verify_app_pool(&app_pool).await?;

        let clock = Arc::new(SystemClock);
        let tenant = TenantId(Uuid::now_v7());
        let owner = PersonId(Uuid::now_v7());
        let admin = PersonId(Uuid::now_v7());
        let guardian = PersonId(Uuid::now_v7());
        seed(&owner_pool, tenant, owner, admin, guardian, clock.now()).await?;

        let state = AppState::new(app_pool.clone(), Arc::clone(&config), clock.clone());
        let modules = Arc::new(eq_api::modules::registry());
        let enabled: Arc<dyn EnabledModuleSet> = Arc::new(AlwaysEnabled);
        let runtime = Arc::new(LoginHttpState::new(
            app_pool.clone(),
            clock.clone(),
            Arc::clone(&modules),
            Arc::clone(&enabled),
            Arc::clone(&state.auth),
            state.tenant_resolver.clone(),
        ));
        let currency_port: Arc<dyn CurrencyLockCheck> = if currency_lock {
            Arc::new(LockedCurrencyLockCheck)
        } else {
            Arc::new(UnlockedCurrencyLockCheck)
        };
        let metadata = eq_api::meta::MetadataState::new(
            Arc::clone(&modules),
            Arc::clone(&enabled),
            Arc::clone(&runtime),
        );
        let protected_routes = eq_api::settings::routes()
            .merge(eq_api::settings::module_routes())
            .merge(eq_api::meta::module_routes())
            .layer(axum::Extension((Arc::clone(&runtime), currency_port)))
            .layer(axum::Extension(metadata.clone()))
            .layer(axum::middleware::from_fn_with_state(
                Arc::clone(&runtime),
                api_auth::actor_auth_middleware,
            ));
        let routes = protected_routes.merge(eq_api::meta::public_routes().layer(axum::Extension(metadata)));
        let (routes, _openapi) = routes.split_for_parts();
        let tenant_context = TenantCtx::Resolved(Arc::new(TenantInfo {
            id: tenant,
            slug: format!("set-{}", tenant.0.simple()),
            status: TenantStatus::Active,
            timezone: String::from("UTC"),
            locale: Locale::En,
            currency: Currency::USD,
        }));
        let router = routes.with_state(state).layer(axum::Extension(tenant_context));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let address = listener.local_addr()?;
        let server = tokio::spawn(async move { axum::serve(listener, router).await });

        Ok(Self {
            owner_pool,
            client: Client::new(),
            address,
            server,
            config,
            clock,
            tenant,
            owner,
            admin,
            guardian,
        })
    }

    fn token_for_tenant(&self, person: PersonId, role: Role, tenant: TenantId) -> Result<AccessToken> {
        Ok(issue_access_token(
            &eq_core::crypto::Keys::derive(self.config.secret_key.expose_secret(), None),
            self.clock.as_ref(),
            AccessTokenSubject::new(person, tenant, role, Uuid::now_v7(), TEST_PASSWORD_VERSION),
        )?)
    }

    async fn request(
        &self,
        method: Method,
        path: &str,
        role: Role,
        body: Option<Value>,
        if_match: Option<&str>,
    ) -> Result<(StatusCode, Value)> {
        let person = match role {
            Role::Owner => self.owner,
            Role::Admin => self.admin,
            Role::Guardian => self.guardian,
            Role::Instructor | Role::Student => {
                anyhow::bail!("unsupported test role")
            }
        };
        let token = self.token_for_tenant(person, role, self.tenant)?;
        self.request_with_token(method, path, body, if_match, token).await
    }

    async fn request_with_token(
        &self,
        method: Method,
        path: &str,
        body: Option<Value>,
        if_match: Option<&str>,
        token: AccessToken,
    ) -> Result<(StatusCode, Value)> {
        let mut request = self
            .client
            .request(method, format!("http://{}{}", self.address, path))
            .bearer_auth(token.expose_secret());
        if let Some(body) = body {
            request = request.json(&body);
        }
        if let Some(etag) = if_match {
            request = request.header(reqwest::header::IF_MATCH, etag);
        }
        let response = request.send().await.context("send settings API request")?;
        let status = response.status();
        let body = response.json().await.context("decode settings API response")?;
        Ok((status, body))
    }

    async fn policy_count(&self) -> Result<i64> {
        Ok(sqlx::query_scalar(
            "SELECT count(*) FROM public.policies WHERE tenant_id = $1 AND module_id = 'core'",
        )
        .bind(self.tenant.0)
        .fetch_one(&self.owner_pool)
        .await?)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        self.server.abort();
    }
}

#[derive(Debug)]
struct AlwaysEnabled;

#[async_trait]
impl EnabledModuleSet for AlwaysEnabled {
    async fn is_enabled(&self, _tenant: TenantId, _module: ModuleId) -> Result<bool, AppError> {
        Ok(true)
    }
}

#[derive(Debug)]
struct LockedCurrencyLockCheck;

#[async_trait]
impl CurrencyLockCheck for LockedCurrencyLockCheck {
    async fn is_locked(&self, _tx: &mut Tx, _tenant: TenantId) -> Result<bool, AppError> {
        Ok(true)
    }
}

fn database_name(url: &str) -> &str {
    url.rsplit('/').next().unwrap_or_default()
}

async fn verify_app_pool(pool: &PgPool) -> Result<()> {
    let row = sqlx::query(
        "SELECT current_database() AS database, current_user AS username, rolsuper, rolbypassrls \
         FROM pg_roles WHERE rolname = current_user",
    )
    .fetch_one(pool)
    .await?;
    ensure!(row.try_get::<String, _>("database")? == "ritma_test_T-113b");
    ensure!(row.try_get::<String, _>("username")? == "eq_app");
    ensure!(!row.try_get::<bool, _>("rolsuper")? && !row.try_get::<bool, _>("rolbypassrls")?);
    Ok(())
}

async fn seed(
    owner_pool: &PgPool,
    tenant: TenantId,
    owner: PersonId,
    admin: PersonId,
    guardian: PersonId,
    now: chrono::DateTime<chrono::Utc>,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO public.tenants (id, slug, name, country, preset_id, preset_version) \
         VALUES ($1, $2, 'Test School', 'ID', 'blank', 1)",
    )
    .bind(tenant.0)
    .bind(format!("set-{}", tenant.0.simple()))
    .execute(owner_pool)
    .await?;
    for (person, role, name) in [
        (owner, "owner", "Test Owner"),
        (admin, "admin", "Test Admin"),
        (guardian, "guardian", "Test Guardian"),
    ] {
        sqlx::query("INSERT INTO public.persons (id, tenant_id, display_name) VALUES ($1, $2, $3)")
            .bind(person.0)
            .bind(tenant.0)
            .bind(name)
            .execute(owner_pool)
            .await?;
        sqlx::query(
            "INSERT INTO public.memberships (tenant_id, person_id, role, status) VALUES ($1, $2, $3, 'active')",
        )
        .bind(tenant.0)
        .bind(person.0)
        .bind(role)
        .execute(owner_pool)
        .await?;
        let password_changed_at = chrono::DateTime::from_timestamp(TEST_PASSWORD_VERSION, 0)
            .context("fixed identity fixture password version is a valid timestamp")?;
        sqlx::query(
            "INSERT INTO public.credentials (tenant_id, person_id, password_changed_at) VALUES ($1, $2, $3)",
        )
        .bind(tenant.0)
        .bind(person.0)
        .bind(password_changed_at)
        .execute(owner_pool)
        .await?;
    }
    let document = serde_json::to_value(CorePolicy::default())?;
    sqlx::query(
        "INSERT INTO public.policies (tenant_id, module_id, version, schema_version, effective_from, document, created_by) \
         VALUES ($1, 'core', 1, 1, $2, $3, $4)",
    )
    .bind(tenant.0)
    .bind(now - Duration::minutes(1))
    .bind(document)
    .bind(owner.0)
    .execute(owner_pool)
    .await?;
    Ok(())
}

fn core_document() -> Result<Value> {
    Ok(serde_json::to_value(CorePolicy::default())?)
}

#[tokio::test]
async fn preview_no_write() -> Result<()> {
    let fixture = Fixture::new(false).await?;
    let count_before = fixture.policy_count().await?;
    let (status, body) = fixture
        .request(
            Method::POST,
            "/api/v1/settings/policies/core/preview",
            Role::Owner,
            Some(json!({ "document": { "locale": "xx" } })),
            None,
        )
        .await?;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["valid"], false);
    assert!(
        !body["errors"]
            .as_array()
            .context("validation errors array")?
            .is_empty()
    );
    assert!(
        body["errors"]
            .as_array()
            .context("validation errors array")?
            .iter()
            .any(|error| error["pointer"] == "/locale")
    );
    let mut out_of_range = core_document()?;
    out_of_range["branding"]["brand_hue"] = json!(360);
    let (range_status, range_body) = fixture
        .request(
            Method::POST,
            "/api/v1/settings/policies/core/preview",
            Role::Owner,
            Some(json!({ "document": out_of_range })),
            None,
        )
        .await?;
    assert_eq!(range_status, StatusCode::OK, "{range_body}");
    assert_eq!(range_body["valid"], false);
    assert!(
        range_body["errors"]
            .as_array()
            .context("range validation errors array")?
            .iter()
            .any(|error| error["pointer"] == "/branding/brand_hue")
    );
    assert_eq!(fixture.policy_count().await?, count_before);
    Ok(())
}

#[tokio::test]
async fn put_policy_versioned() -> Result<()> {
    let fixture = Fixture::new(false).await?;
    assert_if_match_and_effective_time_errors(&fixture).await?;
    let mut document = core_document()?;
    document["locale"] = json!("id");
    let (status, body) = fixture
        .request(
            Method::PUT,
            "/api/v1/settings/policies/core",
            Role::Owner,
            Some(json!({ "document": document })),
            Some("\"v1\""),
        )
        .await?;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["version"], 2);
    let stale = fixture
        .request(
            Method::PUT,
            "/api/v1/settings/policies/core",
            Role::Owner,
            Some(json!({ "document": body["document"].clone() })),
            Some("\"v1\""),
        )
        .await?;
    assert_eq!(stale.0, StatusCode::CONFLICT, "{}", stale.1);
    assert_eq!(stale.1["code"], "version_conflict");

    assert_policy_audit(&fixture).await?;
    Ok(())
}

async fn assert_if_match_and_effective_time_errors(fixture: &Fixture) -> Result<()> {
    let document = core_document()?;
    let missing_etag = fixture
        .request(
            Method::PUT,
            "/api/v1/settings/policies/core",
            Role::Owner,
            Some(json!({ "document": document.clone() })),
            None,
        )
        .await?;
    assert_eq!(
        missing_etag.0,
        StatusCode::PRECONDITION_REQUIRED,
        "{}",
        missing_etag.1
    );
    assert_eq!(missing_etag.1["code"], "precondition_required");

    let malformed_etag = fixture
        .request(
            Method::PUT,
            "/api/v1/settings/policies/core",
            Role::Owner,
            Some(json!({ "document": document.clone() })),
            Some("v1"),
        )
        .await?;
    assert_eq!(malformed_etag.0, StatusCode::BAD_REQUEST, "{}", malformed_etag.1);
    assert_eq!(malformed_etag.1["code"], "bad_request");

    let past = fixture.clock.now() - Duration::minutes(6);
    let past_effective = fixture
        .request(
            Method::PUT,
            "/api/v1/settings/policies/core",
            Role::Owner,
            Some(json!({ "document": document, "effective_from": past })),
            Some("\"v1\""),
        )
        .await?;
    assert_eq!(
        past_effective.0,
        StatusCode::UNPROCESSABLE_ENTITY,
        "{}",
        past_effective.1
    );
    assert_eq!(past_effective.1["code"], "validation");
    assert_eq!(past_effective.1["errors"][0]["pointer"], "/effective_from");
    Ok(())
}

async fn assert_policy_audit(fixture: &Fixture) -> Result<()> {
    let tenant = sqlx::query("SELECT locale FROM public.tenants WHERE id = $1")
        .bind(fixture.tenant.0)
        .fetch_one(&fixture.owner_pool)
        .await?;
    assert_eq!(tenant.try_get::<String, _>("locale")?, "id");
    let audit = sqlx::query(
        "SELECT action, \"before\", \"after\" FROM audit.entries WHERE tenant_id = $1 AND action = 'core.policy.changed'",
    )
    .bind(fixture.tenant.0)
    .fetch_one(&fixture.owner_pool)
    .await?;
    assert_eq!(audit.try_get::<String, _>("action")?, "core.policy.changed");
    assert_eq!(
        audit
            .try_get::<Option<Value>, _>("before")?
            .as_ref()
            .map(|value| value["version"].as_i64()),
        Some(Some(1))
    );
    assert_eq!(
        audit
            .try_get::<Option<Value>, _>("after")?
            .as_ref()
            .map(|value| value["version"].as_i64()),
        Some(Some(2))
    );
    Ok(())
}

#[tokio::test]
async fn pending_future_shown() -> Result<()> {
    let fixture = Fixture::new(false).await?;
    let future = fixture.clock.now() + Duration::minutes(10);
    let mut document = core_document()?;
    document["locale"] = json!("id");
    let saved = fixture
        .request(
            Method::PUT,
            "/api/v1/settings/policies/core",
            Role::Owner,
            Some(json!({ "document": document, "effective_from": future })),
            Some("\"v1\""),
        )
        .await?;
    assert_eq!(saved.0, StatusCode::OK, "{}", saved.1);
    let (status, body) = fixture
        .request(Method::GET, "/api/v1/settings/policies", Role::Owner, None, None)
        .await?;
    assert_eq!(status, StatusCode::OK, "{body}");
    let entry = body
        .as_array()
        .context("policy list array")?
        .first()
        .context("Core policy entry")?;
    assert_eq!(entry["version"], 1);
    assert_eq!(entry["pending_future"]["version"], 2);
    Ok(())
}

#[tokio::test]
async fn admin_cannot_write_policy() -> Result<()> {
    let fixture = Fixture::new(false).await?;
    let (status, body) = fixture
        .request(
            Method::PUT,
            "/api/v1/settings/policies/core",
            Role::Admin,
            Some(json!({ "document": core_document()? })),
            Some("\"v1\""),
        )
        .await?;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
    assert_eq!(body["code"], "forbidden");
    assert_eq!(body["perm"], "core.policy.write");
    let token = fixture.token_for_tenant(fixture.owner, Role::Owner, TenantId(Uuid::now_v7()))?;
    let (cross_tenant_status, cross_tenant_body) = fixture
        .request_with_token(
            Method::PUT,
            "/api/v1/settings/policies/core",
            Some(json!({ "document": core_document()? })),
            Some("\"v1\""),
            token,
        )
        .await?;
    assert_eq!(
        cross_tenant_status,
        StatusCode::UNAUTHORIZED,
        "{cross_tenant_body}"
    );
    assert_eq!(cross_tenant_body["code"], "unauthenticated");
    assert_eq!(fixture.policy_count().await?, 1);
    Ok(())
}

#[tokio::test]
async fn currency_lock() -> Result<()> {
    let fixture = Fixture::new(true).await?;
    let mut document = core_document()?;
    document["currency"] = json!("IDR");
    let (status, body) = fixture
        .request(
            Method::PUT,
            "/api/v1/settings/policies/core",
            Role::Owner,
            Some(json!({ "document": document })),
            Some("\"v1\""),
        )
        .await?;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["code"], "policy_violation");
    assert_eq!(body["rule"], "core.currency.locked");
    assert_eq!(fixture.policy_count().await?, 1);
    let tenant = sqlx::query("SELECT currency FROM public.tenants WHERE id = $1")
        .bind(fixture.tenant.0)
        .fetch_one(&fixture.owner_pool)
        .await?;
    assert_eq!(tenant.try_get::<String, _>("currency")?, "USD");
    Ok(())
}

#[tokio::test]
async fn toggle_modules_api() -> Result<()> {
    let fixture = Fixture::new(false).await?;
    let (dependency_status, dependency_body) = fixture
        .request(
            Method::PUT,
            "/api/v1/settings/modules",
            Role::Owner,
            Some(json!({"enabled": ["billing"]})),
            None,
        )
        .await?;
    assert_eq!(
        dependency_status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "{dependency_body}"
    );
    assert_eq!(dependency_body["code"], "module_dependency");

    let (status, body) = fixture
        .request(
            Method::PUT,
            "/api/v1/settings/modules",
            Role::Owner,
            Some(json!({"enabled": ["enrollment", "billing"]})),
            None,
        )
        .await?;
    assert_eq!(status, StatusCode::OK, "{body}");
    let modules = body["modules"].as_array().context("module catalog array")?;
    for module in ["enrollment", "billing"] {
        assert!(
            modules
                .iter()
                .any(|item| item["id"] == module && item["enabled"] == true)
        );
    }
    Ok(())
}

#[tokio::test]
async fn meta_nav_filtered() -> Result<()> {
    let fixture = Fixture::new(false).await?;
    let (status, body) = fixture
        .request(Method::GET, "/api/v1/meta/modules", Role::Guardian, None, None)
        .await?;
    assert_eq!(status, StatusCode::OK, "{body}");
    let nav = body["nav"].as_array().context("navigation array")?;
    let guardian_grants = eq_api::modules::registry()
        .modules()
        .iter()
        .flat_map(|module| module.permissions())
        .filter(|permission| permission.roles.contains(&eq_core::actor::ActorRole::Guardian))
        .map(|permission| permission.key)
        .collect::<std::collections::BTreeSet<_>>();
    assert!(nav.iter().all(|item| {
        item["perm"]
            .as_str()
            .is_some_and(|permission| guardian_grants.contains(permission))
    }));
    Ok(())
}

#[tokio::test]
async fn meta_app_unprovisioned() -> Result<()> {
    let config = Arc::new(Config::from_env().context("load unprovisioned metadata test config")?);
    let pool = PgPoolOptions::new().connect_lazy(config.database_url.expose_secret())?;
    let clock: Arc<dyn Clock> = Arc::new(SystemClock);
    let state = AppState::new(pool.clone(), Arc::clone(&config), Arc::clone(&clock));
    let modules = Arc::new(eq_api::modules::registry());
    let enabled: Arc<dyn EnabledModuleSet> = Arc::new(AlwaysEnabled);
    let runtime = Arc::new(LoginHttpState::new(
        pool,
        clock,
        Arc::clone(&modules),
        Arc::clone(&enabled),
        Arc::clone(&state.auth),
        state.tenant_resolver.clone(),
    ));
    let metadata = eq_api::meta::MetadataState::new(modules, enabled, runtime);
    let (routes, _openapi) = eq_api::meta::public_routes()
        .layer(axum::Extension(metadata))
        .split_for_parts();
    let router = routes
        .with_state(state)
        .layer(axum::Extension(TenantCtx::Unprovisioned));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
    let server = tokio::spawn(async move { axum::serve(listener, router).await });
    let result = Client::new()
        .get(format!("http://{address}/api/v1/meta/app"))
        .send()
        .await?;
    let status = result.status();
    let body: Value = result.json().await?;
    server.abort();
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["setup_required"], true);
    assert!(body.get("tenant").is_none());
    Ok(())
}
