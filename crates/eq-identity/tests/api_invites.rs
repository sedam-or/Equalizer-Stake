use anyhow::{Context, Result, ensure};
use async_trait::async_trait;
use axum::body::{Body, to_bytes};
use axum::http::header::{CONTENT_TYPE, SET_COOKIE};
use axum::http::{HeaderMap, Method, Request, StatusCode};
use eq_core::actor::{ActorCtx, ActorRole};
use eq_core::app_state::AppState;
use eq_core::auth::PgIdentityDirectory;
use eq_core::clock::Clock;
use eq_core::config::Config;
use eq_core::crypto::{Keyring, Keys};
use eq_core::db;
use eq_core::error::AppError;
use eq_core::ids::{PersonId, TenantId};
use eq_core::model::{Role, TenantStatus};
use eq_core::module::gate::{EnabledModuleSet, ModuleGate};
use eq_core::module::{CoreModule, Module, ModuleId, ModuleRegistry};
use eq_core::money::Currency;
use eq_core::policy::PolicyDoc;
use eq_core::policy::types::Locale;
use eq_core::tenant::TenantCtx;
use eq_identity::api_auth::LoginHttpState;
use eq_identity::invites::{InviteChannel, InviteRequest, InviteService};
use eq_identity::module::IdentityModule;
use eq_identity::policy::IdentityPolicy;
use secrecy::{ExposeSecret, SecretString};
use serde_json::{Value, json};
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

struct Fixture {
    owner_pool: PgPool,
    app_pool: PgPool,
    router: axum::Router,
    directory: PgIdentityDirectory,
    keyring: Keyring,
    tenant: TenantId,
    person_id: PersonId,
    actor: ActorCtx,
    clock: Arc<eq_testkit::clock::ManualClock>,
    policy: IdentityPolicy,
    config: Arc<Config>,
}

impl Fixture {
    async fn new() -> Result<Self> {
        let (config, owner_pool, app_pool) = connect_task_pools().await?;
        let config = Arc::new(config);

        let tenant = TenantId(Uuid::now_v7());
        let actor_id = PersonId(Uuid::now_v7());
        let person_id = PersonId(Uuid::now_v7());
        let policy = IdentityPolicy::default();
        seed_people(&owner_pool, tenant, actor_id, person_id, &policy).await?;

        let master = [0x42; 32];
        let keys = Arc::new(Keys::derive(&master, None));
        let keyring = Keyring::new(Keys::derive(&master, None));
        let directory = PgIdentityDirectory::new(keys);
        let actor = ActorCtx {
            person_id: Some(actor_id),
            role: ActorRole::Admin,
            ip: None,
            request_id: Some(Uuid::now_v7()),
        };
        let clock = Arc::new(eq_testkit::clock::test_clock());
        let app_state = AppState::new(app_pool.clone(), Arc::clone(&config), clock.clone());
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
            Arc::clone(&app_state.auth),
            app_state.tenant_resolver.clone(),
        ));
        let (routes, _openapi) = IdentityModule.routes().split_for_parts();
        let routes = routes
            .layer(axum::middleware::from_fn_with_state(
                Arc::clone(&runtime),
                eq_identity::api_auth::actor_auth_middleware,
            ))
            .layer(ModuleGate::new(ModuleId::Identity, Arc::clone(&enabled)))
            .layer(axum::Extension(Arc::clone(&runtime)));
        let tenant_ctx = TenantCtx::Resolved(Arc::new(eq_core::tenant::TenantInfo {
            id: tenant,
            slug: format!("invite-{}", tenant.0.simple()),
            status: TenantStatus::Active,
            timezone: "UTC".to_owned(),
            locale: Locale::En,
            currency: Currency::USD,
        }));
        let router = routes.with_state(app_state).layer(axum::Extension(tenant_ctx));

        Ok(Self {
            owner_pool,
            app_pool,
            router,
            directory,
            keyring,
            tenant,
            person_id,
            actor,
            clock,
            policy,
            config,
        })
    }

    fn service(&self) -> InviteService<'_> {
        InviteService::new(&self.directory, &self.keyring, self.clock.as_ref())
    }

    fn request(&self, channel: InviteChannel) -> InviteRequest<'_> {
        InviteRequest::new(
            self.tenant,
            &self.actor,
            self.person_id,
            Role::Instructor,
            channel,
            Locale::En,
            &self.config,
            &self.policy,
        )
    }

    async fn create_link_invite(&self) -> Result<String> {
        let mut tx = db::begin(&self.app_pool, self.tenant, &self.actor).await?;
        let invite = self
            .service()
            .create_invite(&mut tx, self.request(InviteChannel::Link))
            .await?;
        tx.commit().await?;
        let url = invite
            .invite_url()
            .context("link channel returns a secret invite URL")?
            .expose_secret();
        Ok(url
            .rsplit('/')
            .next()
            .context("invite URL ends in its token")?
            .to_owned())
    }

    async fn get_preview(&self, token: &str) -> Result<(StatusCode, HeaderMap, Value)> {
        self.request_json(Method::GET, &format!("/api/v1/auth/invites/{token}"), None)
            .await
    }

    async fn accept(&self, token: &str, password: &str) -> Result<(StatusCode, HeaderMap, Value)> {
        self.request_json(
            Method::POST,
            &format!("/api/v1/auth/invites/{token}/accept"),
            Some(json!({"password": password})),
        )
        .await
    }

    async fn request_json(
        &self,
        method: Method,
        uri: &str,
        body: Option<Value>,
    ) -> Result<(StatusCode, HeaderMap, Value)> {
        let mut builder = Request::builder().method(method).uri(uri);
        let request_body = match body {
            Some(body) => {
                builder = builder.header(CONTENT_TYPE, "application/json");
                Body::from(body.to_string())
            }
            None => Body::empty(),
        };
        let request = builder.body(request_body)?;
        let response = self
            .router
            .clone()
            .oneshot(request)
            .await
            .context("dispatch invite HTTP request")?;
        let status = response.status();
        let headers = response.headers().clone();
        let bytes = to_bytes(response.into_body(), 1024 * 1024)
            .await
            .context("read invite HTTP response")?;
        let body = serde_json::from_slice(&bytes).context("decode invite response JSON")?;
        Ok((status, headers, body))
    }
}

struct AlwaysEnabled;

impl std::fmt::Debug for AlwaysEnabled {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("AlwaysEnabled")
    }
}

#[async_trait]
impl EnabledModuleSet for AlwaysEnabled {
    async fn is_enabled(&self, _tenant: TenantId, _module: ModuleId) -> std::result::Result<bool, AppError> {
        Ok(true)
    }
}

async fn connect_task_pools() -> Result<(Config, PgPool, PgPool)> {
    let config = Config::from_env().context("load task database configuration")?;
    ensure!(task_database(config.database_url.expose_secret()) == "ritma_test_T-110d");
    ensure!(task_database(config.database_owner_url.expose_secret()) == "ritma_test_T-110d");
    let owner_pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(config.database_owner_url.expose_secret())
        .await
        .context("connect to the task database owner")?;
    let app_pool = PgPoolOptions::new()
        .max_connections(4)
        .connect(config.database_url.expose_secret())
        .await
        .context("connect to the task database application role")?;
    verify_app_pool(&app_pool).await?;
    Ok((config, owner_pool, app_pool))
}

fn task_database(url: &str) -> &str {
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

async fn seed_people(
    owner_pool: &PgPool,
    tenant: TenantId,
    actor_id: PersonId,
    person_id: PersonId,
    policy: &IdentityPolicy,
) -> Result<()> {
    let slug = format!("invite-{}", tenant.0.simple());
    sqlx::query!(
        "INSERT INTO public.tenants (id, slug, name, country, preset_id, preset_version) \
         VALUES ($1, $2, 'Test School', 'ID', 'test', 1)",
        tenant.0,
        slug
    )
    .execute(owner_pool)
    .await?;
    sqlx::query!(
        "INSERT INTO public.persons (id, tenant_id, display_name, locale) VALUES ($1, $2, $3, $4)",
        actor_id.0,
        tenant.0,
        "Test Admin",
        Some("en")
    )
    .execute(owner_pool)
    .await?;
    sqlx::query!(
        "INSERT INTO public.persons (id, tenant_id, display_name, locale) VALUES ($1, $2, $3, $4)",
        person_id.0,
        tenant.0,
        "Test Invitee",
        Some("id")
    )
    .execute(owner_pool)
    .await?;
    sqlx::query!(
        "INSERT INTO public.memberships (tenant_id, person_id, role, status) \
         VALUES ($1, $2, 'admin', 'active')",
        tenant.0,
        actor_id.0
    )
    .execute(owner_pool)
    .await?;
    let schema_version = i32::try_from(IdentityPolicy::SCHEMA_VERSION)?;
    sqlx::query!(
        "INSERT INTO public.policies (tenant_id, module_id, version, schema_version, effective_from, document) \
         VALUES ($1, 'identity', 1, $2, '2026-01-04T00:00:00Z'::timestamptz, $3)",
        tenant.0,
        schema_version,
        serde_json::to_value(policy)?
    )
    .execute(owner_pool)
    .await?;
    Ok(())
}

#[tokio::test]
async fn resend_revokes_previous() -> Result<()> {
    let fixture = Fixture::new().await?;
    let service = fixture.service();

    let mut first_tx = db::begin(&fixture.app_pool, fixture.tenant, &fixture.actor).await?;
    let first = service
        .create_invite(&mut first_tx, fixture.request(InviteChannel::Link))
        .await?;
    first_tx.commit().await?;
    let first_url = first
        .invite_url()
        .context("link channel must return the first invite URL")?
        .expose_secret()
        .to_owned();
    ensure!(first_url.contains("/id/invite/"));
    let first_debug = format!("{first:?}");
    ensure!(!first_debug.contains(&first_url));

    fixture.clock.advance(chrono::Duration::seconds(1));
    let mut second_tx = db::begin(&fixture.app_pool, fixture.tenant, &fixture.actor).await?;
    let second = service
        .create_invite(&mut second_tx, fixture.request(InviteChannel::Link))
        .await?;
    second_tx.commit().await?;
    let second_url = second
        .invite_url()
        .context("link channel must return the resent invite URL")?
        .expose_secret()
        .to_owned();
    ensure!(first_url != second_url);

    let invite_rows = sqlx::query!(
        "SELECT expires_at, accepted_at FROM public.invites \
         WHERE tenant_id = $1 AND person_id = $2 AND role = 'instructor' ORDER BY created_at, id",
        fixture.tenant.0,
        fixture.person_id.0
    )
    .fetch_all(&fixture.owner_pool)
    .await?;
    ensure!(invite_rows.len() == 2);
    ensure!(invite_rows[0].expires_at == fixture.clock.now());
    ensure!(invite_rows[0].accepted_at.is_none());
    ensure!(invite_rows[1].expires_at > fixture.clock.now());

    let membership = sqlx::query!(
        "SELECT status FROM public.memberships WHERE tenant_id = $1 AND person_id = $2 AND role = 'instructor'",
        fixture.tenant.0,
        fixture.person_id.0
    )
    .fetch_one(&fixture.owner_pool)
    .await?;
    ensure!(membership.status == "invited");
    Ok(())
}

#[tokio::test]
async fn invited_event_excludes_token() -> Result<()> {
    let fixture = Fixture::new().await?;
    let service = fixture.service();
    let mut tx = db::begin(&fixture.app_pool, fixture.tenant, &fixture.actor).await?;
    let invite = service
        .create_invite(&mut tx, fixture.request(InviteChannel::Email))
        .await?;
    ensure!(invite.invite_url().is_none());
    let whatsapp_invite = service
        .create_invite(&mut tx, fixture.request(InviteChannel::Whatsapp))
        .await?;
    ensure!(whatsapp_invite.invite_url().is_none());
    tx.commit().await?;

    let row = sqlx::query!(
        "SELECT event_type, payload FROM public.outbox \
         WHERE tenant_id = $1 AND aggregate_id = $2 ORDER BY occurred_at DESC LIMIT 1",
        fixture.tenant.0,
        fixture.person_id.0
    )
    .fetch_one(&fixture.owner_pool)
    .await?;
    ensure!(row.event_type == "identity.user_invited.v1");
    let payload = row.payload["payload"]
        .as_object()
        .context("event payload must be an object")?;
    ensure!(payload.len() == 3);
    ensure!(payload.contains_key("person_id"));
    ensure!(payload.contains_key("role"));
    ensure!(payload.contains_key("invite_id"));
    ensure!(!row.payload.to_string().contains("token"));
    Ok(())
}

#[tokio::test]
async fn preview_ok() -> Result<()> {
    let fixture = Fixture::new().await?;
    let token = fixture.create_link_invite().await?;

    let (status, _, body) = fixture.get_preview(&token).await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["tenant_name"], "Test School");
    assert_eq!(body["display_name"], "Test Invitee");
    assert_eq!(body["role"], "instructor");
    assert!(body["expires_at"].as_str().is_some());
    assert!(!body.to_string().contains(&token));

    let other_tenant = TenantId(Uuid::now_v7());
    let other_slug = format!("other-{}", other_tenant.0.simple());
    sqlx::query!(
        "INSERT INTO public.tenants (id, slug, name, country, preset_id, preset_version) \
         VALUES ($1, $2, 'Other School', 'ID', 'test', 1)",
        other_tenant.0,
        other_slug
    )
    .execute(&fixture.owner_pool)
    .await?;

    let mut tx = db::begin(
        &fixture.app_pool,
        fixture.tenant,
        &ActorCtx::anonymous(None, None),
    )
    .await?;
    let own_name = sqlx::query_scalar!("SELECT public.resolve_tenant_name($1)", fixture.tenant.0)
        .fetch_one(tx.conn())
        .await?;
    let other_name = sqlx::query_scalar!("SELECT public.resolve_tenant_name($1)", other_tenant.0)
        .fetch_one(tx.conn())
        .await?;
    assert_eq!(own_name.as_deref(), Some("Test School"));
    assert!(other_name.is_none());
    tx.commit().await?;

    let mut other_tx = db::begin(&fixture.app_pool, other_tenant, &ActorCtx::anonymous(None, None)).await?;
    let cross_tenant = fixture
        .service()
        .preview_invite(&mut other_tx, other_tenant, &SecretString::from(token.clone()))
        .await;
    ensure!(matches!(cross_tenant, Err(AppError::NotFound { .. })));
    other_tx.rollback().await?;

    let (unknown_status, _, unknown_body) = fixture.get_preview("test-secret-unknown-token").await?;
    assert_eq!(unknown_status, StatusCode::NOT_FOUND);
    assert_eq!(unknown_body["code"], "not_found");
    Ok(())
}

#[tokio::test]
async fn accept_ok() -> Result<()> {
    let fixture = Fixture::new().await?;
    let token = fixture.create_link_invite().await?;
    let (preview_status, _, preview) = fixture.get_preview(&token).await?;
    assert_eq!(preview_status, StatusCode::OK);
    assert_eq!(preview["tenant_name"], "Test School");

    let (status, headers, body) = fixture
        .accept(&token, "test-secret-invite-password-strong")
        .await?;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(body["access_token"].as_str().is_some());
    assert_eq!(body["person"]["id"], fixture.person_id.0.to_string());
    assert_eq!(body["person"]["display_name"], "Test Invitee");
    assert_eq!(body["person"]["roles"], json!(["instructor"]));
    assert_eq!(body["active_role"], "instructor");
    assert_eq!(body["person"]["email"], Value::Null);
    let cookie = headers
        .get(SET_COOKIE)
        .context("accepted invite response sets refresh cookie")?
        .to_str()?;
    assert!(cookie.starts_with("eq_rt="));
    assert!(cookie.contains("HttpOnly; SameSite=Strict; Path=/api/v1/auth"));
    assert!(
        headers
            .get("cache-control")
            .is_some_and(|value| value == "no-store")
    );
    assert!(!body.to_string().contains(&token));
    assert!(!body.to_string().contains("test-secret-invite-password-strong"));

    let invite = sqlx::query!(
        "SELECT id, accepted_at FROM public.invites WHERE tenant_id = $1 AND person_id = $2 AND role = 'instructor'",
        fixture.tenant.0,
        fixture.person_id.0
    )
    .fetch_one(&fixture.owner_pool)
    .await?;
    assert!(invite.accepted_at.is_some());
    let membership = sqlx::query!(
        "SELECT status FROM public.memberships WHERE tenant_id = $1 AND person_id = $2 AND role = 'instructor'",
        fixture.tenant.0,
        fixture.person_id.0
    )
    .fetch_one(&fixture.owner_pool)
    .await?;
    assert_eq!(membership.status, "active");
    let credentials = sqlx::query!(
        "SELECT password_hash, password_changed_at FROM public.credentials WHERE tenant_id = $1 AND person_id = $2",
        fixture.tenant.0,
        fixture.person_id.0
    )
    .fetch_one(&fixture.owner_pool)
    .await?;
    assert!(credentials.password_hash.is_some());
    assert!(credentials.password_changed_at.is_some());
    let sessions = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM public.refresh_sessions WHERE tenant_id = $1 AND person_id = $2",
        fixture.tenant.0,
        fixture.person_id.0
    )
    .fetch_one(&fixture.owner_pool)
    .await?
    .context("refresh session count is non-null")?;
    assert_eq!(sessions, 1);
    let audit = sqlx::query!(
        "SELECT actor_id, actor_role, action, entity_id, after FROM audit.entries WHERE tenant_id = $1 AND entity_id = $2 AND action = 'identity.invite.accepted'",
        fixture.tenant.0,
        invite.id
    )
    .fetch_one(&fixture.owner_pool)
    .await?;
    assert_eq!(audit.actor_id, Some(fixture.person_id.0));
    assert_eq!(audit.actor_role, "instructor");
    assert_eq!(audit.action, "identity.invite.accepted");
    assert_eq!(
        audit.after.as_ref().map(ToString::to_string),
        Some(
            json!({"tenant_id": fixture.tenant.0, "person_id": fixture.person_id.0, "role": "instructor"})
                .to_string()
        )
    );
    Ok(())
}

#[tokio::test]
async fn accept_expired_410() -> Result<()> {
    let fixture = Fixture::new().await?;
    let expired_token = fixture.create_link_invite().await?;
    let expired_at = fixture.clock.now() - chrono::Duration::seconds(1);
    sqlx::query!(
        "UPDATE public.invites SET expires_at = $3 WHERE tenant_id = $1 AND person_id = $2 AND accepted_at IS NULL",
        fixture.tenant.0,
        fixture.person_id.0,
        expired_at
    )
    .execute(&fixture.owner_pool)
    .await?;
    let (status, _, body) = fixture
        .accept(&expired_token, "test-secret-invite-password-strong")
        .await?;
    assert_eq!(status, StatusCode::GONE);
    assert_eq!(body["type"], "https://ritma.dev/problems/invite-expired");
    assert_eq!(body["title"], "Invite expired");
    assert_eq!(body["status"], StatusCode::GONE.as_u16());
    assert_eq!(body["code"], "invite_expired");
    assert_eq!(body["detail"], "Gone");
    let pending = sqlx::query!(
        "SELECT accepted_at FROM public.invites WHERE tenant_id = $1 AND person_id = $2 AND accepted_at IS NULL",
        fixture.tenant.0,
        fixture.person_id.0
    )
    .fetch_all(&fixture.owner_pool)
    .await?;
    assert_eq!(pending.len(), 1);
    assert!(pending[0].accepted_at.is_none());

    let replay_token = fixture.create_link_invite().await?;
    let (accepted_status, _, accepted_body) = fixture
        .accept(&replay_token, "test-secret-invite-password-strong")
        .await?;
    assert_eq!(accepted_status, StatusCode::OK, "{accepted_body}");
    let (replay_status, _, replay_body) = fixture
        .accept(&replay_token, "test-secret-invite-password-strong")
        .await?;
    assert_eq!(replay_status, StatusCode::GONE);
    assert_eq!(replay_body["code"], "invite_expired");
    let session_count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM public.refresh_sessions WHERE tenant_id = $1 AND person_id = $2",
        fixture.tenant.0,
        fixture.person_id.0
    )
    .fetch_one(&fixture.owner_pool)
    .await?
    .context("refresh session count is non-null")?;
    assert_eq!(session_count, 1);
    Ok(())
}

#[tokio::test]
async fn accept_password_weak() -> Result<()> {
    let fixture = Fixture::new().await?;
    let token = fixture.create_link_invite().await?;
    let (status, _, body) = fixture.accept(&token, "short").await?;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["code"], "password_weak");
    let membership = sqlx::query!(
        "SELECT status FROM public.memberships WHERE tenant_id = $1 AND person_id = $2 AND role = 'instructor'",
        fixture.tenant.0,
        fixture.person_id.0
    )
    .fetch_one(&fixture.owner_pool)
    .await?;
    assert_eq!(membership.status, "invited");
    Ok(())
}

#[tokio::test]
async fn accept_requires_totp_for_enrolled_credential() -> Result<()> {
    let fixture = Fixture::new().await?;
    let token = fixture.create_link_invite().await?;
    sqlx::query!(
        "INSERT INTO public.credentials (tenant_id, person_id, totp_secret_enc, totp_enabled) VALUES ($1, $2, $3, true)",
        fixture.tenant.0,
        fixture.person_id.0,
        vec![0_u8; 32]
    )
    .execute(&fixture.owner_pool)
    .await?;
    let (status, _, body) = fixture
        .accept(&token, "test-secret-invite-password-strong")
        .await?;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["code"], "totp_required");
    let invite = sqlx::query!(
        "SELECT accepted_at FROM public.invites WHERE tenant_id = $1 AND person_id = $2",
        fixture.tenant.0,
        fixture.person_id.0
    )
    .fetch_one(&fixture.owner_pool)
    .await?;
    assert!(invite.accepted_at.is_none());
    let membership = sqlx::query!(
        "SELECT status FROM public.memberships WHERE tenant_id = $1 AND person_id = $2 AND role = 'instructor'",
        fixture.tenant.0,
        fixture.person_id.0
    )
    .fetch_one(&fixture.owner_pool)
    .await?;
    assert_eq!(membership.status, "invited");
    let password = sqlx::query_scalar!(
        "SELECT password_hash FROM public.credentials WHERE tenant_id = $1 AND person_id = $2",
        fixture.tenant.0,
        fixture.person_id.0
    )
    .fetch_one(&fixture.owner_pool)
    .await?;
    assert!(password.is_none());
    Ok(())
}
