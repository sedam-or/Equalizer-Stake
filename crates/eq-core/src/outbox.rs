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
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::{PgPool, Postgres, Transaction};
use std::collections::{BTreeMap, VecDeque};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

const CLAIM_BATCH_SIZE: i64 = 100; // technical: FLW-03 bounds one drain query
const MAX_CONCURRENT_GROUPS: usize = 8; // technical: T-107 bounds aggregate concurrency
const MAX_HANDLER_FAILURES: i32 = 5; // technical: T-107's retryable failure bound
const RETRY_BACKOFF_SECONDS: [i64; 5] = [1, 5, 30, 300, 1_800]; // technical: T-107 specifies retry backoff
const OUTBOX_POLL_INTERVAL: Duration = Duration::from_secs(2); // technical: T-107's notification fallback
const RETRY_MARKER_PREFIX: &str = "ritma-retry-v1:";
const DEAD_MARKER: &str = "ritma-dead-v1:handler_failed";
const DEAD_LETTER_KIND: &str = "outbox_dead_letter";
const DEAD_LETTER_DEDUPE_PREFIX: &str = "outbox_dead_letter:";

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

/// ERR-52: classifies event-handler failures without exposing their source text in Debug output.
#[derive(thiserror::Error)]
pub enum EventHandlerError {
    #[error("retryable event handler failure")]
    Retryable(#[source] anyhow::Error),
    #[error("permanent event handler failure")]
    Permanent(#[source] anyhow::Error),
}

impl std::fmt::Debug for EventHandlerError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let class = match self {
            Self::Retryable(_) => "Retryable",
            Self::Permanent(_) => "Permanent",
        };
        formatter.debug_tuple("EventHandlerError").field(&class).finish()
    }
}

impl From<anyhow::Error> for EventHandlerError {
    fn from(error: anyhow::Error) -> Self {
        Self::Retryable(error)
    }
}

impl EventHandlerError {
    fn is_permanent(&self) -> bool {
        matches!(self, Self::Permanent(_))
    }

    fn with_context(self, context: String) -> Self {
        match self {
            Self::Retryable(error) => Self::Retryable(error.context(context)),
            Self::Permanent(error) => Self::Permanent(error.context(context)),
        }
    }
}

/// MOD-04: one idempotent consumer registered by a module.
#[async_trait]
pub trait EventHandler: Send + Sync {
    fn id(&self) -> &'static str;
    fn module(&self) -> ModuleId;
    fn subscribes(&self) -> &'static [&'static str];
    async fn handle(&self, tx: &mut Tx, event: &Event) -> Result<(), EventHandlerError>;
}

/// T-107: process-local delivery counters with no tenant or event labels.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DispatcherCounters {
    pub attempted: u64,
    pub dispatched: u64,
    pub failed: u64,
    pub retry_scheduled: u64,
    pub dead_lettered: u64,
}

#[derive(Debug, Default)]
struct CounterState {
    attempted: AtomicU64,
    dispatched: AtomicU64,
    failed: AtomicU64,
    retry_scheduled: AtomicU64,
    dead_lettered: AtomicU64,
}

/// FLW-03: bounded delivery of outbox events to tenant-enabled module handlers.
pub struct Dispatcher {
    pool: PgPool,
    handlers: Arc<Vec<Arc<dyn EventHandler>>>,
    enabled: Arc<dyn EnabledModuleSet>,
    clock: Arc<dyn Clock>,
    counters: CounterState,
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
        Self::new_with_clock(pool, handlers, enabled, Arc::new(SystemClock))
    }

    /// ARC-10: constructs a dispatcher with a controllable clock for retry scheduling.
    pub fn new_with_clock(
        pool: PgPool,
        handlers: Vec<Box<dyn EventHandler>>,
        enabled: Arc<dyn EnabledModuleSet>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        let handlers = handlers.into_iter().map(Arc::<dyn EventHandler>::from).collect();
        Self {
            pool,
            handlers: Arc::new(handlers),
            enabled,
            clock,
            counters: CounterState::default(),
        }
    }

    /// T-107: returns the current process-local outbox counters.
    pub fn counters(&self) -> DispatcherCounters {
        DispatcherCounters {
            attempted: self.counters.attempted.load(Ordering::Relaxed),
            dispatched: self.counters.dispatched.load(Ordering::Relaxed),
            failed: self.counters.failed.load(Ordering::Relaxed),
            retry_scheduled: self.counters.retry_scheduled.load(Ordering::Relaxed),
            dead_lettered: self.counters.dead_lettered.load(Ordering::Relaxed),
        }
    }

    /// T-107: wakes on committed outbox inserts and polls as a recovery fallback.
    pub async fn run(&self, shutdown: CancellationToken) -> anyhow::Result<()> {
        let mut listener = sqlx::postgres::PgListener::connect_with(&self.pool)
            .await
            .context("connect outbox listener")?;
        listener
            .listen("eq_outbox")
            .await
            .context("listen for outbox inserts")?;

        loop {
            tokio::select! {
                _ = shutdown.cancelled() => return Ok(()),
                notification = listener.recv() => {
                    notification.context("receive outbox notification")?;
                    self.drain_once().await?;
                }
                _ = tokio::time::sleep(OUTBOX_POLL_INTERVAL) => {
                    self.drain_once().await?;
                }
            }
        }
    }

    /// FLW-03: claims and delivers one bounded batch, returning attempted events.
    pub async fn drain_once(&self) -> anyhow::Result<usize> {
        let now_ms = self.clock.now().timestamp_millis();
        let (mut claim, rows) = claim_batch(&self.pool, now_ms).await?;
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
        let (retry_scheduled, dead_lettered) = self.mark_failed_events(&mut claim, &outcome.failures).await?;
        claim
            .commit()
            .await
            .context("commit outbox claim and dispatch markers")?;
        self.record_committed_outcome(&outcome, retry_scheduled, dead_lettered);
        if let Some(error) = outcome.error {
            return Err(error);
        }
        Ok(outcome.attempted)
    }

    async fn mark_failed_events(
        &self,
        claim: &mut Transaction<'_, Postgres>,
        failures: &[FailedEvent],
    ) -> anyhow::Result<(u64, u64)> {
        let mut retry_scheduled = 0_u64;
        let mut dead_lettered = 0_u64;
        for failure in failures {
            let attempts = failure.attempts.saturating_add(1);
            let dead = failure.permanent || attempts >= MAX_HANDLER_FAILURES;
            let marker = if dead {
                raise_dead_letter_attention(&self.pool, failure.tenant_id, failure.id).await?;
                dead_lettered += 1;
                DEAD_MARKER.to_owned()
            } else {
                let delay = RETRY_BACKOFF_SECONDS[usize::try_from(attempts - 1)
                    .context("convert outbox retry attempt to schedule index")?];
                let due_at = self
                    .clock
                    .now()
                    .checked_add_signed(ChronoDuration::seconds(delay))
                    .context("calculate outbox retry due time")?;
                let due_ms = due_at.timestamp_millis();
                retry_scheduled += 1;
                format!("{RETRY_MARKER_PREFIX}{due_ms}:handler_failed")
            };
            mark_failed(claim, failure.id, attempts, &marker).await?;
        }
        Ok((retry_scheduled, dead_lettered))
    }

    fn record_committed_outcome(&self, outcome: &GroupOutcome, retry_scheduled: u64, dead_lettered: u64) {
        self.counters
            .attempted
            .fetch_add(outcome.attempted as u64, Ordering::Relaxed);
        self.counters
            .dispatched
            .fetch_add(outcome.completed.len() as u64, Ordering::Relaxed);
        self.counters
            .failed
            .fetch_add(outcome.failures.len() as u64, Ordering::Relaxed);
        self.counters
            .retry_scheduled
            .fetch_add(retry_scheduled, Ordering::Relaxed);
        self.counters
            .dead_lettered
            .fetch_add(dead_lettered, Ordering::Relaxed);
        for event_id in &outcome.completed {
            tracing::info!(
                event_id = %event_id,
                failure_code = "none",
                outcome = "dispatched",
                outbox_dispatched = 1_u64,
                "outbox event dispatched"
            );
        }
        for failure in &outcome.failures {
            let attempts = failure.attempts.saturating_add(1);
            let dead = failure.permanent || attempts >= MAX_HANDLER_FAILURES;
            if dead {
                tracing::error!(
                    event_id = %failure.id,
                    tenant_id = %failure.tenant_id.0,
                    failure_code = "handler_failed",
                    error_class = if failure.permanent { "permanent" } else { "retryable" },
                    outcome = "dead_lettered",
                    outbox_failed = 1_u64,
                    outbox_dead_lettered = 1_u64,
                    "outbox event dead-lettered"
                );
            } else {
                tracing::warn!(
                    event_id = %failure.id,
                    tenant_id = %failure.tenant_id.0,
                    failure_code = "handler_failed",
                    attempt = attempts,
                    error_class = "retryable",
                    outcome = "retry_scheduled",
                    outbox_failed = 1_u64,
                    outbox_retry_scheduled = 1_u64,
                    "outbox handler failed; retry scheduled"
                );
            }
        }
    }
}

struct ClaimedEvent {
    id: Uuid,
    tenant_id: TenantId,
    attempts: i32,
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
    failures: Vec<FailedEvent>,
    attempted: usize,
    error: Option<anyhow::Error>,
}

struct FailedEvent {
    id: Uuid,
    tenant_id: TenantId,
    attempts: i32,
    permanent: bool,
}

async fn claim_batch(
    pool: &PgPool,
    now_ms: i64,
) -> anyhow::Result<(Transaction<'_, Postgres>, Vec<ClaimedEvent>)> {
    let mut claim = pool.begin().await.context("begin outbox claim")?;
    let rows = sqlx::query!(
        "WITH aggregate_heads AS MATERIALIZED ( \
             SELECT DISTINCT ON (tenant_id, aggregate_id) id \
             FROM public.outbox \
             WHERE dispatched_at IS NULL AND attempts < $3 AND aggregate_id IS NOT NULL \
               AND (last_error IS NULL OR last_error <> $4) \
             ORDER BY tenant_id, aggregate_id, id \
         ) \
         SELECT candidate.id, candidate.tenant_id, candidate.attempts, candidate.event_type, \
                candidate.aggregate_id, candidate.payload, candidate.occurred_at \
         FROM public.outbox AS candidate \
         WHERE candidate.dispatched_at IS NULL \
           AND candidate.attempts < $3 \
           AND (candidate.last_error IS NULL OR CASE \
               WHEN candidate.last_error ~ '^ritma-retry-v1:[0-9]+:handler_failed$' \
               THEN split_part(candidate.last_error, ':', 2)::bigint <= $2 \
               ELSE false END) \
           AND (candidate.aggregate_id IS NULL OR candidate.id IN ( \
               SELECT id FROM aggregate_heads \
           )) \
         ORDER BY candidate.id \
         LIMIT $1 \
         FOR UPDATE OF candidate SKIP LOCKED",
        CLAIM_BATCH_SIZE,
        now_ms,
        MAX_HANDLER_FAILURES,
        DEAD_MARKER
    )
    .fetch_all(&mut *claim)
    .await
    .context("claim due outbox rows")?
    .into_iter()
    .map(|row| ClaimedEvent {
        id: row.id,
        tenant_id: TenantId(row.tenant_id),
        attempts: row.attempts,
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
             SET dispatched_at = clock_timestamp(), last_error = NULL \
             WHERE id = $1 AND dispatched_at IS NULL",
            event_id
        )
        .execute(&mut **claim)
        .await
        .context("mark completed outbox event dispatched")?;
    }
    Ok(())
}

async fn mark_failed(
    claim: &mut Transaction<'_, Postgres>,
    event_id: Uuid,
    attempts: i32,
    marker: &str,
) -> anyhow::Result<()> {
    let result = sqlx::query!(
        "UPDATE public.outbox SET attempts = $2, last_error = $3 \
         WHERE id = $1 AND dispatched_at IS NULL",
        event_id,
        attempts,
        marker
    )
    .execute(&mut **claim)
    .await
    .context("persist outbox retry or dead marker")?;
    if result.rows_affected() != 1 {
        anyhow::bail!("outbox failure marker did not update exactly one row");
    }
    Ok(())
}

async fn raise_dead_letter_attention(
    pool: &PgPool,
    tenant_id: TenantId,
    event_id: Uuid,
) -> anyhow::Result<()> {
    let mut tx = crate::db::begin_system(pool, tenant_id)
        .await
        .context("begin tenant-scoped dead-letter attention transaction")?;
    let dedupe_key = format!("{DEAD_LETTER_DEDUPE_PREFIX}{event_id}");
    let payload = json!({ "event_id": event_id });
    sqlx::query!(
        "INSERT INTO public.attention_items \
             (id, tenant_id, module_id, kind, ref_type, ref_id, payload, severity, dedupe_key) \
         VALUES ($1, $2, 'core', $3, 'outbox', $4, $5, 'high', $6) \
         ON CONFLICT (tenant_id, dedupe_key) DO UPDATE SET \
             payload = EXCLUDED.payload, \
             severity = EXCLUDED.severity, \
             resolved_at = NULL, \
             resolved_by = NULL",
        Uuid::now_v7(),
        tenant_id.0,
        DEAD_LETTER_KIND,
        event_id,
        payload,
        dedupe_key
    )
    .execute(tx.conn())
    .await
    .context("upsert outbox dead-letter attention item")?;
    tx.commit()
        .await
        .context("commit outbox dead-letter attention item")?;
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
        failures: Vec::new(),
        attempted: 0,
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
                    outcome.failures.extend(group_outcome.failures);
                    outcome.attempted += group_outcome.attempted;
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
    let mut failures = Vec::new();
    let mut attempted = 0;
    for row in group {
        attempted += 1;
        match deliver_event(&pool, &handlers, &enabled, &row).await {
            Ok(()) => completed.push(row.id),
            Err(delivery_error) => {
                failures.push(FailedEvent {
                    id: row.id,
                    tenant_id: row.tenant_id,
                    attempts: row.attempts,
                    permanent: delivery_error.is_permanent(),
                });
                break;
            }
        }
    }
    GroupOutcome {
        completed,
        failures,
        attempted,
        error: None,
    }
}

async fn deliver_event(
    pool: &PgPool,
    handlers: &[Arc<dyn EventHandler>],
    enabled: &Arc<dyn EnabledModuleSet>,
    row: &ClaimedEvent,
) -> Result<(), EventHandlerError> {
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
        if let Err(error) = deliver_to_handler(pool, enabled, &handler, &event).await {
            return Err(error.with_context(format!(
                "handler {} failed for event {}",
                handler.id(),
                event.id
            )));
        }
    }
    Ok(())
}

async fn deliver_to_handler(
    pool: &PgPool,
    enabled: &Arc<dyn EnabledModuleSet>,
    handler: &Arc<dyn EventHandler>,
    event: &Event,
) -> Result<(), EventHandlerError> {
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
            return Err(anyhow::Error::from(error).into());
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
