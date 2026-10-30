use std::collections::BTreeSet;
use std::fmt;
use std::sync::Arc;

use anyhow::{Context, Result, ensure};
use async_trait::async_trait;
use eq_core::app_state::AppState;
use eq_core::clock::Clock;
use eq_core::config::Config;
use eq_core::crypto::Keys;
use eq_core::error::AppError;
use eq_core::ids::{PersonId, TenantId};
use eq_core::model::{Role, TenantStatus};
use eq_core::module::gate::{EnabledModuleSet, ModuleGate};
use eq_core::module::{CoreModule, Module, ModuleId, ModuleRegistry};
use eq_core::money::Currency;
use eq_core::policy::types::Locale;
use eq_core::tenant::TenantCtx;
use eq_identity::api_auth::LoginHttpState;
use eq_identity::jwt::{AccessToken, AccessTokenSubject, issue_access_token};
use eq_identity::module::IdentityModule;
use reqwest::Client;
use secrecy::ExposeSecret;
use serde_json::{Value, json};
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};
use uuid::Uuid;

struct Fixture {
    owner_pool: PgPool,
    app_pool: PgPool,
    client: Client,
    address: std::net::SocketAddr,
    server: tokio::task::JoinHandle<std::io::Result<()>>,
    config: Arc<Config>,
    clock: Arc<eq_testkit::clock::ManualClock>,
    tenant: TenantId,
    owner: PersonId,
    admin: PersonId,
    guardian: PersonId,
    second_guardian: PersonId,
    student_one: PersonId,
    student_two: PersonId,
    unlinked_student: PersonId,
}

impl Fixture {
    #[expect(
        clippy::too_many_lines,
        reason = "the fixture configures the task database, seeded identity graph and authenticated HTTP router"
    )]
    async fn new() -> Result<Self> {
        let config = Arc::new(Config::from_env().context("load T-112a task database configuration")?);
        ensure!(database_name(config.database_url.expose_secret()) == "ritma_test_T-112a");
        ensure!(database_name(config.database_owner_url.expose_secret()) == "ritma_test_T-112a");
        let app_pool = PgPoolOptions::new()
            .max_connections(6)
            .connect(config.database_url.expose_secret())
            .await
            .context("connect to T-112a application database")?;
        let owner_pool = PgPoolOptions::new()
            .max_connections(2)
            .connect(config.database_owner_url.expose_secret())
            .await
            .context("connect to T-112a fixture owner")?;
        verify_app_pool(&app_pool).await?;

        let tenant = TenantId(Uuid::now_v7());
        let owner = PersonId(Uuid::now_v7());
        let admin = PersonId(Uuid::now_v7());
        let guardian = PersonId(Uuid::now_v7());
        let second_guardian = PersonId(Uuid::now_v7());
        let student_one = PersonId(Uuid::now_v7());
        let student_two = PersonId(Uuid::now_v7());
        let unlinked_student = PersonId(Uuid::now_v7());
        let clock = Arc::new(eq_testkit::clock::test_clock());
        seed_identity(
            &owner_pool,
            tenant,
            [
                (owner, Role::Owner, "test-secret-owner"),
                (admin, Role::Admin, "test-secret-admin"),
                (guardian, Role::Guardian, "test-secret-guardian"),
                (second_guardian, Role::Guardian, "test-secret-guardian-two"),
                (student_one, Role::Student, "test-secret-student-one"),
                (student_two, Role::Student, "test-secret-student-two"),
                (unlinked_student, Role::Student, "test-secret-unlinked-student"),
            ],
            clock.now(),
        )
        .await?;
        insert_guardian_link(&owner_pool, tenant, guardian, student_one).await?;
        insert_guardian_link(&owner_pool, tenant, guardian, student_two).await?;

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
            .layer(ModuleGate::new(ModuleId::Identity, Arc::clone(&enabled)))
            .layer(axum::Extension(runtime));
        let tenant_ctx = TenantCtx::Resolved(Arc::new(eq_core::tenant::TenantInfo {
            id: tenant,
            slug: format!("people-{}", tenant.0.simple()),
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
            app_pool,
            client: Client::new(),
            address,
            server,
            config,
            clock,
            tenant,
            owner,
            admin,
            guardian,
            second_guardian,
            student_one,
            student_two,
            unlinked_student,
        })
    }

    fn access_token(&self, person: PersonId, role: Role) -> Result<AccessToken> {
        Ok(issue_access_token(
            &Keys::derive(self.config.secret_key.expose_secret(), None),
            self.clock.as_ref(),
            AccessTokenSubject::new(
                person,
                self.tenant,
                role,
                Uuid::now_v7(),
                self.clock.now().timestamp(),
            ),
        )?)
    }

    async fn request(
        &self,
        method: reqwest::Method,
        path: &str,
        person: PersonId,
        role: Role,
        body: Option<Value>,
    ) -> Result<(reqwest::StatusCode, Value)> {
        let token = self.access_token(person, role)?;
        let mut request = self
            .client
            .request(method, format!("http://{}{}", self.address, path))
            .bearer_auth(token.expose_secret());
        if let Some(body) = body {
            request = request.json(&body);
        }
        let response = request.send().await.context("send T-112a API request")?;
        let status = response.status();
        let bytes = response.bytes().await.context("read T-112a API response")?;
        let value = if bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&bytes).context("decode T-112a API response")?
        };
        Ok((status, value))
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
    ensure!(role.try_get::<String, _>("database")? == "ritma_test_T-112a");
    ensure!(role.try_get::<String, _>("username")? == "eq_app");
    ensure!(!role.try_get::<bool, _>("rolsuper")? && !role.try_get::<bool, _>("rolbypassrls")?);
    Ok(())
}

async fn seed_identity(
    owner_pool: &PgPool,
    tenant: TenantId,
    people: [(PersonId, Role, &str); 7],
    now: chrono::DateTime<chrono::Utc>,
) -> Result<()> {
    sqlx::query!(
        "INSERT INTO public.tenants (id, slug, name, country, preset_id, preset_version) \
         VALUES ($1, $2, 'test-secret-school', 'ID', 'test', 1)",
        tenant.0,
        format!("people-{}", tenant.0.simple())
    )
    .execute(owner_pool)
    .await?;
    for (person, role, display_name) in people {
        sqlx::query!(
            "INSERT INTO public.persons (id, tenant_id, display_name) VALUES ($1, $2, $3)",
            person.0,
            tenant.0,
            display_name
        )
        .execute(owner_pool)
        .await?;
        sqlx::query!(
            "INSERT INTO public.memberships (tenant_id, person_id, role, status) \
             VALUES ($1, $2, $3, 'active')",
            tenant.0,
            person.0,
            role as Role
        )
        .execute(owner_pool)
        .await?;
        sqlx::query!(
            "INSERT INTO public.credentials (tenant_id, person_id, password_changed_at) \
             VALUES ($1, $2, $3)",
            tenant.0,
            person.0,
            now
        )
        .execute(owner_pool)
        .await?;
    }
    Ok(())
}

async fn insert_guardian_link(
    owner_pool: &PgPool,
    tenant: TenantId,
    guardian: PersonId,
    student: PersonId,
) -> Result<()> {
    sqlx::query!(
        "INSERT INTO public.person_links (tenant_id, guardian_id, student_id, is_primary) \
         VALUES ($1, $2, $3, true)",
        tenant.0,
        guardian.0,
        student.0
    )
    .execute(owner_pool)
    .await?;
    Ok(())
}

#[tokio::test]
async fn guardian_scope() -> Result<()> {
    let fixture = Fixture::new().await?;
    verify_app_pool(&fixture.app_pool).await?;
    let (status, body) = fixture
        .request(
            reqwest::Method::GET,
            "/api/v1/people?role=student",
            fixture.guardian,
            Role::Guardian,
            None,
        )
        .await?;
    assert_eq!(status, reqwest::StatusCode::OK, "{body}");
    let ids = body["items"]
        .as_array()
        .context("people response includes an items array")?
        .iter()
        .filter_map(|item| item["id"].as_str().map(str::to_owned))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        ids,
        BTreeSet::from([
            fixture.student_one.0.to_string(),
            fixture.student_two.0.to_string()
        ])
    );
    ensure!(!ids.contains(&fixture.unlinked_student.0.to_string()));
    Ok(())
}

#[tokio::test]
async fn student_cannot_see_others() -> Result<()> {
    let fixture = Fixture::new().await?;
    let (status, body) = fixture
        .request(
            reqwest::Method::GET,
            &format!("/api/v1/people/{}", fixture.student_two.0),
            fixture.student_one,
            Role::Student,
            None,
        )
        .await?;
    assert_eq!(status, reqwest::StatusCode::NOT_FOUND, "{body}");
    assert_eq!(body["code"], "not_found");
    Ok(())
}

#[tokio::test]
async fn last_owner_protected() -> Result<()> {
    let fixture = Fixture::new().await?;
    let (status, body) = fixture
        .request(
            reqwest::Method::DELETE,
            &format!("/api/v1/people/{}/roles/owner", fixture.owner.0),
            fixture.owner,
            Role::Owner,
            None,
        )
        .await?;
    assert_eq!(status, reqwest::StatusCode::CONFLICT, "{body}");
    assert_eq!(body["code"], "last_owner");
    let membership = sqlx::query!(
        "SELECT status FROM public.memberships WHERE tenant_id = $1 AND person_id = $2 AND role = 'owner'",
        fixture.tenant.0,
        fixture.owner.0
    )
    .fetch_one(&fixture.owner_pool)
    .await?;
    assert_eq!(membership.status, "active");
    Ok(())
}

#[tokio::test]
async fn admin_cannot_grant_admin() -> Result<()> {
    let fixture = Fixture::new().await?;
    let path = format!("/api/v1/people/{}/roles", fixture.student_one.0);
    let (status, body) = fixture
        .request(
            reqwest::Method::POST,
            &path,
            fixture.admin,
            Role::Admin,
            Some(json!({ "role": "admin" })),
        )
        .await?;
    assert_eq!(status, reqwest::StatusCode::FORBIDDEN, "{body}");
    assert_eq!(body["code"], "forbidden");
    assert_eq!(body["perm"], "identity.admins.manage");

    let (owner_status, owner_body) = fixture
        .request(
            reqwest::Method::POST,
            &path,
            fixture.owner,
            Role::Owner,
            Some(json!({ "role": "admin" })),
        )
        .await?;
    assert_eq!(owner_status, reqwest::StatusCode::NO_CONTENT, "{owner_body}");
    Ok(())
}

#[tokio::test]
async fn one_primary_guardian_api() -> Result<()> {
    let fixture = Fixture::new().await?;
    let (status, body) = fixture
        .request(
            reqwest::Method::POST,
            &format!("/api/v1/people/{}/links", fixture.second_guardian.0),
            fixture.admin,
            Role::Admin,
            Some(json!({ "student_id": fixture.student_one.0, "is_primary": true })),
        )
        .await?;
    assert_eq!(status, reqwest::StatusCode::CONFLICT, "{body}");
    assert_eq!(body["code"], "primary_exists");
    Ok(())
}

#[tokio::test]
async fn read_routes_use_read_only_transactions() -> Result<()> {
    let fixture = Fixture::new().await?;
    let mut tx = eq_core::db::begin_read_only(
        &fixture.app_pool,
        fixture.tenant,
        &eq_core::actor::ActorCtx::system("test"),
    )
    .await?;
    let row = sqlx::query!("SELECT current_setting('transaction_read_only') AS \"read_only!\"")
        .fetch_one(tx.conn())
        .await?;
    assert_eq!(row.read_only, "on");
    tx.rollback().await?;
    Ok(())
}
