//! Audit persistence adapter (AUD-02, ERD-04).

use crate::record::AuditEvent;
use eq_core::actor::ActorRole;
use eq_core::db::Tx;
use eq_core::error::AppError;
use sqlx::types::Json;
use uuid::Uuid;

pub(super) async fn record(tx: &mut Tx, event: AuditEvent<'_>) -> Result<(), AppError> {
    let context = sqlx::query!(
        "SELECT NULLIF(pg_catalog.current_setting('app.tenant_id', true), '')::uuid AS tenant_id, NULLIF(pg_catalog.current_setting('app.actor_id', true), '')::uuid AS actor_id"
    )
    .fetch_one(tx.conn())
    .await?;

    if context.actor_id != event.actor.person_id.map(|person_id| person_id.0) {
        return Err(AppError::Internal(
            std::io::Error::other("audit actor does not match transaction context").into(),
        ));
    }
    let tenant_id = context.tenant_id.ok_or_else(|| {
        AppError::Internal(std::io::Error::other("audit tenant is missing from transaction context").into())
    })?;

    let actor_role = actor_role(event.actor.role)?;
    let id = Uuid::now_v7();
    let actor_id = event.actor.person_id.map(|person_id| person_id.0);
    let action = event.action;
    let entity_type = event.entity_type;
    let entity_id = event.entity_id;
    let category = event.category.as_str();
    let before = event.before.map(Json);
    let after = event.after.map(Json);
    let ip = event.actor.ip.map(|ip| ip.to_string());
    let request_id = event.actor.request_id;

    sqlx::query!(
        "INSERT INTO audit.entries (id, tenant_id, at, actor_id, actor_role, action, entity_type, entity_id, category, \"before\", \"after\", ip, request_id) VALUES ($1, $2, pg_catalog.statement_timestamp(), $3, $4, $5, $6, $7, $8, $9, $10, $11::text::inet, $12)",
        id,
        tenant_id,
        actor_id,
        actor_role,
        action,
        entity_type,
        entity_id,
        category,
        before as Option<Json<serde_json::Value>>,
        after as Option<Json<serde_json::Value>>,
        ip,
        request_id
    )
    .execute(tx.conn())
    .await?;

    Ok(())
}

fn actor_role(role: ActorRole) -> Result<&'static str, AppError> {
    let role = match role {
        ActorRole::Anonymous => {
            return Err(AppError::Internal(
                std::io::Error::other("anonymous actor role is not mapped by the audit schema").into(),
            ));
        }
        ActorRole::Owner => "owner",
        ActorRole::Admin => "admin",
        ActorRole::Instructor => "instructor",
        ActorRole::Guardian => "guardian",
        ActorRole::Student => "student",
        ActorRole::System => "system",
        ActorRole::Platform => "platform",
    };
    Ok(role)
}

#[cfg(test)]
mod tests {
    use super::actor_role;
    use eq_core::actor::ActorRole;

    #[test]
    fn anonymous_actor_role_is_rejected() {
        let result = actor_role(ActorRole::Anonymous);
        assert!(result.is_err());
    }
}
