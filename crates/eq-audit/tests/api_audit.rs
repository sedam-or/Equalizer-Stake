use anyhow::{Context, Result, ensure};
use async_trait::async_trait;
use axum::Extension;
use eq_audit::policy::{AuditPolicy, Category};
use eq_audit::{AuditEvent, AuditModule, AuditPolicyProvider, Sealer, record};
use eq_core::actor::{ActorCtx, ActorRole};
use eq_core::app_state::AppState;
use eq_core::clock::SystemClock;
use eq_core::config::Config;
use eq_core::db::{Tx, begin};
use eq_core::ids::{PersonId, TenantId};
use eq_core::model::{Role, TenantStatus};
use eq_core::module::Module;
use eq_core::tenant::{TenantCtx, TenantInfo};
use eq_core::{money::Currency, policy::types::Locale};
use reqwest::{Client, Method, StatusCode};
use secrecy::ExposeSecret;
use serde_json::Value;
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};
use std::collections::BTreeSet;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::task::JoinHandle;
use uuid::Uuid;

struct Fixture {
    app_pool: PgPool,
    owner_pool: PgPool,
    config: Arc<Config>,
    tenant: TenantId,
    owner: PersonId,
    admin: PersonId,
}

impl Fixture {
    async fn new() -> Result<Self> {
        let config = Arc::new(Config::from_env().context("load T-113b audit test config")?);
        ensure!(
            config
                .database_url
                .expose_secret()
                .ends_with("/ritma_test_T-113b"),
            "audit tests must use ritma_test_T-113b"
        );
        let app_pool = PgPoolOptions::new()
            .max_connections(4)
            .connect(config.database_url.expose_secret())
            .await
            .context("connect to the T-113b application database")?;
        let owner_pool = PgPoolOptions::new()
            .max_connections(2)
            .connect(config.database_owner_url.expose_secret())
            .await
            .context("connect to the T-113b database owner")?;
        let role = sqlx::query(
            "SELECT current_database() AS database, current_user AS username, rolsuper, rolbypassrls \
             FROM pg_roles WHERE rolname = current_user",
        )
        .fetch_one(&app_pool)
        .await?;
        ensure!(role.try_get::<String, _>("database")? == "ritma_test_T-113b");
        ensure!(role.try_get::<String, _>("username")? == "eq_app");
        ensure!(!role.try_get::<bool, _>("rolsuper")? && !role.try_get::<bool, _>("rolbypassrls")?);

        let tenant = TenantId(Uuid::now_v7());
        let owner = PersonId(Uuid::now_v7());
        let admin = PersonId(Uuid::now_v7());
        sqlx::query(
            "INSERT INTO public.tenants (id, slug, name, country, preset_id, preset_version) \
             VALUES ($1, $2, 'Audit Test School', 'ID', 'test', 1)",
        )
        .bind(tenant.0)
        .bind(format!("audit-{}", tenant.0.simple()))
        .execute(&owner_pool)
        .await?;
        for (person, role, name) in [(owner, "owner", "Audit Owner"), (admin, "admin", "Audit Admin")] {
            sqlx::query("INSERT INTO public.persons (id, tenant_id, display_name) VALUES ($1, $2, $3)")
                .bind(person.0)
                .bind(tenant.0)
                .bind(name)
                .execute(&owner_pool)
                .await?;
            sqlx::query(
                "INSERT INTO public.memberships (tenant_id, person_id, role, status) \
                 VALUES ($1, $2, $3, 'active')",
            )
            .bind(tenant.0)
            .bind(person.0)
            .bind(role)
            .execute(&owner_pool)
            .await?;
        }
        Ok(Self {
            app_pool,
            owner_pool,
            config,
            tenant,
            owner,
            admin,
        })
    }

    fn actor(&self, role: Role) -> (ActorCtx, eq_core::auth::Actor) {
        let (person, actor_role, permissions) = match role {
            Role::Owner => (
                self.owner,
                ActorRole::Owner,
                BTreeSet::from([
                    String::from("audit.read"),
                    String::from("audit.verify"),
                    String::from("identity.people.read_pii"),
                ]),
            ),
            Role::Admin => (
                self.admin,
                ActorRole::Admin,
                BTreeSet::from([String::from("audit.read")]),
            ),
            _ => unreachable!("only owner and admin actors are used by these tests"),
        };
        let context = ActorCtx {
            person_id: Some(person),
            role: actor_role,
            ip: Some(IpAddr::V4(Ipv4Addr::LOCALHOST)),
            request_id: Some(Uuid::now_v7()),
        };
        let actor = eq_core::auth::Actor::from_verified_token(
            person,
            self.tenant,
            role,
            Uuid::now_v7(),
            permissions,
            context.clone(),
        );
        (context, actor)
    }

    async fn add_entry(&self, role: Role, category: Category, action: &'static str) -> Result<Uuid> {
        let (actor, _) = self.actor(role);
        let entity_id = Uuid::now_v7();
        let mut tx = begin(&self.app_pool, self.tenant, &actor).await?;
        record(
            &mut tx,
            AuditEvent {
                action,
                category,
                entity_type: "test",
                entity_id: Some(entity_id),
                before: None,
                after: Some(serde_json::json!({"result": "ok"})),
                actor: &actor,
            },
        )
        .await?;
        tx.commit().await?;
        Ok(sqlx::query_scalar::<_, Uuid>(
            "SELECT id FROM audit.entries WHERE tenant_id = $1 AND entity_id = $2 ORDER BY seq DESC LIMIT 1",
        )
        .bind(self.tenant.0)
        .bind(entity_id)
        .fetch_one(&self.owner_pool)
        .await?)
    }

    async fn sequence_bounds(&self) -> Result<(i64, i64)> {
        let (first, last): (Option<i64>, Option<i64>) =
            sqlx::query_as("SELECT min(seq), max(seq) FROM audit.entries WHERE tenant_id = $1")
                .bind(self.tenant.0)
                .fetch_one(&self.owner_pool)
                .await?;
        Ok((
            first.context("tenant audit sequence has a first row")?,
            last.context("tenant audit sequence has a last row")?,
        ))
    }

    async fn request(
        &self,
        role: Role,
        method: Method,
        path: &str,
    ) -> Result<(StatusCode, reqwest::header::HeaderMap, String)> {
        let state = AppState::new(
            self.app_pool.clone(),
            Arc::clone(&self.config),
            Arc::new(SystemClock),
        );
        let (actor_context, actor) = self.actor(role);
        let tenant_context = TenantCtx::Resolved(Arc::new(TenantInfo {
            id: self.tenant,
            slug: format!("audit-{}", self.tenant.0.simple()),
            status: TenantStatus::Active,
            timezone: String::from("UTC"),
            locale: Locale::En,
            currency: Currency::USD,
        }));
        let policy: Arc<dyn AuditPolicyProvider> = Arc::new(DefaultAuditPolicy);
        let (routes, _openapi) = AuditModule.routes().layer(Extension(policy)).split_for_parts();
        let router = routes
            .with_state(state)
            .layer(Extension(tenant_context))
            .layer(Extension(actor_context))
            .layer(Extension(actor));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let address: SocketAddr = listener.local_addr()?;
        let server: JoinHandle<std::io::Result<()>> =
            tokio::spawn(async move { axum::serve(listener, router).await });
        let response = Client::new()
            .request(method, format!("http://{address}{path}"))
            .send()
            .await?;
        let status = response.status();
        let headers = response.headers().clone();
        let body = response.text().await?;
        server.abort();
        Ok((status, headers, body))
    }
}

#[derive(Debug)]
struct DefaultAuditPolicy;

#[async_trait]
impl AuditPolicyProvider for DefaultAuditPolicy {
    async fn admin_categories(
        &self,
        _tx: &mut Tx,
        _tenant: TenantId,
    ) -> Result<Vec<Category>, eq_core::error::AppError> {
        Ok(AuditPolicy::default().admin_categories().to_vec())
    }
}

#[tokio::test]
async fn admin_category_scope() -> Result<()> {
    let fixture = Fixture::new().await?;
    let operational = fixture
        .add_entry(Role::Owner, Category::Operational, "people.read")
        .await?;
    let security = fixture
        .add_entry(Role::Owner, Category::Security, "identity.role.changed")
        .await?;
    let (admin_status, _, admin_body) = fixture
        .request(Role::Admin, Method::GET, "/api/v1/audit/entries")
        .await?;
    assert_eq!(admin_status, StatusCode::OK, "{admin_body}");
    let admin_json: Value = serde_json::from_str(&admin_body)?;
    let admin_entries = admin_json["entries"].as_array().context("admin entries")?;
    assert!(
        admin_entries
            .iter()
            .any(|entry| entry["id"] == operational.to_string())
    );
    assert!(
        !admin_entries
            .iter()
            .any(|entry| entry["id"] == security.to_string())
    );

    let (hidden_status, _, hidden_body) = fixture
        .request(
            Role::Admin,
            Method::GET,
            &format!("/api/v1/audit/entries/{security}"),
        )
        .await?;
    assert_eq!(hidden_status, StatusCode::NOT_FOUND, "{hidden_body}");

    let (owner_status, _, owner_body) = fixture
        .request(Role::Owner, Method::GET, "/api/v1/audit/entries")
        .await?;
    assert_eq!(owner_status, StatusCode::OK, "{owner_body}");
    let owner_json: Value = serde_json::from_str(&owner_body)?;
    let owner_entries = owner_json["entries"].as_array().context("owner entries")?;
    assert!(
        owner_entries
            .iter()
            .any(|entry| entry["id"] == security.to_string())
    );
    Ok(())
}

#[tokio::test]
async fn export_is_audited() -> Result<()> {
    let fixture = Fixture::new().await?;
    fixture
        .add_entry(Role::Owner, Category::Operational, "=1+1")
        .await?;
    let (status, headers, body) = fixture
        .request(
            Role::Owner,
            Method::GET,
            "/api/v1/audit/export?category=operational",
        )
        .await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        headers
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()),
        Some("text/csv; charset=utf-8")
    );
    assert!(body.starts_with("id,seq,at,actor_id,actor_role,action"));
    let mut csv_reader = csv::Reader::from_reader(body.as_bytes());
    let actions = csv_reader
        .records()
        .map(|row| row.map(|record| record.get(5).map(str::to_owned)))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    assert!(actions.iter().flatten().any(|action| action == "'=1+1"));
    let audited = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM audit.entries WHERE tenant_id = $1 AND action = 'audit.export'",
    )
    .bind(fixture.tenant.0)
    .fetch_one(&fixture.owner_pool)
    .await?;
    assert_eq!(audited, 1);
    assert_eq!(eq_core::export::guard_cell("=1+1"), "'=1+1");
    Ok(())
}

#[tokio::test]
async fn verify_owner_only() -> Result<()> {
    let fixture = Fixture::new().await?;
    fixture
        .add_entry(Role::Owner, Category::Operational, "people.read")
        .await?;
    fixture
        .add_entry(Role::Owner, Category::Security, "identity.role.changed")
        .await?;
    Sealer::new(fixture.app_pool.clone())
        .seal_tenant(fixture.tenant)
        .await?;
    let (from_seq, to_seq) = fixture.sequence_bounds().await?;
    let (admin_status, _, admin_body) = fixture
        .request(
            Role::Admin,
            Method::GET,
            &format!("/api/v1/audit/verify?from_seq={from_seq}&to_seq={to_seq}"),
        )
        .await?;
    assert_eq!(admin_status, StatusCode::FORBIDDEN, "{admin_body}");

    let (owner_status, _, owner_body) = fixture
        .request(
            Role::Owner,
            Method::GET,
            &format!("/api/v1/audit/verify?from_seq={from_seq}&to_seq={to_seq}"),
        )
        .await?;
    assert_eq!(owner_status, StatusCode::OK, "{owner_body}");
    let report: Value = serde_json::from_str(&owner_body)?;
    assert_eq!(report["checked"], 2);
    assert_eq!(report["first_mismatch"], Value::Null);
    Ok(())
}
