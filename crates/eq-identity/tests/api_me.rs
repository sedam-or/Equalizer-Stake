use std::fmt;
use std::sync::Arc;

use anyhow::{Context, Result, ensure};
use async_trait::async_trait;
use axum::http::StatusCode;
use eq_core::app_state::AppState;
use eq_core::clock::Clock;
use eq_core::config::Config;
use eq_core::crypto::{Keys, bidx_phone_e164};
use eq_core::error::AppError;
use eq_core::fields::{FieldCtx, FieldKind, Password, PasswordInput};
use eq_core::ids::{PersonId, TenantId};
use eq_core::model::{Role, TenantStatus};
use eq_core::module::gate::{EnabledModuleSet, ModuleGate};
use eq_core::module::{CoreModule, Module, ModuleId, ModuleRegistry};
use eq_core::money::Currency;
use eq_core::policy::PolicyDoc;
use eq_core::policy::types::Locale;
use eq_core::tenant::TenantCtx;
use eq_identity::api_auth::LoginHttpState;
use eq_identity::jwt::{AccessToken, AccessTokenSubject, issue_access_token};
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

const CURRENT_PASSWORD: &str = "test-secret-current-password";
const NEW_PASSWORD: &str = "test-secret-new-password-strong";
const PHONE: &str = "+6281234567890";

struct Fixture {
    owner_pool: PgPool,
    client: Client,
    address: std::net::SocketAddr,
    server: tokio::task::JoinHandle<Result<(), std::io::Error>>,
    config: Arc<Config>,
    clock: Arc<eq_testkit::clock::ManualClock>,
    tenant: TenantId,
    guardian: PersonId,
    other_guardian: PersonId,
    student: PersonId,
    other_student: PersonId,
    current_session: Uuid,
    other_guardian_session: Uuid,
    revoke_session: Uuid,
    revoke_successor: Uuid,
}

impl Fixture {
    #[expect(
        clippy::too_many_lines,
        reason = "the fixture configures the task database, least-privilege pool, router and live HTTP listener together"
    )]
    async fn new() -> Result<Self> {
        let config = Arc::new(Config::from_env().context("load T-110f task database configuration")?);
        ensure!(database_name(config.database_url.expose_secret()) == "ritma_test_T-110d");
        ensure!(database_name(config.database_owner_url.expose_secret()) == "ritma_test_T-110d");
        let app_pool = PgPoolOptions::new()
            .max_connections(6)
            .connect(config.database_url.expose_secret())
            .await
            .context("connect to T-110f application database")?;
        let owner_pool = PgPoolOptions::new()
            .max_connections(2)
            .connect(config.database_owner_url.expose_secret())
            .await
            .context("connect to T-110f fixture owner")?;
        verify_app_pool(&app_pool).await?;

        let tenant = TenantId(Uuid::now_v7());
        let guardian = PersonId(Uuid::now_v7());
        let other_guardian = PersonId(Uuid::now_v7());
        let student = PersonId(Uuid::now_v7());
        let other_student = PersonId(Uuid::now_v7());
        let current_session = Uuid::now_v7();
        let other_guardian_session = Uuid::now_v7();
        let revoke_session = Uuid::now_v7();
        let revoke_successor = Uuid::now_v7();
        let clock = Arc::new(eq_testkit::clock::test_clock());
        let policy = IdentityPolicy::default();
        seed_identity(
            &owner_pool,
            tenant,
            guardian,
            other_guardian,
            student,
            other_student,
            current_session,
            other_guardian_session,
            revoke_session,
            revoke_successor,
            &policy,
            clock.now(),
        )
        .await?;

        let state = AppState::new(app_pool.clone(), Arc::clone(&config), clock.clone());
        let modules = Arc::new(ModuleRegistry::new(vec![
            Arc::new(CoreModule) as Arc<dyn Module>,
            Arc::new(IdentityModule) as Arc<dyn Module>,
        ]));
        let enabled: Arc<dyn EnabledModuleSet> = Arc::new(AlwaysEnabled);
        let runtime = Arc::new(LoginHttpState::new(
            app_pool,
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
            slug: format!("me-{}", tenant.0.simple()),
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
            config,
            clock,
            tenant,
            guardian,
            other_guardian,
            student,
            other_student,
            current_session,
            other_guardian_session,
            revoke_session,
            revoke_successor,
        })
    }

    fn access_token(&self, person: PersonId, session: Uuid) -> Result<AccessToken> {
        let password_version = if person == self.guardian {
            self.clock.now().timestamp()
        } else {
            0
        };
        Ok(issue_access_token(
            &Keys::derive(self.config.secret_key.expose_secret(), None),
            self.clock.as_ref(),
            AccessTokenSubject::new(person, self.tenant, Role::Guardian, session, password_version),
        )?)
    }

    async fn request(
        &self,
        method: reqwest::Method,
        path: &str,
        token: &AccessToken,
        body: Option<Value>,
        if_match: Option<&str>,
    ) -> Result<(StatusCode, reqwest::header::HeaderMap, Vec<u8>)> {
        let mut request = self
            .client
            .request(method, format!("http://{}{}", self.address, path))
            .bearer_auth(token.expose_secret());
        if let Some(body) = body {
            request = request.json(&body);
        }
        if let Some(if_match) = if_match {
            request = request.header(reqwest::header::IF_MATCH, if_match);
        }
        let response = request
            .send()
            .await
            .context("dispatch self-profile HTTP request")?;
        let status = StatusCode::from_u16(response.status().as_u16())?;
        let headers = response.headers().clone();
        let bytes = response
            .bytes()
            .await
            .context("read self-profile response")?
            .to_vec();
        Ok((status, headers, bytes))
    }

    async fn json_request(
        &self,
        method: reqwest::Method,
        path: &str,
        token: &AccessToken,
        body: Option<Value>,
        if_match: Option<&str>,
    ) -> Result<(StatusCode, reqwest::header::HeaderMap, Value)> {
        let (status, headers, bytes) = self.request(method, path, token, body, if_match).await?;
        let value = if bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&bytes).context("decode self-profile response JSON")?
        };
        Ok((status, headers, value))
    }

    async fn session_state(&self, id: Uuid) -> Result<Option<Option<chrono::DateTime<chrono::Utc>>>> {
        let row =
            sqlx::query("SELECT revoked_at FROM public.refresh_sessions WHERE tenant_id = $1 AND id = $2")
                .bind(self.tenant.0)
                .bind(id)
                .fetch_optional(&self.owner_pool)
                .await?;
        match row {
            Some(value) => Ok(Some(value.try_get("revoked_at")?)),
            None => Ok(None),
        }
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
    ensure!(role.try_get::<String, _>("database")? == "ritma_test_T-110d");
    ensure!(role.try_get::<String, _>("username")? == "eq_app");
    ensure!(!role.try_get::<bool, _>("rolsuper")? && !role.try_get::<bool, _>("rolbypassrls")?);
    Ok(())
}

#[expect(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    reason = "the fixture seeds the fixed T-110f identity, link, policy and session graph"
)]
async fn seed_identity(
    owner_pool: &PgPool,
    tenant: TenantId,
    guardian: PersonId,
    other_guardian: PersonId,
    student: PersonId,
    other_student: PersonId,
    current_session: Uuid,
    other_guardian_session: Uuid,
    revoke_session: Uuid,
    revoke_successor: Uuid,
    policy: &IdentityPolicy,
    now: chrono::DateTime<chrono::Utc>,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO public.tenants (id, slug, name, country, preset_id, preset_version) \
         VALUES ($1, $2, 'T-110f Test School', 'ID', 'test', 1)",
    )
    .bind(tenant.0)
    .bind(format!("me-{}", tenant.0.simple()))
    .execute(owner_pool)
    .await?;
    insert_person(owner_pool, tenant, guardian, "Guardian One").await?;
    insert_person(owner_pool, tenant, other_guardian, "Guardian Two").await?;
    insert_person(owner_pool, tenant, student, "Student One").await?;
    insert_person(owner_pool, tenant, other_student, "Student Two").await?;
    insert_membership(owner_pool, tenant, guardian, "guardian").await?;
    insert_membership(owner_pool, tenant, other_guardian, "guardian").await?;
    sqlx::query(
        "INSERT INTO public.person_links (tenant_id, guardian_id, student_id) VALUES ($1, $2, $3), ($1, $4, $5)",
    )
    .bind(tenant.0)
    .bind(guardian.0)
    .bind(student.0)
    .bind(other_guardian.0)
    .bind(other_student.0)
    .execute(owner_pool)
    .await?;

    let field_ctx = FieldCtx {
        country: None,
        tenant_currency: Currency::USD,
        tenant_locale: Locale::En,
        today: now.date_naive(),
        password_min_length: None,
    };
    let password_input = serde_json::from_value::<PasswordInput>(json!(CURRENT_PASSWORD))?;
    let password = Password::parse(password_input, &field_ctx)
        .map_err(|_| anyhow::anyhow!("fake test password is valid"))?;
    let password_hash = hash_password(&password)?;
    sqlx::query(
        "INSERT INTO public.credentials (tenant_id, person_id, password_hash, password_changed_at, failed_count) \
         VALUES ($1, $2, $3, $4, 0), ($1, $5, NULL, NULL, 0)",
    )
    .bind(tenant.0)
    .bind(guardian.0)
    .bind(password_hash.expose_secret())
    .bind(now)
    .bind(other_guardian.0)
    .execute(owner_pool)
    .await?;
    sqlx::query(
        "INSERT INTO public.policies (tenant_id, module_id, version, schema_version, effective_from, document) \
         VALUES ($1, 'identity', 1, $2, $3, $4)",
    )
    .bind(tenant.0)
    .bind(i32::try_from(IdentityPolicy::SCHEMA_VERSION)?)
    .bind(now - chrono::Duration::days(1))
    .bind(serde_json::to_value(policy)?)
    .execute(owner_pool)
    .await?;

    let current_family = Uuid::now_v7();
    let extra_family = Uuid::now_v7();
    let deleted_family = Uuid::now_v7();
    insert_session(
        owner_pool,
        tenant,
        guardian,
        current_session,
        current_family,
        now,
        "guardian",
    )
    .await?;
    insert_session(
        owner_pool,
        tenant,
        other_guardian,
        other_guardian_session,
        Uuid::now_v7(),
        now,
        "guardian",
    )
    .await?;
    insert_session(
        owner_pool,
        tenant,
        guardian,
        revoke_session,
        extra_family,
        now,
        "guardian",
    )
    .await?;
    insert_session(
        owner_pool,
        tenant,
        guardian,
        revoke_successor,
        extra_family,
        now,
        "guardian",
    )
    .await?;
    insert_session(
        owner_pool,
        tenant,
        guardian,
        Uuid::now_v7(),
        deleted_family,
        now,
        "guardian",
    )
    .await?;
    insert_session(
        owner_pool,
        tenant,
        other_guardian,
        Uuid::now_v7(),
        Uuid::now_v7(),
        now,
        "guardian",
    )
    .await?;
    Ok(())
}

async fn insert_person(pool: &PgPool, tenant: TenantId, person: PersonId, name: &str) -> Result<()> {
    sqlx::query("INSERT INTO public.persons (id, tenant_id, display_name, locale) VALUES ($1, $2, $3, 'en')")
        .bind(person.0)
        .bind(tenant.0)
        .bind(name)
        .execute(pool)
        .await?;
    Ok(())
}

async fn insert_membership(pool: &PgPool, tenant: TenantId, person: PersonId, role: &str) -> Result<()> {
    sqlx::query(
        "INSERT INTO public.memberships (tenant_id, person_id, role, status) VALUES ($1, $2, $3, 'active')",
    )
    .bind(tenant.0)
    .bind(person.0)
    .bind(role)
    .execute(pool)
    .await?;
    Ok(())
}

async fn insert_session(
    pool: &PgPool,
    tenant: TenantId,
    person: PersonId,
    id: Uuid,
    family: Uuid,
    now: chrono::DateTime<chrono::Utc>,
    role: &str,
) -> Result<()> {
    let token_hash = Sha256::digest(format!("test-secret-session-{id}").as_bytes()).to_vec();
    sqlx::query(
        "INSERT INTO public.refresh_sessions \
         (id, tenant_id, person_id, family_id, token_hash, active_role, created_at, last_used_at, idle_expires_at, absolute_expires_at, user_agent, ip) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $7, $8, $9, 'test-secret-browser', '192.0.2.10'::inet)",
    )
    .bind(id)
    .bind(tenant.0)
    .bind(person.0)
    .bind(family)
    .bind(token_hash)
    .bind(role)
    .bind(now)
    .bind(now + chrono::Duration::days(1))
    .bind(now + chrono::Duration::days(30))
    .execute(pool)
    .await?;
    Ok(())
}

#[tokio::test]
async fn me_get_scoped_to_actor() -> Result<()> {
    let fixture = Fixture::new().await?;
    let first_token = fixture.access_token(fixture.guardian, fixture.current_session)?;
    let (first_status, first_headers, first_body) = fixture
        .json_request(reqwest::Method::GET, "/api/v1/me", &first_token, None, None)
        .await?;
    assert_eq!(first_status, StatusCode::OK, "{first_body}");
    assert_eq!(first_body["id"], fixture.guardian.0.to_string());
    assert_eq!(first_body["display_name"], "Guardian One");
    assert_eq!(first_body["version"], 1);
    assert_eq!(
        first_headers
            .get(reqwest::header::ETAG)
            .and_then(|value| value.to_str().ok()),
        Some("\"v1\"")
    );
    assert_eq!(
        first_body["linked_students"][0]["id"],
        fixture.student.0.to_string()
    );
    assert_ne!(
        first_body["linked_students"][0]["id"],
        fixture.other_student.0.to_string()
    );
    assert_eq!(
        first_headers
            .get(reqwest::header::CACHE_CONTROL)
            .and_then(|value| value.to_str().ok()),
        Some("no-store")
    );

    let second_token = fixture.access_token(fixture.other_guardian, fixture.other_guardian_session)?;
    let (second_status, _, second_body) = fixture
        .json_request(reqwest::Method::GET, "/api/v1/me", &second_token, None, None)
        .await?;
    assert_eq!(second_status, StatusCode::OK, "{second_body}");
    assert_eq!(second_body["id"], fixture.other_guardian.0.to_string());
    assert_eq!(second_body["display_name"], "Guardian Two");
    assert_eq!(
        second_body["linked_students"][0]["id"],
        fixture.other_student.0.to_string()
    );
    Ok(())
}

#[tokio::test]
#[expect(
    clippy::too_many_lines,
    reason = "the acceptance case checks missing, stale and matching ETags plus invalid and persisted profile fields"
)]
async fn me_patch_requires_if_match() -> Result<()> {
    let fixture = Fixture::new().await?;
    let token = fixture.access_token(fixture.guardian, fixture.current_session)?;
    let (missing_status, _, missing_body) = fixture
        .json_request(
            reqwest::Method::PATCH,
            "/api/v1/me",
            &token,
            Some(json!({"display_name": "Updated Guardian"})),
            None,
        )
        .await?;
    assert_eq!(
        missing_status,
        StatusCode::PRECONDITION_REQUIRED,
        "{missing_body}"
    );
    assert_eq!(missing_body["code"], "precondition_required");

    let (stale_status, _, stale_body) = fixture
        .json_request(
            reqwest::Method::PATCH,
            "/api/v1/me",
            &token,
            Some(json!({"display_name": "Updated Guardian"})),
            Some("\"v99\""),
        )
        .await?;
    assert_eq!(stale_status, StatusCode::CONFLICT, "{stale_body}");
    assert_eq!(stale_body["code"], "version_conflict");
    assert_eq!(stale_body["current"]["version"], 1);

    let (updated_status, updated_headers, updated_body) = fixture
        .json_request(
            reqwest::Method::PATCH,
            "/api/v1/me",
            &token,
            Some(json!({"display_name": "Updated Guardian", "phone": PHONE, "locale": "id"})),
            Some("\"v1\""),
        )
        .await?;
    assert_eq!(updated_status, StatusCode::OK, "{updated_body}");
    assert_eq!(updated_body["display_name"], "Updated Guardian");
    assert_eq!(updated_body["phone"], PHONE);
    assert_eq!(updated_body["locale"], "id");
    assert_eq!(updated_body["version"], 2);
    assert_eq!(
        updated_headers
            .get(reqwest::header::ETAG)
            .and_then(|value| value.to_str().ok()),
        Some("\"v2\"")
    );

    let (null_status, _, null_body) = fixture
        .json_request(
            reqwest::Method::PATCH,
            "/api/v1/me",
            &token,
            Some(json!({"display_name": null})),
            Some("\"v2\""),
        )
        .await?;
    assert_eq!(null_status, StatusCode::UNPROCESSABLE_ENTITY, "{null_body}");
    assert_eq!(null_body["code"], "validation");
    let (bad_phone_status, _, bad_phone_body) = fixture
        .json_request(
            reqwest::Method::PATCH,
            "/api/v1/me",
            &token,
            Some(json!({"phone": "+1"})),
            Some("\"v2\""),
        )
        .await?;
    assert_eq!(
        bad_phone_status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "{bad_phone_body}"
    );
    assert_eq!(bad_phone_body["code"], "validation");

    let keys = Keys::derive(fixture.config.secret_key.expose_secret(), None);
    let expected_index = bidx_phone_e164(&keys, fixture.tenant, PHONE)?;
    let profile = sqlx::query(
        "SELECT phone_enc, phone_bidx, locale, version FROM public.persons WHERE tenant_id = $1 AND id = $2",
    )
    .bind(fixture.tenant.0)
    .bind(fixture.guardian.0)
    .fetch_one(&fixture.owner_pool)
    .await?;
    let encrypted_phone: Option<Vec<u8>> = profile.try_get("phone_enc")?;
    let phone_index: Option<Vec<u8>> = profile.try_get("phone_bidx")?;
    ensure!(encrypted_phone.is_some());
    assert_eq!(phone_index.as_deref(), Some(expected_index.as_slice()));
    assert_eq!(
        profile.try_get::<Option<String>, _>("locale")?.as_deref(),
        Some("id")
    );
    assert_eq!(profile.try_get::<i32, _>("version")?, 2);
    let profile_audit = sqlx::query(
        "SELECT action, actor_id FROM audit.entries WHERE tenant_id = $1 AND entity_id = $2 AND action = 'core.person.updated'",
    )
    .bind(fixture.tenant.0)
    .bind(fixture.guardian.0)
    .fetch_one(&fixture.owner_pool)
    .await?;
    assert_eq!(
        profile_audit.try_get::<String, _>("action")?,
        "core.person.updated"
    );
    assert_eq!(
        profile_audit.try_get::<Option<Uuid>, _>("actor_id")?,
        Some(fixture.guardian.0)
    );
    Ok(())
}

#[tokio::test]
#[expect(
    clippy::too_many_lines,
    reason = "the acceptance case verifies password rejection, family revocation, ownership isolation and audit in one flow"
)]
async fn me_password_and_sessions() -> Result<()> {
    let fixture = Fixture::new().await?;
    let token = fixture.access_token(fixture.guardian, fixture.current_session)?;
    let (list_status, _, list_body) = fixture
        .json_request(reqwest::Method::GET, "/api/v1/me/sessions", &token, None, None)
        .await?;
    assert_eq!(list_status, StatusCode::OK, "{list_body}");
    assert_eq!(list_body["sessions"].as_array().map(Vec::len), Some(4));
    assert!(list_body.to_string().contains("test-secret-browser"));
    assert!(list_body.to_string().contains("192.0.2.10"));

    let (other_status, _, other_body) = fixture
        .json_request(
            reqwest::Method::DELETE,
            &format!("/api/v1/me/sessions/{}", fixture.other_guardian_session),
            &token,
            None,
            None,
        )
        .await?;
    assert_eq!(other_status, StatusCode::NOT_FOUND, "{other_body}");
    assert_eq!(
        fixture.session_state(fixture.other_guardian_session).await?,
        Some(None)
    );

    let (family_status, _, family_body) = fixture
        .json_request(
            reqwest::Method::DELETE,
            &format!("/api/v1/me/sessions/{}", fixture.revoke_session),
            &token,
            None,
            None,
        )
        .await?;
    assert_eq!(family_status, StatusCode::NO_CONTENT, "{family_body}");
    assert!(
        fixture
            .session_state(fixture.revoke_session)
            .await?
            .flatten()
            .is_some()
    );
    assert!(
        fixture
            .session_state(fixture.revoke_successor)
            .await?
            .flatten()
            .is_some()
    );

    let (wrong_password_status, _, wrong_password_body) = fixture
        .json_request(
            reqwest::Method::POST,
            "/api/v1/me/password",
            &token,
            Some(json!({"current_password": "test-secret-wrong-password", "new_password": NEW_PASSWORD})),
            None,
        )
        .await?;
    assert_eq!(
        wrong_password_status,
        StatusCode::UNAUTHORIZED,
        "{wrong_password_body}"
    );
    assert_eq!(wrong_password_body["code"], "invalid_credentials");
    assert_eq!(fixture.session_state(fixture.current_session).await?, Some(None));

    let (change_status, _, change_body) = fixture
        .json_request(
            reqwest::Method::POST,
            "/api/v1/me/password",
            &token,
            Some(json!({"current_password": CURRENT_PASSWORD, "new_password": NEW_PASSWORD})),
            None,
        )
        .await?;
    assert_eq!(change_status, StatusCode::NO_CONTENT, "{change_body}");
    assert!(change_body.is_null());
    assert_eq!(fixture.session_state(fixture.current_session).await?, Some(None));
    let revoked_count = sqlx::query(
        "SELECT COUNT(*) AS count FROM public.refresh_sessions WHERE tenant_id = $1 AND person_id = $2 AND revoked_at IS NOT NULL",
    )
    .bind(fixture.tenant.0)
    .bind(fixture.guardian.0)
    .fetch_one(&fixture.owner_pool)
    .await?
    .try_get::<i64, _>("count")?;
    assert_eq!(revoked_count, 3);
    let password_audit = sqlx::query(
        "SELECT action, actor_id FROM audit.entries WHERE tenant_id = $1 AND entity_id = $2 AND action = 'security.password.changed'",
    )
    .bind(fixture.tenant.0)
    .bind(fixture.guardian.0)
    .fetch_one(&fixture.owner_pool)
    .await?;
    assert_eq!(
        password_audit.try_get::<String, _>("action")?,
        "security.password.changed"
    );
    assert_eq!(
        password_audit.try_get::<Option<Uuid>, _>("actor_id")?,
        Some(fixture.guardian.0)
    );
    Ok(())
}
