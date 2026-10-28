//! Event handler contracts consumed by compiled modules (ARC-07, MOD-04).

use crate::actor::{ActorCtx, ActorRole};
use crate::clock::{Clock, SystemClock};
use crate::db::Tx;
use crate::error::AppError;
use crate::ids::{PersonId, TenantId};
use crate::module::ModuleId;
use crate::ports::sinks::EventSink;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

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
