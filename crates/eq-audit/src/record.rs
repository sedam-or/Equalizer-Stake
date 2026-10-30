//! Transaction-bound audit insertion (AUD-02, ARC-05).

use crate::Category;
use chrono::{DateTime, Utc};
use eq_core::actor::ActorCtx;
use eq_core::db::Tx;
use eq_core::error::AppError;
use std::fmt;
use uuid::Uuid;

/// AUD-02: event data written alongside its business mutation.
pub struct AuditEvent<'a> {
    pub action: &'a str,
    pub category: Category,
    pub entity_type: &'a str,
    pub entity_id: Option<Uuid>,
    pub before: Option<serde_json::Value>,
    pub after: Option<serde_json::Value>,
    pub actor: &'a ActorCtx,
}

impl fmt::Debug for AuditEvent<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AuditEvent")
            .field("action", &self.action)
            .field("category", &self.category)
            .field("entity_type", &self.entity_type)
            .field("entity_id", &self.entity_id)
            .field("before", &self.before.as_ref().map(|_| "[REDACTED]"))
            .field("after", &self.after.as_ref().map(|_| "[REDACTED]"))
            .field("actor", &"[REDACTED]")
            .finish()
    }
}

/// AUD-02, ARC-05: inserts the audit entry in the transaction that owns the business mutation.
pub async fn record(tx: &mut Tx, event: AuditEvent<'_>) -> Result<(), AppError> {
    crate::repo::record(tx, event).await
}

/// AUD-01: inserts an event only when this actor/entity/action has no record in the supplied UTC window.
pub async fn record_once_for_actor_entity_window(
    tx: &mut Tx,
    event: AuditEvent<'_>,
    at: DateTime<Utc>,
    from: DateTime<Utc>,
    until: DateTime<Utc>,
) -> Result<bool, AppError> {
    crate::repo::record_once_for_actor_entity_window(tx, event, at, from, until).await
}

#[cfg(test)]
mod tests {
    use super::AuditEvent;
    use eq_core::actor::{ActorCtx, ActorRole};
    use std::net::IpAddr;
    use std::str::FromStr;
    use uuid::Uuid;

    #[test]
    fn debug_redacts_audit_payload_and_actor() {
        let actor = ActorCtx {
            person_id: None,
            role: ActorRole::System,
            ip: Some(IpAddr::from_str("192.0.2.10").expect("valid test IP")),
            request_id: Some(Uuid::now_v7()),
        };
        let event = AuditEvent {
            action: "test.audit.record",
            category: crate::Category::Operational,
            entity_type: "test",
            entity_id: Some(Uuid::now_v7()),
            before: Some(serde_json::json!({ "name": "test-secret-audit" })),
            after: None,
            actor: &actor,
        };

        let debug = format!("{event:?}");
        assert!(!debug.contains("test-secret-audit"));
        assert!(!debug.contains("192.0.2.10"));
    }
}
