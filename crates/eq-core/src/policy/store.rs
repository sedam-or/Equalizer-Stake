//! Tenant-scoped policy persistence and invalidation (POL-03..08).

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use moka::future::Cache;
use serde_json::Value;
use sqlx::PgPool;
use sqlx::postgres::PgListener;
use tracing::warn;
use uuid::Uuid;

use crate::actor::ActorCtx;
use crate::clock::Clock;
use crate::db::Tx;
use crate::error::AppError;
use crate::ids::TenantId;
use crate::module::{ModuleId, ModuleRegistry};
use crate::policy::repo::{self, NewPolicyRow, PolicyRow};
use crate::policy::{DynPolicy, PolicyDoc, PresetPatchSource, Versioned, merge_patch, validate_document};
use crate::ports::sinks::{AuditSink, EventSink, PolicyInit};

const POLICY_CACHE_CAPACITY: u64 = 1_024; // technical: bound cached tenant/module policy histories in process memory.
const MISSING_POLICY_VERSION: i32 = 0; // technical: no persisted policy version exists before initial setup.

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct CacheKey {
    tenant: TenantId,
    module: ModuleId,
}

/// POL-04: tenant-scoped policy reads, versioned writes and NOTIFY invalidation.
pub struct PolicyStore {
    modules: Arc<ModuleRegistry>,
    preset_patches: Arc<dyn PresetPatchSource>,
    audits: Arc<dyn AuditSink>,
    events: Arc<dyn EventSink>,
    clock: Arc<dyn Clock>,
    cache: Cache<CacheKey, Vec<PolicyRow>>,
}

impl std::fmt::Debug for PolicyStore {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PolicyStore")
            .field("pool", &"[tenant-scoped PostgreSQL pool]")
            .field("modules", &self.modules)
            .field("cache", &"[tenant/module policy cache]")
            .finish()
    }
}

impl PolicyStore {
    /// POL-04: connects the cache invalidation listener before returning a usable store.
    pub async fn new(
        pool: PgPool,
        modules: Arc<ModuleRegistry>,
        preset_patches: Arc<dyn PresetPatchSource>,
        audits: Arc<dyn AuditSink>,
        events: Arc<dyn EventSink>,
        clock: Arc<dyn Clock>,
    ) -> Result<Self, AppError> {
        let cache = Cache::builder().max_capacity(POLICY_CACHE_CAPACITY).build();
        let mut listener = PgListener::connect_with(&pool).await?;
        listener.listen("eq_policy_changed").await?;
        let listener_cache = cache.clone();
        tokio::spawn(async move {
            while let Ok(notification) = listener.recv().await {
                let mut parts = notification.payload().split(':');
                let Some(tenant) = parts.next().and_then(|value| Uuid::parse_str(value).ok()) else {
                    continue;
                };
                let Some(module) = parts.next().and_then(ModuleId::parse) else {
                    continue;
                };
                if parts.next().is_none() {
                    listener_cache
                        .invalidate(&CacheKey {
                            tenant: TenantId(tenant),
                            module,
                        })
                        .await;
                }
            }
        });
        Ok(Self {
            modules,
            preset_patches,
            audits,
            events,
            clock,
            cache,
        })
    }

    /// POL-05: returns the highest version effective at the requested instant.
    pub async fn get<P: PolicyDoc>(
        &self,
        tx: &mut Tx,
        tenant: TenantId,
        at: DateTime<Utc>,
    ) -> Result<Versioned<P>, AppError> {
        if !repo::tenant_context_matches(tx, tenant).await? {
            return Err(AppError::Internal(anyhow::anyhow!(
                "policy tenant does not match transaction context"
            )));
        }
        let rows = self.history(tx, tenant, P::MODULE).await?;
        let row = rows
            .into_iter()
            .filter(|row| row.effective_from <= at)
            .max_by(|left, right| {
                left.effective_from
                    .cmp(&right.effective_from)
                    .then_with(|| left.version.cmp(&right.version))
            });

        if let Some(row) = row {
            let document = self.migrate_document(P::MODULE, row.schema_version, row.document)?;
            return Ok(Versioned {
                version: row.version,
                effective_from: row.effective_from,
                doc: validate_document::<P>(document)?,
            });
        }

        warn!(tenant_id = %tenant.0, module = P::MODULE.as_str(), "policy row missing");
        let mut document =
            serde_json::to_value(P::default()).map_err(|error| AppError::Internal(error.into()))?;
        if let Some(patch) = self.tenant_patch(tx, tenant, P::MODULE).await? {
            document = merge_patch(document, &patch);
        }
        Ok(Versioned {
            version: MISSING_POLICY_VERSION,
            effective_from: at,
            doc: validate_document::<P>(document)?,
        })
    }

    /// POL-05: returns the policy in force at the injected clock's current instant.
    pub async fn current<P: PolicyDoc>(
        &self,
        tx: &mut Tx,
        tenant: TenantId,
    ) -> Result<Versioned<P>, AppError> {
        self.get::<P>(tx, tenant, self.clock.now()).await
    }

    /// POL-03: appends a validated policy version with audit, event and NOTIFY in the caller transaction.
    pub async fn put<P: PolicyDoc>(
        &self,
        tx: &mut Tx,
        tenant: TenantId,
        doc: P,
        effective_from: DateTime<Utc>,
        actor: &ActorCtx,
        note: Option<String>,
    ) -> Result<Versioned<P>, AppError> {
        if !repo::tenant_context_matches(tx, tenant).await? {
            return Err(AppError::Internal(anyhow::anyhow!(
                "policy tenant does not match transaction context"
            )));
        }
        let document = serde_json::to_value(&doc).map_err(|error| AppError::Internal(error.into()))?;
        let doc = validate_document::<P>(document.clone())?;
        let module = P::MODULE;
        let version = repo::next_version(tx, tenant, module).await?;
        let previous = repo::previous_document(tx, tenant, module).await?;
        let schema_version =
            i32::try_from(P::SCHEMA_VERSION).map_err(|error| AppError::Internal(error.into()))?;
        repo::insert(
            tx,
            NewPolicyRow {
                tenant,
                module,
                version,
                schema_version,
                effective_from,
                document,
                note,
                created_by: actor.person_id.map(|person_id| person_id.0),
            },
        )
        .await?;

        self.audits
            .record_policy_change(
                tx,
                tenant,
                actor,
                crate::ports::sinks::PolicyChange {
                    module,
                    version,
                    previous,
                    document: doc_value(&doc)?,
                },
            )
            .await?;
        self.events
            .emit_policy_changed(tx, tenant, actor, module, version, effective_from)
            .await?;
        repo::notify_changed(tx, tenant, module).await?;
        self.cache.invalidate(&CacheKey { tenant, module }).await;

        Ok(Versioned {
            version,
            effective_from,
            doc,
        })
    }

    async fn history(
        &self,
        tx: &mut Tx,
        tenant: TenantId,
        module: ModuleId,
    ) -> Result<Vec<PolicyRow>, AppError> {
        let key = CacheKey { tenant, module };
        let transaction_has_writes = repo::transaction_has_writes(tx).await?;
        if !transaction_has_writes && let Some(rows) = self.cache.get(&key).await {
            return Ok(rows);
        }
        let rows = repo::history(tx, tenant, module).await?;
        if !transaction_has_writes {
            self.cache.insert(key, rows.clone()).await;
        }
        Ok(rows)
    }

    async fn tenant_patch(
        &self,
        tx: &mut Tx,
        tenant: TenantId,
        module: ModuleId,
    ) -> Result<Option<Value>, AppError> {
        let identity = repo::tenant_preset(tx, tenant).await?;
        self.preset_patches
            .policy_patch(&identity.preset_id, identity.preset_version, module)
            .map_err(|error| AppError::Internal(anyhow::Error::new(error)))
    }

    fn migrate_document(
        &self,
        module: ModuleId,
        from_version: i32,
        document: Value,
    ) -> Result<Value, AppError> {
        let implementation = self
            .modules
            .get(module)
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("policy module is not compiled")))?;
        let current_version = i32::try_from(implementation.manifest().policy_schema_version)
            .map_err(|error| AppError::Internal(error.into()))?;
        if from_version < current_version {
            let from_version =
                u32::try_from(from_version).map_err(|error| AppError::Internal(error.into()))?;
            implementation
                .migrate_policy(from_version, document)
                .map_err(AppError::Internal)
        } else {
            Ok(document)
        }
    }
}

#[async_trait]
impl PolicyInit for PolicyStore {
    async fn ensure_initial(&self, tx: &mut Tx, tenant: TenantId, module: ModuleId) -> Result<(), AppError> {
        if !repo::tenant_context_matches(tx, tenant).await? {
            return Err(AppError::Internal(anyhow::anyhow!(
                "policy tenant does not match transaction context"
            )));
        }
        if repo::exists(tx, tenant, module).await? {
            return Ok(());
        }
        if repo::next_version(tx, tenant, module).await? != 1 {
            return Ok(());
        }

        let implementation = self
            .modules
            .get(module)
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("policy module is not compiled")))?;
        let mut document = implementation.default_policy();
        if let Some(patch) = self.tenant_patch(tx, tenant, module).await? {
            document = merge_patch(document, &patch);
        }
        let errors = DynPolicy::new(implementation.as_ref())
            .validate(&document)
            .err()
            .unwrap_or_default();
        if !errors.is_empty() {
            return Err(AppError::Validation { errors });
        }

        let effective_from = self.clock.now();
        let schema_version = i32::try_from(implementation.manifest().policy_schema_version)
            .map_err(|error| AppError::Internal(error.into()))?;
        repo::insert(
            tx,
            NewPolicyRow {
                tenant,
                module,
                version: 1,
                schema_version,
                effective_from,
                document,
                note: None,
                created_by: None,
            },
        )
        .await?;
        repo::notify_changed(tx, tenant, module).await?;
        self.cache.invalidate(&CacheKey { tenant, module }).await;
        Ok(())
    }
}

fn doc_value<P: PolicyDoc>(document: &P) -> Result<Value, AppError> {
    serde_json::to_value(document).map_err(|error| AppError::Internal(error.into()))
}
