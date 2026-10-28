//! Event handler contracts consumed by compiled modules (ARC-07, MOD-04).

use crate::actor::{ActorCtx, ActorRole};
use crate::clock::{Clock, SystemClock};
use crate::db::Tx;
use crate::error::AppError;
use crate::ids::{PersonId, TenantId};
use crate::module::ModuleId;
use crate::module::gate::EnabledModuleSet;
use crate::ports::sinks::EventSink;
use anyhow::Context;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{PgPool, Postgres, Transaction};
use std::collections::{BTreeMap, VecDeque};
use std::sync::Arc;
use tokio::task::JoinSet;
use uuid::Uuid;

const CLAIM_BATCH_SIZE: i64 = 100; // technical: FLW-03 bounds one drain query
const MAX_CONCURRENT_GROUPS: usize = 8; // technical: T-107 bounds aggregate concurrency

/// CAN-60: actor fields included in an event envelope.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EventActor {
    pub id: Option<PersonId>,
    pub role: ActorRole,
}

/// CAN-60: event envelope delivered to registered module handlers.
#[derive(Clone)]
pub struct Event {
    pub id: Uuid,
    pub event_type: &'static str,
    pub tenant_id: TenantId,
    pub aggregate_id: Option<Uuid>,
    pub occurred_at: DateTime<Utc>,
    pub actor: EventActor,
    pub request_id: Option<Uuid>,
    pub payload: Value,
}

impl std::fmt::Debug for Event {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Event")
            .field("id", &self.id)
            .field("event_type", &self.event_type)
            .field("tenant_id", &self.tenant_id)
            .field("aggregate_id", &self.aggregate_id)
            .field("occurred_at", &self.occurred_at)
            .field("actor", &self.actor)
            .field("request_id", &self.request_id)
            .field("payload", &"[REDACTED]")
            .finish()
    }
}

/// CAN-60: serializable payload for a versioned domain event.
pub trait DomainEvent: Serialize {
    const TYPE: &'static str;

    fn aggregate_id(&self) -> Option<Uuid>;
}

#[derive(Serialize)]
struct StoredEvent<'a, E> {
    actor: EventActor,
    request_id: Option<Uuid>,
    payload: &'a E,
}

/// ARC-07: appends an event to the same transaction as its domain change.
pub async fn emit<E: DomainEvent>(tx: &mut Tx, tenant: TenantId, event: &E) -> Result<(), AppError> {
    if tx.tenant() != tenant {
        return Err(AppError::Internal(anyhow::anyhow!(
            "outbox tenant does not match transaction context"
        )));
    }

    let actor = tx.actor();
    let stored = StoredEvent {
        actor: EventActor {
            id: actor.person_id,
            role: actor.role,
        },
        request_id: actor.request_id,
        payload: event,
    };
    let payload = serde_json::to_value(stored)
        .map_err(|_| AppError::Internal(anyhow::anyhow!("serialize outbox event envelope")))?;
    let id = Uuid::now_v7();
    let occurred_at = SystemClock.now();

    sqlx::query!(
        "INSERT INTO public.outbox (id, tenant_id, event_type, aggregate_id, payload, occurred_at) VALUES ($1, $2, $3, $4, $5, $6)",
        id,
        tenant.0,
        E::TYPE,
        event.aggregate_id(),
        payload,
        occurred_at
    )
    .execute(tx.conn())
    .await?;

    Ok(())
}

/// MOD-06: event sink backed by the transaction-bound outbox producer.
#[derive(Debug, Default)]
pub struct OutboxEventSink;

#[derive(Deserialize, JsonSchema, Serialize)]
struct ModuleToggledEvent {
    module: ModuleId,
    enabled: bool,
}

impl DomainEvent for ModuleToggledEvent {
    const TYPE: &'static str = "core.module_toggled.v1";

    fn aggregate_id(&self) -> Option<Uuid> {
        None
    }
}

#[derive(Deserialize, JsonSchema, Serialize)]
struct PolicyChangedEvent {
    module: ModuleId,
    version: i32,
    effective_from: DateTime<Utc>,
}

impl DomainEvent for PolicyChangedEvent {
    const TYPE: &'static str = "core.policy_changed.v1";

    fn aggregate_id(&self) -> Option<Uuid> {
        None
    }
}

fn ensure_event_context(tx: &Tx, tenant: TenantId, actor: &ActorCtx) -> Result<(), AppError> {
    if tx.tenant() != tenant {
        return Err(AppError::Internal(anyhow::anyhow!(
            "outbox tenant does not match transaction context"
        )));
    }
    if tx.actor() != actor {
        return Err(AppError::Internal(anyhow::anyhow!(
            "outbox actor does not match transaction context"
        )));
    }
    Ok(())
}

#[async_trait]
impl EventSink for OutboxEventSink {
    async fn emit_module_toggled(
        &self,
        tx: &mut Tx,
        tenant: TenantId,
        actor: &ActorCtx,
        module: ModuleId,
        enabled: bool,
    ) -> Result<(), AppError> {
        ensure_event_context(tx, tenant, actor)?;
        emit(tx, tenant, &ModuleToggledEvent { module, enabled }).await
    }

    async fn emit_policy_changed(
        &self,
        tx: &mut Tx,
        tenant: TenantId,
        actor: &ActorCtx,
        module: ModuleId,
        version: i32,
        effective_from: DateTime<Utc>,
    ) -> Result<(), AppError> {
        ensure_event_context(tx, tenant, actor)?;
        emit(
            tx,
            tenant,
            &PolicyChangedEvent {
                module,
                version,
                effective_from,
            },
        )
        .await
    }
}

/// MOD-04: one idempotent consumer registered by a module.
#[async_trait]
pub trait EventHandler: Send + Sync {
    fn id(&self) -> &'static str;
    fn module(&self) -> ModuleId;
    fn subscribes(&self) -> &'static [&'static str];
    async fn handle(&self, tx: &mut Tx, event: &Event) -> anyhow::Result<()>;
}

/// FLW-03: bounded delivery of outbox events to tenant-enabled module handlers.
pub struct Dispatcher {
    pool: PgPool,
    handlers: Arc<Vec<Arc<dyn EventHandler>>>,
    enabled: Arc<dyn EnabledModuleSet>,
}

impl std::fmt::Debug for Dispatcher {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Dispatcher")
            .field("pool", &"[configured]")
            .field(
                "handler_ids",
                &self
                    .handlers
                    .iter()
                    .map(|handler| handler.id())
                    .collect::<Vec<_>>(),
            )
            .field("enabled_module_set", &"[configured]")
            .finish()
    }
}

impl Dispatcher {
    /// MOD-04: registers compiled event handlers and the tenant module gate.
    pub fn new(
        pool: PgPool,
        handlers: Vec<Box<dyn EventHandler>>,
        enabled: Arc<dyn EnabledModuleSet>,
    ) -> Self {
        let handlers = handlers.into_iter().map(Arc::<dyn EventHandler>::from).collect();
        Self {
            pool,
            handlers: Arc::new(handlers),
            enabled,
        }
    }

    /// FLW-03: claims and delivers one bounded batch, returning completed events.
    pub async fn drain_once(&self) -> anyhow::Result<usize> {
        let (mut claim, rows) = claim_batch(&self.pool).await?;
        if rows.is_empty() {
            claim.commit().await.context("finish empty outbox claim")?;
            return Ok(0);
        }

        let outcome = deliver_groups(
            self.pool.clone(),
            Arc::clone(&self.handlers),
            Arc::clone(&self.enabled),
            group_events(rows),
        )
        .await;
        mark_dispatched(&mut claim, &outcome.completed).await?;
        claim
            .commit()
            .await
            .context("commit outbox claim and dispatch markers")?;
        if let Some(error) = outcome.error {
            return Err(error);
        }
        Ok(outcome.completed.len())
    }
}

struct ClaimedEvent {
    id: Uuid,
    tenant_id: TenantId,
    event_type: String,
    aggregate_id: Option<Uuid>,
    occurred_at: DateTime<Utc>,
    payload: Value,
}

#[derive(Deserialize)]
struct StoredEnvelope {
    actor: EventActor,
    request_id: Option<Uuid>,
    payload: Value,
}

struct GroupOutcome {
    completed: Vec<Uuid>,
    error: Option<anyhow::Error>,
}

async fn claim_batch(pool: &PgPool) -> anyhow::Result<(Transaction<'_, Postgres>, Vec<ClaimedEvent>)> {
    let mut claim = pool.begin().await.context("begin outbox claim")?;
    let rows = sqlx::query!(
        "WITH aggregate_heads AS MATERIALIZED ( \
             SELECT DISTINCT ON (tenant_id, aggregate_id) id \
             FROM public.outbox \
             WHERE dispatched_at IS NULL AND aggregate_id IS NOT NULL \
             ORDER BY tenant_id, aggregate_id, id \
         ) \
         SELECT candidate.id, candidate.tenant_id, candidate.event_type, \
                candidate.aggregate_id, candidate.payload, candidate.occurred_at \
         FROM public.outbox AS candidate \
         WHERE candidate.dispatched_at IS NULL \
           AND (candidate.aggregate_id IS NULL OR candidate.id IN ( \
               SELECT id FROM aggregate_heads \
           )) \
         ORDER BY candidate.id \
         LIMIT $1 \
         FOR UPDATE OF candidate SKIP LOCKED",
        CLAIM_BATCH_SIZE
    )
    .fetch_all(&mut *claim)
    .await
    .context("claim due outbox rows")?
    .into_iter()
    .map(|row| ClaimedEvent {
        id: row.id,
        tenant_id: TenantId(row.tenant_id),
        event_type: row.event_type,
        aggregate_id: row.aggregate_id,
        occurred_at: row.occurred_at,
        payload: row.payload,
    })
    .collect();
    Ok((claim, rows))
}

fn group_events(rows: Vec<ClaimedEvent>) -> VecDeque<Vec<ClaimedEvent>> {
    let mut grouped = BTreeMap::<(Uuid, Option<Uuid>), Vec<ClaimedEvent>>::new();
    for event in rows {
        grouped
            .entry((event.tenant_id.0, event.aggregate_id))
            .or_default()
            .push(event);
    }
    grouped.into_values().collect()
}

async fn mark_dispatched(claim: &mut Transaction<'_, Postgres>, event_ids: &[Uuid]) -> anyhow::Result<()> {
    for event_id in event_ids {
        sqlx::query!(
            "UPDATE public.outbox \
             SET dispatched_at = clock_timestamp() \
             WHERE id = $1 AND dispatched_at IS NULL",
            event_id
        )
        .execute(&mut **claim)
        .await
        .context("mark completed outbox event dispatched")?;
    }
    Ok(())
}

async fn deliver_groups(
    pool: PgPool,
    handlers: Arc<Vec<Arc<dyn EventHandler>>>,
    enabled: Arc<dyn EnabledModuleSet>,
    mut groups: VecDeque<Vec<ClaimedEvent>>,
) -> GroupOutcome {
    let mut outcome = GroupOutcome {
        completed: Vec::new(),
        error: None,
    };
    while !groups.is_empty() {
        let mut running = JoinSet::new();
        for _ in 0..MAX_CONCURRENT_GROUPS {
            let Some(group) = groups.pop_front() else {
                break;
            };
            let group_pool = pool.clone();
            let group_handlers = Arc::clone(&handlers);
            let group_enabled = Arc::clone(&enabled);
            running
                .spawn(async move { process_group(group_pool, group_handlers, group_enabled, group).await });
        }

        while let Some(result) = running.join_next().await {
            match result {
                Ok(group_outcome) => {
                    outcome.completed.extend(group_outcome.completed);
                    if outcome.error.is_none() {
                        outcome.error = group_outcome.error;
                    }
                }
                Err(error) if outcome.error.is_none() => {
                    outcome.error = Some(anyhow::anyhow!("outbox group task failed: {error}"));
                }
                Err(_) => {}
            }
        }
    }
    outcome
}

async fn process_group(
    pool: PgPool,
    handlers: Arc<Vec<Arc<dyn EventHandler>>>,
    enabled: Arc<dyn EnabledModuleSet>,
    group: Vec<ClaimedEvent>,
) -> GroupOutcome {
    let mut completed = Vec::new();
    let mut error = None;
    for row in group {
        match deliver_event(&pool, &handlers, &enabled, &row).await {
            Ok(()) => completed.push(row.id),
            Err(delivery_error) => {
                error = Some(delivery_error);
                break;
            }
        }
    }
    GroupOutcome { completed, error }
}

async fn deliver_event(
    pool: &PgPool,
    handlers: &[Arc<dyn EventHandler>],
    enabled: &Arc<dyn EnabledModuleSet>,
    row: &ClaimedEvent,
) -> anyhow::Result<()> {
    let subscribers = handlers
        .iter()
        .filter_map(|handler| {
            handler
                .subscribes()
                .iter()
                .copied()
                .find(|event_type| *event_type == row.event_type)
                .map(|event_type| (Arc::clone(handler), event_type))
        })
        .collect::<Vec<_>>();

    let Some((_, event_type)) = subscribers.first() else {
        return Ok(());
    };
    let stored: StoredEnvelope =
        serde_json::from_value(row.payload.clone()).context("decode stored outbox envelope")?;
    let event = Event {
        id: row.id,
        event_type,
        tenant_id: row.tenant_id,
        aggregate_id: row.aggregate_id,
        occurred_at: row.occurred_at,
        actor: stored.actor,
        request_id: stored.request_id,
        payload: stored.payload,
    };

    for (handler, _) in subscribers {
        deliver_to_handler(pool, enabled, &handler, &event)
            .await
            .with_context(|| format!("handler {} failed for event {}", handler.id(), event.id))?;
    }
    Ok(())
}

async fn deliver_to_handler(
    pool: &PgPool,
    enabled: &Arc<dyn EnabledModuleSet>,
    handler: &Arc<dyn EventHandler>,
    event: &Event,
) -> anyhow::Result<()> {
    let mut tx = crate::db::begin_system(pool, event.tenant_id)
        .await
        .context("begin tenant-scoped event handler transaction")?;
    let receipt = sqlx::query!(
        "INSERT INTO public.event_receipts (handler_id, event_id) \
         VALUES ($1, $2) \
         ON CONFLICT DO NOTHING \
         RETURNING handler_id",
        handler.id(),
        event.id
    )
    .fetch_optional(tx.conn())
    .await
    .context("insert event handler receipt")?;

    if receipt.is_none() {
        tx.commit().await.context("finish duplicate event receipt")?;
        return Ok(());
    }

    let is_enabled = match enabled.is_enabled(event.tenant_id, handler.module()).await {
        Ok(is_enabled) => is_enabled,
        Err(error) => {
            tx.rollback()
                .await
                .context("roll back receipt after module gate failure")?;
            return Err(error.into());
        }
    };
    if !is_enabled {
        tx.commit().await.context("commit receipt for disabled module")?;
        return Ok(());
    }

    if let Err(error) = handler.handle(&mut tx, event).await {
        tx.rollback()
            .await
            .context("roll back failed event handler transaction")?;
        return Err(error);
    }
    tx.commit().await.context("commit event handler transaction")?;
    Ok(())
}
