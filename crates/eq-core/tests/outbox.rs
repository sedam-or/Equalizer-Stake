use anyhow::Result;
use chrono::Utc;
use eq_core::actor::{ActorCtx, ActorRole};
use eq_core::db;
use eq_core::ids::{PersonId, TenantId};
use eq_core::module::ModuleId;
use eq_core::outbox::{DomainEvent, Event, OutboxEventSink, emit};
use eq_core::ports::sinks::EventSink;
use schemars::JsonSchema;
use serde::Serialize;
use serde_json::{Value, json};
use sqlx::Row;
use uuid::Uuid;

#[expect(
    dead_code,
    reason = "the shared TestDb helper exposes URLs for suites that construct extra pools"
)]
#[path = "../../eq-testkit/src/db.rs"]
mod shared_test_db;

use shared_test_db::TestDb;

const EVENT_TYPE: &str = "core.test.v1";

fn test_time() -> chrono::DateTime<Utc> {
    chrono::DateTime::UNIX_EPOCH
}

#[derive(Debug, JsonSchema, Serialize)]
struct ToggleEvent {
    module: ModuleId,
    enabled: bool,
}

impl DomainEvent for ToggleEvent {
    const TYPE: &'static str = EVENT_TYPE;

    fn aggregate_id(&self) -> Option<Uuid> {
        None
    }
}

async fn fixture() -> Result<(TestDb, TenantId)> {
    let db = TestDb::new().await;
    let tenant = TenantId(Uuid::now_v7());
    let slug = format!("outbox-{}", tenant.0.simple());
    sqlx::query(
        "INSERT INTO tenants (id, slug, name, country, preset_id, preset_version) \
         VALUES ($1, $2, 'Outbox Test School', 'ID', 'test', 1)",
    )
    .bind(tenant.0)
    .bind(slug)
    .execute(&db.pool)
    .await?;
    Ok((db, tenant))
}

async fn emit_toggle(tx: &mut db::Tx, tenant: TenantId) -> Result<()> {
    emit(
        tx,
        tenant,
        &ToggleEvent {
            module: ModuleId::Billing,
            enabled: true,
        },
    )
    .await?;
    Ok(())
}

async fn count_outbox(db: &TestDb, tenant: TenantId) -> Result<i64> {
    Ok(
        sqlx::query_scalar("SELECT count(*) FROM outbox WHERE tenant_id = $1")
            .bind(tenant.0)
            .fetch_one(&db.pool)
            .await?,
    )
}

mod outbox {
    pub mod tests {
        use super::super::*;

        #[tokio::test]
        async fn rollback_no_event() -> Result<()> {
            let (db, tenant) = fixture().await?;
            let mut tx = db::begin_system(&db.app_pool, tenant).await?;
            emit_toggle(&mut tx, tenant).await?;
            tx.rollback().await?;

            assert_eq!(count_outbox(&db, tenant).await?, 0);
            Ok(())
        }
    }
}

#[tokio::test]
async fn emit_rejects_tenant_mismatch() -> Result<()> {
    let (db, tenant) = fixture().await?;
    let other_tenant = TenantId(Uuid::now_v7());
    let mut tx = db::begin_system(&db.app_pool, tenant).await?;
    let result = emit_toggle(&mut tx, other_tenant).await;
    assert!(result.is_err());
    tx.rollback().await?;

    assert_eq!(count_outbox(&db, tenant).await?, 0);
    Ok(())
}

#[tokio::test]
async fn outbox_insert_notifies_listener_after_commit() -> Result<()> {
    let (db, tenant) = fixture().await?;
    let mut listener = sqlx::postgres::PgListener::connect_with(&db.app_pool).await?;
    listener.listen("eq_outbox").await?;

    let mut tx = db::begin_system(&db.app_pool, tenant).await?;
    emit_toggle(&mut tx, tenant).await?;
    tx.commit().await?;

    let notification = listener.recv().await?;
    assert_eq!(notification.channel(), "eq_outbox");
    assert_eq!(notification.payload(), "");
    Ok(())
}

#[tokio::test]
async fn event_sink_rejects_actor_mismatch() -> Result<()> {
    let (db, tenant) = fixture().await?;
    let transaction_actor = ActorCtx::system("test");
    let other_actor = ActorCtx {
        person_id: Some(PersonId(Uuid::now_v7())),
        role: ActorRole::Admin,
        ip: None,
        request_id: None,
    };
    let mut tx = db::begin(&db.app_pool, tenant, &transaction_actor).await?;
    let result = OutboxEventSink
        .emit_module_toggled(&mut tx, tenant, &other_actor, ModuleId::Billing, true)
        .await;
    assert!(result.is_err());
    tx.rollback().await?;

    assert_eq!(count_outbox(&db, tenant).await?, 0);
    Ok(())
}

#[tokio::test]
async fn event_sink_preserves_actor_and_request_context() -> Result<()> {
    let (db, tenant) = fixture().await?;
    let actor_id = Uuid::now_v7();
    let request_id = Uuid::now_v7();
    let actor = ActorCtx {
        person_id: Some(PersonId(actor_id)),
        role: ActorRole::Instructor,
        ip: None,
        request_id: Some(request_id),
    };
    let mut tx = db::begin(&db.app_pool, tenant, &actor).await?;
    OutboxEventSink
        .emit_module_toggled(&mut tx, tenant, &actor, ModuleId::Billing, true)
        .await?;
    tx.commit().await?;

    let payload: Value = sqlx::query_scalar("SELECT payload FROM outbox WHERE tenant_id = $1")
        .bind(tenant.0)
        .fetch_one(&db.pool)
        .await?;
    assert_eq!(payload["actor"]["id"], actor_id.to_string());
    assert_eq!(payload["actor"]["role"], "instructor");
    assert_eq!(payload["request_id"], request_id.to_string());
    Ok(())
}

#[tokio::test]
async fn policy_changed_event_preserves_metadata() -> Result<()> {
    let (db, tenant) = fixture().await?;
    let actor = ActorCtx::system("test");
    let effective_from = test_time();
    let mut tx = db::begin(&db.app_pool, tenant, &actor).await?;
    OutboxEventSink
        .emit_policy_changed(&mut tx, tenant, &actor, ModuleId::Core, 7, effective_from)
        .await?;
    tx.commit().await?;

    let row = sqlx::query("SELECT event_type, payload FROM outbox WHERE tenant_id = $1")
        .bind(tenant.0)
        .fetch_one(&db.pool)
        .await?;
    let event_type: String = row.try_get("event_type")?;
    let payload: Value = row.try_get("payload")?;
    assert_eq!(event_type, "core.policy_changed.v1");
    assert_eq!(payload["payload"]["module"], "core");
    assert_eq!(payload["payload"]["version"], 7);
    assert_eq!(
        payload["payload"]["effective_from"],
        serde_json::to_value(effective_from)?
    );
    Ok(())
}

#[test]
fn event_debug_redacts_payload() {
    let event = Event {
        id: Uuid::now_v7(),
        event_type: EVENT_TYPE,
        tenant_id: TenantId(Uuid::now_v7()),
        aggregate_id: None,
        occurred_at: test_time(),
        actor: eq_core::outbox::EventActor {
            id: None,
            role: ActorRole::System,
        },
        request_id: None,
        payload: json!({ "token": "test-secret-outbox-payload" }),
    };

    let debug = format!("{event:?}");
    assert!(!debug.contains("test-secret-outbox-payload"));
    assert!(debug.contains("REDACTED"));
}
