use std::collections::BTreeSet;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use axum::routing::get;
use eq_core::actor::ActorCtx;
use eq_core::app_state::AppState;
use eq_core::db;
use eq_core::error::FieldError;
use eq_core::ids::TenantId;
use eq_core::jobs::{CronSpec, JobHandler};
use eq_core::model::TenantStatus;
use eq_core::module::gate::{EnabledModuleSet, EnabledModulesCache, ModuleGate};
use eq_core::module::service::ModuleService;
use eq_core::module::{
    Module, ModuleId, ModuleManifest, ModuleRegistry, NavItem, PermissionDef, SummaryLine,
};
use eq_core::outbox::EventHandler;
use eq_core::ports::PortRegistry;
use eq_core::tenant::TenantInfo;
use serde_json::{Value, json};
use tower::ServiceExt;
use utoipa_axum::router::OpenApiRouter;
use uuid::Uuid;

#[expect(
    dead_code,
    reason = "shared test database fixture includes credential URLs used by other integration suites"
)]
#[path = "../../eq-testkit/src/db.rs"]
#[expect(
    clippy::panic,
    reason = "the shared database fixture reports only redacted setup failures"
)]
mod shared_test_db;
use shared_test_db::TestDb;

#[path = "../../eq-testkit/src/module_sinks.rs"]
mod shared_module_sinks;
use shared_module_sinks::{ModuleSinkDoubles, ModuleToggleRecord};

static MANIFESTS: [ModuleManifest; 11] = [
    manifest(ModuleId::Core, true, &[], &[]),
    manifest(ModuleId::Identity, true, &[], &[]),
    manifest(ModuleId::Audit, true, &[], &[]),
    manifest(ModuleId::Scheduling, true, &[], &[]),
    manifest(ModuleId::Enrollment, false, &[ModuleId::Scheduling], &[]),
    manifest(
        ModuleId::Billing,
        false,
        &[],
        &[ModuleId::Enrollment, ModuleId::Rental],
    ),
    manifest(ModuleId::Attendance, false, &[ModuleId::Scheduling], &[]),
    manifest(ModuleId::Rental, false, &[ModuleId::Scheduling], &[]),
    manifest(ModuleId::Notify, false, &[], &[]),
    manifest(ModuleId::Reports, false, &[], &[]),
    manifest(ModuleId::Feedback, false, &[ModuleId::Enrollment], &[]),
];

const fn manifest(
    id: ModuleId,
    always_on: bool,
    requires_all: &'static [ModuleId],
    requires_any: &'static [ModuleId],
) -> ModuleManifest {
    ModuleManifest {
        id,
        name_key: "module.test.name",
        description_key: "module.test.description",
        icon: "settings",
        always_on,
        requires_all,
        requires_any,
        policy_schema_version: 1,
    }
}

struct TestModule(&'static ModuleManifest);

#[async_trait]
impl Module for TestModule {
    fn manifest(&self) -> &'static ModuleManifest {
        self.0
    }
    fn policy_schema(&self) -> Value {
        json!({})
    }
    fn default_policy(&self) -> Value {
        json!({})
    }
    fn validate_policy(&self, _document: &Value) -> Result<(), Vec<FieldError>> {
        Ok(())
    }
    fn summarize_policy(&self, _document: &Value) -> Vec<SummaryLine> {
        Vec::new()
    }
    fn migrate_policy(&self, _from_version: u32, document: Value) -> anyhow::Result<Value> {
        Ok(document)
    }
    fn routes(&self) -> OpenApiRouter<AppState> {
        OpenApiRouter::new()
    }
    fn event_handlers(&self) -> Vec<Box<dyn EventHandler>> {
        Vec::new()
    }
    fn jobs(&self) -> Vec<Box<dyn JobHandler>> {
        Vec::new()
    }
    fn cron(&self) -> Vec<CronSpec> {
        Vec::new()
    }
    fn register_ports(&self, _ports: &mut PortRegistry) {}
    fn permissions(&self) -> &'static [PermissionDef] {
        &[]
    }
    fn nav(&self) -> &'static [NavItem] {
        &[]
    }
}

fn registry() -> Arc<ModuleRegistry> {
    let modules = MANIFESTS
        .iter()
        .map(|manifest| Arc::new(TestModule(manifest)) as Arc<dyn Module>)
        .collect();
    Arc::new(ModuleRegistry::new(modules))
}

#[expect(
    clippy::expect_used,
    reason = "fixture setup fails clearly at the failed database operation"
)]
async fn tenant_fixture() -> (TestDb, Arc<TenantInfo>) {
    let db = TestDb::new().await;
    let tenant = TenantId(Uuid::now_v7());
    sqlx::query(
        "INSERT INTO tenants (id, slug, name, country, preset_id, preset_version) \
         VALUES ($1, $2, 'Module Test School', 'ID', 'test', 1)",
    )
    .bind(tenant.0)
    .bind(format!("module-{}", tenant.0.simple()))
    .execute(&db.pool)
    .await
    .expect("insert synthetic tenant");
    let info = Arc::new(TenantInfo {
        id: tenant,
        slug: String::from("module-test"),
        status: TenantStatus::Active,
        timezone: String::from("UTC"),
        locale: eq_core::policy::types::Locale::En,
        currency: eq_core::money::Currency::IDR,
    });
    (db, info)
}

struct DisabledModule;

#[async_trait]
impl EnabledModuleSet for DisabledModule {
    async fn is_enabled(
        &self,
        _tenant: TenantId,
        _module: ModuleId,
    ) -> Result<bool, eq_core::error::AppError> {
        Ok(false)
    }
}

fn module_service(registry: Arc<ModuleRegistry>, sinks: ModuleSinkDoubles) -> ModuleService {
    ModuleService::new(
        registry,
        sinks.policy_init(),
        sinks.audit_sink(),
        sinks.event_sink(),
    )
}

#[tokio::test]
async fn disabled_route_404() {
    let (_db, tenant) = tenant_fixture().await;
    let gate = ModuleGate::new(ModuleId::Billing, Arc::new(DisabledModule));
    let app = Router::new()
        .route("/billing", get(|| async { "should not reach handler" }))
        .route_layer(gate);
    let mut request = Request::builder()
        .uri("/billing")
        .body(Body::empty())
        .expect("build synthetic request");
    request
        .extensions_mut()
        .insert(eq_core::tenant::TenantCtx::Resolved(tenant));

    let response = app.oneshot(request).await.expect("run module gate");
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read problem body");
    let problem: Value = serde_json::from_slice(&bytes).expect("decode problem body");
    assert_eq!(problem["code"], "module_disabled");
    assert_eq!(problem["module"], "billing");
}

#[tokio::test]
async fn tenant_modules_are_isolated_under_rls() {
    let (db, tenant) = tenant_fixture().await;
    let other_tenant = TenantId(Uuid::now_v7());
    sqlx::query(
        "INSERT INTO tenants (id, slug, name, country, preset_id, preset_version) \
         VALUES ($1, $2, 'Module Test School', 'ID', 'test', 1)",
    )
    .bind(other_tenant.0)
    .bind(format!("module-{}", other_tenant.0.simple()))
    .execute(&db.pool)
    .await
    .expect("insert second synthetic tenant");
    sqlx::query(
        "INSERT INTO tenant_modules (tenant_id, module_id, enabled) \
         VALUES ($1, 'notify', TRUE)",
    )
    .bind(other_tenant.0)
    .execute(&db.pool)
    .await
    .expect("enable notify for second tenant");

    let mut tx = db::begin_system(&db.app_pool, tenant.id)
        .await
        .expect("begin app-role isolation transaction");
    let other_rows =
        sqlx::query_scalar::<_, String>("SELECT module_id FROM tenant_modules WHERE tenant_id = $1")
            .bind(other_tenant.0)
            .fetch_all(tx.conn())
            .await
            .expect("query other tenant modules under RLS");
    tx.commit().await.expect("commit isolation transaction");
    assert!(other_rows.is_empty());

    let cache = EnabledModulesCache::new(db.app_pool.clone(), &registry().manifests());
    assert!(
        !cache
            .is_enabled(tenant.id, ModuleId::Notify)
            .await
            .expect("load first tenant module state")
    );
    assert!(
        cache
            .is_enabled(other_tenant, ModuleId::Notify)
            .await
            .expect("load second tenant module state")
    );
}

#[tokio::test]
async fn gate_invalidated_by_notify() {
    let (db, tenant) = tenant_fixture().await;
    let registry = registry();
    let manifests = registry.manifests();
    let cache = Arc::new(EnabledModulesCache::new(db.app_pool.clone(), &manifests));
    cache
        .start_listener()
        .await
        .expect("start module notification listener");
    assert!(
        !cache
            .is_enabled(tenant.id, ModuleId::Billing)
            .await
            .expect("warm disabled cache")
    );

    let sinks = ModuleSinkDoubles::default();
    let service = module_service(registry, sinks);
    let actor = ActorCtx::system("test");
    let mut tx = db::begin_system(&db.app_pool, tenant.id)
        .await
        .expect("begin enablement transaction");
    service
        .set_enabled(
            &mut tx,
            tenant.id,
            &actor,
            &BTreeSet::from([ModuleId::Enrollment, ModuleId::Billing]),
        )
        .await
        .expect("enable modules");
    tx.commit().await.expect("commit module enablement");

    tokio::time::timeout(Duration::from_secs(1), async {
        loop {
            if cache
                .is_enabled(tenant.id, ModuleId::Billing)
                .await
                .expect("read module gate")
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("notification invalidates the tenant cache within one second");
}

#[tokio::test]
async fn toggle_audits_and_emits() {
    let (db, tenant) = tenant_fixture().await;
    let registry = registry();
    let sinks = ModuleSinkDoubles::default();
    let service = module_service(registry, sinks.clone());
    let actor = ActorCtx::system("test");
    let mut tx = db::begin_system(&db.app_pool, tenant.id)
        .await
        .expect("begin enablement transaction");
    service
        .set_enabled(
            &mut tx,
            tenant.id,
            &actor,
            &BTreeSet::from([ModuleId::Enrollment, ModuleId::Billing]),
        )
        .await
        .expect("enable modules");
    tx.commit().await.expect("commit module enablement");
    let records = sinks.records().expect("read module sink records");
    let expected = vec![
        ModuleToggleRecord {
            tenant: tenant.id,
            actor: None,
            module: ModuleId::Enrollment,
            enabled: true,
        },
        ModuleToggleRecord {
            tenant: tenant.id,
            actor: None,
            module: ModuleId::Billing,
            enabled: true,
        },
    ];
    assert_eq!(
        records.policies_initialized,
        vec![(tenant.id, ModuleId::Enrollment), (tenant.id, ModuleId::Billing)]
    );
    assert_eq!(records.audits, expected);
    assert_eq!(records.events, expected);
}
