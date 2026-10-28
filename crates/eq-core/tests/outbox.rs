use anyhow::Result;
use chrono::Utc;
use eq_core::actor::{ActorCtx, ActorRole};
use eq_core::db;
use eq_core::error::AppError;
use eq_core::ids::{PersonId, TenantId};
use eq_core::module::ModuleId;
use eq_core::module::gate::EnabledModuleSet;
use eq_core::outbox::{Dispatcher, DomainEvent, Event, EventHandler, OutboxEventSink, emit};
use eq_core::ports::sinks::EventSink;
use schemars::JsonSchema;
use serde::Serialize;
use serde_json::{Value, json};
use sqlx::Row;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::Notify;
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
    insert_test_tenant(&db, tenant).await?;
    Ok((db, tenant))
}

async fn insert_test_tenant(db: &TestDb, tenant: TenantId) -> Result<()> {
    let slug = format!("outbox-{}", tenant.0.simple());
    sqlx::query(
        "INSERT INTO tenants (id, slug, name, country, preset_id, preset_version) \
         VALUES ($1, $2, 'Outbox Test School', 'ID', 'test', 1)",
    )
    .bind(tenant.0)
    .bind(slug)
    .execute(&db.pool)
    .await?;
    Ok(())
}

async fn insert_event(
    db: &TestDb,
    id: Uuid,
    tenant: TenantId,
    event_type: &str,
    aggregate_id: Option<Uuid>,
) -> Result<()> {
    let payload = json!({
        "actor": { "id": null, "role": "system" },
        "request_id": null,
        "payload": { "test": true }
    });
    let occurred_at = chrono::DateTime::UNIX_EPOCH;
    sqlx::query!(
        "INSERT INTO public.outbox (id, tenant_id, event_type, aggregate_id, payload, occurred_at) \
         VALUES ($1, $2, $3, $4, $5, $6)",
        id,
        tenant.0,
        event_type,
        aggregate_id,
        payload,
        occurred_at
    )
    .execute(&db.pool)
    .await?;
    Ok(())
}

async fn count_handler_receipts(db: &TestDb, handler_id: &str) -> Result<i64> {
    Ok(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM public.event_receipts WHERE handler_id = $1")
            .bind(handler_id)
            .fetch_one(&db.pool)
            .await?,
    )
}

async fn count_dispatched(db: &TestDb) -> Result<i64> {
    Ok(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM public.outbox WHERE dispatched_at IS NOT NULL")
            .fetch_one(&db.pool)
            .await?,
    )
}

struct TestModuleGate {
    enabled: bool,
}

#[async_trait::async_trait]
impl EnabledModuleSet for TestModuleGate {
    async fn is_enabled(&self, _tenant: TenantId, _module: ModuleId) -> std::result::Result<bool, AppError> {
        Ok(self.enabled)
    }
}

struct RecordingHandler {
    handler_id: &'static str,
    module: ModuleId,
    subscriptions: &'static [&'static str],
    calls: Arc<AtomicUsize>,
    seen: Arc<Mutex<Vec<(Uuid, String)>>>,
    fails: bool,
}

impl Clone for RecordingHandler {
    fn clone(&self) -> Self {
        Self {
            handler_id: self.handler_id,
            module: self.module,
            subscriptions: self.subscriptions,
            calls: Arc::clone(&self.calls),
            seen: Arc::clone(&self.seen),
            fails: self.fails,
        }
    }
}

impl RecordingHandler {
    fn new(handler_id: &'static str, subscriptions: &'static [&'static str]) -> Self {
        Self {
            handler_id,
            module: ModuleId::Billing,
            subscriptions,
            calls: Arc::new(AtomicUsize::new(0)),
            seen: Arc::new(Mutex::new(Vec::new())),
            fails: false,
        }
    }

    fn failing(mut self) -> Self {
        self.fails = true;
        self
    }
}

#[async_trait::async_trait]
impl EventHandler for RecordingHandler {
    fn id(&self) -> &'static str {
        self.handler_id
    }

    fn module(&self) -> ModuleId {
        self.module
    }

    fn subscribes(&self) -> &'static [&'static str] {
        self.subscriptions
    }

    async fn handle(&self, tx: &mut db::Tx, event: &Event) -> Result<()> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let tenant_id = sqlx::query_scalar::<_, String>("SELECT current_setting('app.tenant_id', true)")
            .fetch_one(tx.conn())
            .await?;
        self.seen
            .lock()
            .map_err(|_| anyhow::anyhow!("test observation mutex poisoned"))?
            .push((event.id, tenant_id));
        if self.fails {
            anyhow::bail!("synthetic handler failure");
        }
        Ok(())
    }
}

fn dispatcher(db: &TestDb, handler: RecordingHandler, enabled: bool) -> Dispatcher {
    Dispatcher::new(
        db.app_pool.clone(),
        vec![Box::new(handler)],
        Arc::new(TestModuleGate { enabled }),
    )
}

#[derive(Clone)]
struct BlockingOrderingHandler {
    blocked_event: Uuid,
    started: Arc<Notify>,
    release: Arc<Notify>,
    seen: Arc<Mutex<Vec<(Uuid, String)>>>,
}

#[async_trait::async_trait]
impl EventHandler for BlockingOrderingHandler {
    fn id(&self) -> &'static str {
        "test.outbox.ordering"
    }

    fn module(&self) -> ModuleId {
        ModuleId::Billing
    }

    fn subscribes(&self) -> &'static [&'static str] {
        &[EVENT_TYPE]
    }

    async fn handle(&self, tx: &mut db::Tx, event: &Event) -> Result<()> {
        if event.id == self.blocked_event {
            self.started.notify_one();
            self.release.notified().await;
        }
        let tenant_id = sqlx::query_scalar::<_, String>("SELECT current_setting('app.tenant_id', true)")
            .fetch_one(tx.conn())
            .await?;
        self.seen
            .lock()
            .map_err(|_| anyhow::anyhow!("test observation mutex poisoned"))?
            .push((event.id, tenant_id));
        Ok(())
    }
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

#[tokio::test]
async fn two_dispatchers_exactly_once_per_handler() -> Result<()> {
    let (db, tenant) = fixture().await?;
    for index in 0..500_u128 {
        insert_event(
            &db,
            Uuid::from_u128(1_000 + index),
            tenant,
            EVENT_TYPE,
            Some(Uuid::from_u128(10_000 + index / 10)),
        )
        .await?;
    }

    let handler = RecordingHandler::new("test.outbox.counter", &[EVENT_TYPE]);
    let calls = Arc::clone(&handler.calls);
    let first = dispatcher(&db, handler.clone(), true);
    let second = dispatcher(&db, handler, true);
    let mut delivered = 0;
    loop {
        let (first_count, second_count) = tokio::join!(first.drain_once(), second.drain_once());
        let first_count = first_count?;
        let second_count = second_count?;
        delivered += first_count + second_count;
        if first_count == 0 && second_count == 0 {
            break;
        }
    }

    assert_eq!(delivered, 500);
    assert_eq!(calls.load(Ordering::SeqCst), 500);
    assert_eq!(count_handler_receipts(&db, "test.outbox.counter").await?, 500);
    assert_eq!(count_dispatched(&db).await?, 500);
    Ok(())
}

#[tokio::test]
async fn order_within_aggregate() -> Result<()> {
    let (db, first_tenant) = fixture().await?;
    let second_tenant = TenantId(Uuid::now_v7());
    insert_test_tenant(&db, second_tenant).await?;
    let aggregate = Uuid::from_u128(0xaaa);
    let first_id = Uuid::from_u128(0x100);
    let first_tenant_ids = (0..150_u128)
        .map(|index| Uuid::from_u128(0x100 + index))
        .collect::<Vec<_>>();
    for event_id in &first_tenant_ids {
        insert_event(&db, *event_id, first_tenant, EVENT_TYPE, Some(aggregate)).await?;
    }
    let other_tenant_id = Uuid::from_u128(0x500);
    insert_event(&db, other_tenant_id, second_tenant, EVENT_TYPE, Some(aggregate)).await?;

    let handler = BlockingOrderingHandler {
        blocked_event: first_id,
        started: Arc::new(Notify::new()),
        release: Arc::new(Notify::new()),
        seen: Arc::new(Mutex::new(Vec::new())),
    };
    let enabled: Arc<dyn EnabledModuleSet> = Arc::new(TestModuleGate { enabled: true });
    let seen = Arc::clone(&handler.seen);
    let first_dispatcher = Dispatcher::new(
        db.app_pool.clone(),
        vec![Box::new(handler.clone())],
        Arc::clone(&enabled),
    );
    let second_dispatcher = Dispatcher::new(db.app_pool.clone(), vec![Box::new(handler.clone())], enabled);
    let started = Arc::clone(&handler.started);
    let release = Arc::clone(&handler.release);
    let first_drain = tokio::spawn(async move { first_dispatcher.drain_once().await });
    tokio::time::timeout(std::time::Duration::from_secs(5), started.notified()).await?;

    // While the oldest event is held in its handler transaction, a second dispatcher must not
    // claim any later event from the same tenant/aggregate, even when those rows are outside the
    // first dispatcher's 100-row batch.
    assert_eq!(second_dispatcher.drain_once().await?, 0);
    {
        let seen = seen
            .lock()
            .map_err(|_| anyhow::anyhow!("test observation mutex poisoned"))?;
        assert!(!seen.iter().any(|(event_id, tenant_id)| {
            tenant_id == &first_tenant.0.to_string() && *event_id != first_id
        }));
    }
    release.notify_one();

    let mut delivered = first_drain.await??;
    while let count @ 1.. = second_dispatcher.drain_once().await? {
        delivered += count;
    }
    assert_eq!(delivered, 151);

    let seen = seen
        .lock()
        .map_err(|_| anyhow::anyhow!("test observation mutex poisoned"))?;
    let first_tenant_events = seen
        .iter()
        .filter(|(_, tenant_id)| tenant_id == &first_tenant.0.to_string())
        .map(|(event_id, _)| *event_id)
        .collect::<Vec<_>>();
    assert_eq!(first_tenant_events, first_tenant_ids);
    assert!(seen.iter().any(|(event_id, tenant_id)| {
        *event_id == other_tenant_id && tenant_id == &second_tenant.0.to_string()
    }));
    Ok(())
}

#[tokio::test]
async fn disabled_module_handler_skipped() -> Result<()> {
    let (db, tenant) = fixture().await?;
    let disabled_event_id = Uuid::from_u128(0x200);
    let unregistered_event_id = Uuid::from_u128(0x201);
    insert_event(&db, disabled_event_id, tenant, EVENT_TYPE, None).await?;
    insert_event(&db, unregistered_event_id, tenant, "core.test.unhandled", None).await?;

    let handler = RecordingHandler::new("test.outbox.disabled", &[EVENT_TYPE]);
    let calls = Arc::clone(&handler.calls);
    let dispatcher = dispatcher(&db, handler, false);
    assert_eq!(dispatcher.drain_once().await?, 2);

    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert_eq!(count_handler_receipts(&db, "test.outbox.disabled").await?, 1);
    assert_eq!(count_dispatched(&db).await?, 2);
    Ok(())
}

#[tokio::test]
async fn duplicate_receipt_skips_handler() -> Result<()> {
    let (db, tenant) = fixture().await?;
    let event_id = Uuid::from_u128(0x300);
    insert_event(&db, event_id, tenant, EVENT_TYPE, None).await?;
    sqlx::query("INSERT INTO public.event_receipts (handler_id, event_id) VALUES ($1, $2)")
        .bind("test.outbox.duplicate")
        .bind(event_id)
        .execute(&db.pool)
        .await?;

    let handler = RecordingHandler::new("test.outbox.duplicate", &[EVENT_TYPE]);
    let calls = Arc::clone(&handler.calls);
    let dispatcher = dispatcher(&db, handler, true);
    assert_eq!(dispatcher.drain_once().await?, 1);

    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert_eq!(count_handler_receipts(&db, "test.outbox.duplicate").await?, 1);
    assert_eq!(count_dispatched(&db).await?, 1);
    Ok(())
}

#[tokio::test]
async fn handler_failure_rolls_back_receipt_and_keeps_event_pending() -> Result<()> {
    let (db, tenant) = fixture().await?;
    insert_event(&db, Uuid::from_u128(0x400), tenant, EVENT_TYPE, None).await?;

    let handler = RecordingHandler::new("test.outbox.failure", &[EVENT_TYPE]).failing();
    let calls = Arc::clone(&handler.calls);
    let dispatcher = dispatcher(&db, handler, true);
    assert!(dispatcher.drain_once().await.is_err());

    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(count_handler_receipts(&db, "test.outbox.failure").await?, 0);
    assert_eq!(count_dispatched(&db).await?, 0);
    Ok(())
}
