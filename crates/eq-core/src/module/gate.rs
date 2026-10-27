//! Tenant-scoped runtime route gate and notification-invalidated enabled-set cache (MOD-05).

use std::collections::BTreeSet;
use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;

use async_trait::async_trait;
use axum::body::Body;
use axum::http::Request;
use axum::response::{IntoResponse, Response};
use sqlx::PgPool;
use sqlx::postgres::PgListener;
use tokio::sync::{Mutex, OnceCell};
use tower::{Layer, Service};
use uuid::Uuid;

use crate::db::begin_system;
use crate::error::AppError;
use crate::ids::TenantId;
use crate::module::{ModuleId, ModuleManifest, repo};
use crate::tenant::TenantCtx;

const MODULE_CACHE_TTL: Duration = Duration::from_secs(30); // technical: MOD-05 cache bound
const NOTIFY_CHANNEL: &str = "eq_modules_changed";
const LISTENER_RECONNECT_DELAY: Duration = Duration::from_secs(1); // technical: retry PostgreSQL LISTEN after disconnect

/// MOD-05: read-only enabled-set lookup used by the route gate and its test seam.
#[async_trait]
pub trait EnabledModuleSet: Send + Sync {
    async fn is_enabled(&self, tenant: TenantId, module: ModuleId) -> Result<bool, AppError>;
}

/// MOD-05: tenant keyed cache of enabled modules, invalidated by PostgreSQL NOTIFY.
#[derive(Clone)]
pub struct EnabledModulesCache {
    pool: PgPool,
    enabled: moka::future::Cache<TenantId, Arc<BTreeSet<ModuleId>>>,
    always_on: Arc<BTreeSet<ModuleId>>,
    listener_started: Arc<OnceCell<()>>,
    cache_update: Arc<Mutex<()>>,
}

impl std::fmt::Debug for EnabledModulesCache {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("EnabledModulesCache")
            .field("pool", &"[configured]")
            .field("always_on", &self.always_on)
            .field("cache", &"[tenant module states]")
            .finish()
    }
}

impl EnabledModulesCache {
    /// MOD-05: constructs a bounded-TTL cache from the compiled module manifests.
    pub fn new(pool: PgPool, manifests: &[&'static ModuleManifest]) -> Self {
        let always_on = manifests
            .iter()
            .filter(|manifest| manifest.always_on)
            .map(|manifest| manifest.id)
            .collect();
        Self {
            pool,
            enabled: moka::future::Cache::builder()
                .time_to_live(MODULE_CACHE_TTL)
                .build(),
            always_on: Arc::new(always_on),
            listener_started: Arc::new(OnceCell::new()),
            cache_update: Arc::new(Mutex::new(())),
        }
    }

    /// MOD-05: starts the tenant-specific PostgreSQL invalidation listener.
    pub async fn start_listener(&self) -> Result<(), sqlx::Error> {
        self.listener_started
            .get_or_try_init(|| async {
                let mut listener = PgListener::connect_with(&self.pool).await?;
                listener.listen(NOTIFY_CHANNEL).await?;
                tokio::spawn(run_listener(
                    listener,
                    self.pool.clone(),
                    self.enabled.clone(),
                    Arc::clone(&self.cache_update),
                ));
                Ok::<(), sqlx::Error>(())
            })
            .await?;
        Ok(())
    }

    async fn enabled_for(&self, tenant: TenantId) -> Result<Arc<BTreeSet<ModuleId>>, AppError> {
        self.start_listener().await?;
        self.load_enabled(tenant, || async {
            let mut tx = begin_system(&self.pool, tenant).await?;
            let modules = repo::enabled(&mut tx, tenant).await?;
            tx.commit().await?;
            Ok(modules)
        })
        .await
    }

    async fn load_enabled<Fut>(
        &self,
        tenant: TenantId,
        load: impl FnOnce() -> Fut,
    ) -> Result<Arc<BTreeSet<ModuleId>>, AppError>
    where
        Fut: Future<Output = Result<BTreeSet<ModuleId>, AppError>>,
    {
        if let Some(enabled) = self.enabled.get(&tenant).await {
            return Ok(enabled);
        }

        // MOD-05: serialize misses with NOTIFY invalidations so an older read cannot be inserted after eviction.
        let _guard = self.cache_update.lock().await;
        if let Some(enabled) = self.enabled.get(&tenant).await {
            return Ok(enabled);
        }

        let enabled = Arc::new(load().await?);
        self.enabled.insert(tenant, Arc::clone(&enabled)).await;
        Ok(enabled)
    }
}

async fn invalidate_cached_tenant(
    cache: &moka::future::Cache<TenantId, Arc<BTreeSet<ModuleId>>>,
    cache_update: &Mutex<()>,
    tenant: TenantId,
) {
    let _guard = cache_update.lock().await;
    cache.invalidate(&tenant).await;
}

async fn run_listener(
    mut listener: PgListener,
    pool: PgPool,
    cache: moka::future::Cache<TenantId, Arc<BTreeSet<ModuleId>>>,
    cache_update: Arc<Mutex<()>>,
) {
    loop {
        match listener.recv().await {
            Ok(notification) => {
                if let Ok(id) = Uuid::parse_str(notification.payload()) {
                    invalidate_cached_tenant(&cache, &cache_update, TenantId(id)).await;
                }
            }
            Err(_) => loop {
                tokio::time::sleep(LISTENER_RECONNECT_DELAY).await;
                let Ok(mut replacement) = PgListener::connect_with(&pool).await else {
                    continue;
                };
                if replacement.listen(NOTIFY_CHANNEL).await.is_err() {
                    continue;
                }
                listener = replacement;
                break;
            },
        }
    }
}

#[async_trait]
impl EnabledModuleSet for EnabledModulesCache {
    async fn is_enabled(&self, tenant: TenantId, module: ModuleId) -> Result<bool, AppError> {
        if self.always_on.contains(&module) {
            return Ok(true);
        }
        Ok(self.enabled_for(tenant).await?.contains(&module))
    }
}

/// MOD-05: Tower middleware that hides routes for disabled modules.
#[derive(Clone)]
pub struct ModuleGate {
    module: ModuleId,
    enabled: Arc<dyn EnabledModuleSet>,
}

impl std::fmt::Debug for ModuleGate {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ModuleGate")
            .field("module", &self.module)
            .finish_non_exhaustive()
    }
}

impl ModuleGate {
    /// MOD-05: creates a route gate for one module id.
    pub fn new(module: ModuleId, enabled: Arc<dyn EnabledModuleSet>) -> Self {
        Self { module, enabled }
    }

    async fn check(&self, tenant: TenantId) -> Result<(), AppError> {
        match self.enabled.is_enabled(tenant, self.module).await {
            Ok(true) => Ok(()),
            Ok(false) => Err(AppError::ModuleDisabled {
                module: self.module.to_string(),
            }),
            Err(error) => Err(error),
        }
    }
}

impl<S> Layer<S> for ModuleGate {
    type Service = ModuleGateService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ModuleGateService {
            inner,
            gate: self.clone(),
        }
    }
}

#[derive(Clone)]
pub struct ModuleGateService<S> {
    inner: S,
    gate: ModuleGate,
}

impl<S> std::fmt::Debug for ModuleGateService<S> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ModuleGateService")
            .field("gate", &self.gate)
            .finish_non_exhaustive()
    }
}

impl<S> Service<Request<Body>> for ModuleGateService<S>
where
    S: Service<Request<Body>, Response = Response<Body>, Error = Infallible> + Clone + Send + Sync + 'static,
    S::Future: Send + 'static,
{
    type Response = Response<Body>;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _context: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: Request<Body>) -> Self::Future {
        let context = request.extensions().get::<TenantCtx>();
        let tenant = match context {
            Some(TenantCtx::Resolved(info)) => Some(info.id),
            Some(TenantCtx::Unprovisioned) | None => None,
        };
        let setup_required = matches!(context, Some(TenantCtx::Unprovisioned));
        let gate = self.gate.clone();
        let mut inner = self.inner.clone();
        Box::pin(async move {
            if setup_required {
                return Ok(AppError::SetupRequired.into_response());
            }
            let Some(tenant) = tenant else {
                return Ok(AppError::NotFound { entity: "tenant" }.into_response());
            };
            if let Err(error) = gate.check(tenant).await {
                return Ok(error.into_response());
            }
            std::future::poll_fn(|context| inner.poll_ready(context)).await?;
            inner.call(request).await
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::sync::Arc;

    use async_trait::async_trait;
    use axum::Router;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use axum::routing::get;
    use sqlx::postgres::PgPoolOptions;
    use tokio::sync::oneshot;
    use tower::ServiceExt;

    use crate::error::AppError;
    use crate::ids::TenantId;
    use crate::module::ModuleId;
    use crate::tenant::{TenantCtx, TenantInfo};

    use super::{EnabledModuleSet, EnabledModulesCache, ModuleGate, invalidate_cached_tenant};

    struct Disabled;

    #[async_trait]
    impl EnabledModuleSet for Disabled {
        async fn is_enabled(&self, _tenant: TenantId, _module: ModuleId) -> Result<bool, AppError> {
            Ok(false)
        }
    }

    #[tokio::test]
    async fn disabled_route_404() {
        let gate = ModuleGate::new(ModuleId::Billing, Arc::new(Disabled));
        let app = Router::new()
            .route("/billing", get(|| async { "should not reach handler" }))
            .route_layer(gate);
        let info = Arc::new(TenantInfo {
            id: TenantId(uuid::Uuid::now_v7()),
            slug: String::from("test-school"),
            status: crate::model::TenantStatus::Active,
            timezone: String::from("UTC"),
            locale: crate::policy::types::Locale::En,
            currency: crate::money::Currency::IDR,
        });
        let mut request = Request::builder()
            .uri("/billing")
            .body(Body::empty())
            .expect("build synthetic request");
        request.extensions_mut().insert(TenantCtx::Resolved(info));

        let response = app.oneshot(request).await.expect("run module gate");
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("read problem body");
        let problem: serde_json::Value = serde_json::from_slice(&bytes).expect("decode problem body");
        assert_eq!(problem["code"], "module_disabled");
        assert_eq!(problem["module"], "billing");
    }

    #[tokio::test]
    async fn cache_miss_racing_invalidation_cannot_reinsert_stale_state() {
        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://eq_app:test-secret-app@127.0.0.1:1/ritma")
            .expect("parse fake database URL");
        let cache = Arc::new(EnabledModulesCache::new(pool, &[]));
        let tenant = TenantId(uuid::Uuid::now_v7());
        let (load_started_tx, load_started_rx) = oneshot::channel();
        let (resume_load_tx, resume_load_rx) = oneshot::channel();

        let loading_cache = Arc::clone(&cache);
        let load = tokio::spawn(async move {
            loading_cache
                .load_enabled(tenant, || async move {
                    load_started_tx.send(()).expect("signal cache load start");
                    resume_load_rx.await.expect("resume cache load");
                    Ok(BTreeSet::new())
                })
                .await
        });
        load_started_rx.await.expect("wait for cache load");

        let (invalidate_started_tx, invalidate_started_rx) = oneshot::channel();
        let invalidating_cache = Arc::clone(&cache);
        let invalidate = tokio::spawn(async move {
            invalidate_started_tx
                .send(())
                .expect("signal cache invalidation start");
            invalidate_cached_tenant(
                &invalidating_cache.enabled,
                &invalidating_cache.cache_update,
                tenant,
            )
            .await;
        });
        invalidate_started_rx.await.expect("wait for cache invalidation");
        tokio::task::yield_now().await;

        resume_load_tx.send(()).expect("resume stale cache load");
        load.await
            .expect("join cache load")
            .expect("load enabled modules");
        invalidate.await.expect("join cache invalidation");

        assert!(cache.enabled.get(&tenant).await.is_none());
        let updated = cache
            .load_enabled(tenant, || async { Ok(BTreeSet::from([ModuleId::Billing])) })
            .await
            .expect("load updated enabled modules");
        assert!(updated.contains(&ModuleId::Billing));
    }
}
