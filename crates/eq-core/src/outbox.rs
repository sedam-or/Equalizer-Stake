//! Event handler contracts consumed by compiled modules (ARC-07, MOD-04).

use crate::actor::ActorRole;
use crate::db::Tx;
use crate::ids::{PersonId, TenantId};
use crate::module::ModuleId;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

/// CAN-60: actor fields included in an event envelope.
#[derive(Clone, Debug)]
pub struct EventActor {
    pub id: Option<PersonId>,
    pub role: ActorRole,
}

/// CAN-60: event envelope delivered to registered module handlers.
#[derive(Clone, Debug)]
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

/// MOD-04: one idempotent consumer registered by a module.
#[async_trait]
pub trait EventHandler: Send + Sync {
    fn id(&self) -> &'static str;
    fn module(&self) -> ModuleId;
    fn subscribes(&self) -> &'static [&'static str];
    async fn handle(&self, tx: &mut Tx, event: &Event) -> anyhow::Result<()>;
}
