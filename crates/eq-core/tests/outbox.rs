use anyhow::Result;
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use eq_core::actor::{ActorCtx, ActorRole};
use eq_core::clock::Clock;
use eq_core::db;
use eq_core::error::AppError;
use eq_core::ids::{PersonId, TenantId};
use eq_core::module::ModuleId;
use eq_core::module::gate::EnabledModuleSet;
use eq_core::outbox::{
    Dispatcher, DomainEvent, Event, EventHandler, EventHandlerError, OutboxEventSink, emit,
};
use eq_core::ports::sinks::EventSink;
use schemars::JsonSchema;
use serde::Serialize;
use serde_json::{Value, json};
use sqlx::Row;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

#[expect(
    dead_code,
    reason = "the shared TestDb helper exposes URLs for suites that construct extra pools"
)]
#[path = "../../eq-testkit/src/db.rs"]
mod shared_test_db;

use shared_test_db::TestDb;

const EVENT_TYPE: &str = "core.test.v1";

fn test_time() -> DateTime<Utc> {
    DateTime::UNIX_EPOCH
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
    let occurred_at = DateTime::UNIX_EPOCH;
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

async fn outbox_attempt_state(
    db: &TestDb,
    event_id: Uuid,
) -> Result<(i32, Option<String>, Option<DateTime<Utc>>)> {
    let row = sqlx::query("SELECT attempts, last_error, dispatched_at FROM public.outbox WHERE id = $1")
        .bind(event_id)
        .fetch_one(&db.pool)
        .await?;
    Ok((
        row.try_get("attempts")?,
        row.try_get("last_error")?,
        row.try_get("dispatched_at")?,
    ))
}

async fn count_dead_letter_attention(db: &TestDb, tenant: TenantId, event_id: Uuid) -> Result<i64> {
    let mut tx = db::begin_system(&db.app_pool, tenant).await?;
    let count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM public.attention_items \
         WHERE tenant_id = $1 AND dedupe_key = $2 AND kind = 'outbox_dead_letter' \
           AND severity = 'high' AND payload = jsonb_build_object('event_id', $3::text)",
    )
    .bind(tenant.0)
    .bind(format!("outbox_dead_letter:{event_id}"))
    .bind(event_id)
    .fetch_one(tx.conn())
    .await?;
    tx.rollback().await?;
    Ok(count)
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
    failures_before_success: usize,
    permanent: bool,
}

impl Clone for RecordingHandler {
    fn clone(&self) -> Self {
        Self {
            handler_id: self.handler_id,
            module: self.module,
            subscriptions: self.subscriptions,
            calls: Arc::clone(&self.calls),
            seen: Arc::clone(&self.seen),
            failures_before_success: self.failures_before_success,
            permanent: self.permanent,
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
            failures_before_success: 0,
            permanent: false,
        }
    }

    fn failing(mut self) -> Self {
        self.failures_before_success = usize::MAX;
        self
    }

    fn fails_before_success(mut self, count: usize) -> Self {
        self.failures_before_success = count;
        self
    }

    fn permanently_failing_before_success(mut self, count: usize) -> Self {
        self.failures_before_success = count;
        self.permanent = true;
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

    async fn handle(&self, tx: &mut db::Tx, event: &Event) -> Result<(), EventHandlerError> {
        let call = self.calls.fetch_add(1, Ordering::SeqCst) + 1;
        let tenant_id = sqlx::query_scalar::<_, String>("SELECT current_setting('app.tenant_id', true)")
            .fetch_one(tx.conn())
            .await
            .map_err(anyhow::Error::from)?;
        self.seen
            .lock()
            .map_err(|_| anyhow::anyhow!("test observation mutex poisoned"))?
            .push((event.id, tenant_id));
        if call <= self.failures_before_success {
            let error = anyhow::anyhow!("test-secret-handler-failure");
            return Err(if self.permanent {
                EventHandlerError::Permanent(error)
            } else {
                EventHandlerError::Retryable(error)
            });
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

#[derive(Clone, Debug)]
struct ManualTestClock(Arc<Mutex<DateTime<Utc>>>);

impl ManualTestClock {
    fn new(at: DateTime<Utc>) -> Self {
        Self(Arc::new(Mutex::new(at)))
    }

    fn advance(&self, duration: ChronoDuration) {
        let mut now = self.0.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(next) = now.checked_add_signed(duration) {
            *now = next;
        }
    }
}

impl Clock for ManualTestClock {
    fn now(&self) -> eq_core::clock::Timestamp {
        *self.0.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

fn dispatcher_with_clock(
    db: &TestDb,
    handler: RecordingHandler,
    enabled: bool,
    clock: Arc<dyn Clock>,
) -> Dispatcher {
    Dispatcher::new_with_clock(
        db.app_pool.clone(),
        vec![Box::new(handler)],
        Arc::new(TestModuleGate { enabled }),
        clock,
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

    async fn handle(&self, tx: &mut db::Tx, event: &Event) -> Result<(), EventHandlerError> {
        if event.id == self.blocked_event {
            self.started.notify_one();
            self.release.notified().await;
        }
        let tenant_id = sqlx::query_scalar::<_, String>("SELECT current_setting('app.tenant_id', true)")
            .fetch_one(tx.conn())
            .await
            .map_err(anyhow::Error::from)?;
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
    tokio::time::timeout(Duration::from_secs(5), started.notified()).await?;

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
    let event_id = Uuid::from_u128(0x400);
    insert_event(&db, event_id, tenant, EVENT_TYPE, None).await?;

    let handler = RecordingHandler::new("test.outbox.failure", &[EVENT_TYPE]).failing();
    let calls = Arc::clone(&handler.calls);
    let dispatcher = dispatcher(&db, handler, true);
    assert_eq!(dispatcher.drain_once().await?, 1);

    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(count_handler_receipts(&db, "test.outbox.failure").await?, 0);
    assert_eq!(count_dispatched(&db).await?, 0);
    let (attempts, last_error, dispatched_at) = outbox_attempt_state(&db, event_id).await?;
    assert_eq!(attempts, 1);
    assert!(
        last_error
            .as_ref()
            .is_some_and(|marker| marker.starts_with("ritma-retry-v1:"))
    );
    assert!(!last_error.is_some_and(|marker| marker.contains("test-secret-handler-failure")));
    assert_eq!(dispatched_at, None);
    Ok(())
}

#[tokio::test]
async fn retry_then_success() -> Result<()> {
    let (db, tenant) = fixture().await?;
    let event_id = Uuid::from_u128(0x401);
    insert_event(&db, event_id, tenant, EVENT_TYPE, None).await?;

    let handler = RecordingHandler::new("test.outbox.retry", &[EVENT_TYPE]).fails_before_success(2);
    let calls = Arc::clone(&handler.calls);
    let clock = ManualTestClock::new(test_time());
    let dispatcher = dispatcher_with_clock(&db, handler, true, Arc::new(clock.clone()));

    assert_eq!(dispatcher.drain_once().await?, 1);
    assert_eq!(dispatcher.drain_once().await?, 0);
    clock.advance(ChronoDuration::seconds(1));
    assert_eq!(dispatcher.drain_once().await?, 1);
    assert_eq!(dispatcher.drain_once().await?, 0);
    clock.advance(ChronoDuration::seconds(5));
    assert_eq!(dispatcher.drain_once().await?, 1);

    assert_eq!(calls.load(Ordering::SeqCst), 3);
    let (attempts, last_error, dispatched_at) = outbox_attempt_state(&db, event_id).await?;
    assert_eq!(attempts, 2);
    assert_eq!(last_error, None);
    assert!(dispatched_at.is_some());
    assert_eq!(
        dispatcher.counters(),
        eq_core::outbox::DispatcherCounters {
            attempted: 3,
            dispatched: 1,
            failed: 2,
            retry_scheduled: 2,
            dead_lettered: 0,
        }
    );
    Ok(())
}

#[tokio::test]
async fn dead_letter_after_max() -> Result<()> {
    let (db, tenant) = fixture().await?;
    let event_id = Uuid::from_u128(0x402);
    insert_event(&db, event_id, tenant, EVENT_TYPE, None).await?;

    let handler = RecordingHandler::new("test.outbox.dead", &[EVENT_TYPE]).failing();
    let calls = Arc::clone(&handler.calls);
    let clock = ManualTestClock::new(test_time());
    let dispatcher = dispatcher_with_clock(&db, handler, true, Arc::new(clock.clone()));
    let backoff = [1_i64, 5, 30, 300];
    for delay in backoff {
        assert_eq!(dispatcher.drain_once().await?, 1);
        clock.advance(ChronoDuration::seconds(delay));
    }
    assert_eq!(dispatcher.drain_once().await?, 1);
    assert_eq!(dispatcher.drain_once().await?, 0);

    assert_eq!(calls.load(Ordering::SeqCst), 5);
    let (attempts, last_error, dispatched_at) = outbox_attempt_state(&db, event_id).await?;
    assert_eq!(attempts, 5);
    assert_eq!(last_error.as_deref(), Some("ritma-dead-v1:handler_failed"));
    assert_eq!(dispatched_at, None);
    assert_eq!(count_dead_letter_attention(&db, tenant, event_id).await?, 1);
    assert_eq!(
        dispatcher.counters(),
        eq_core::outbox::DispatcherCounters {
            attempted: 5,
            dispatched: 0,
            failed: 5,
            retry_scheduled: 4,
            dead_lettered: 1,
        }
    );
    Ok(())
}

#[tokio::test]
async fn permanent_handler_failure_dead_letters_without_retry() -> Result<()> {
    let (db, tenant) = fixture().await?;
    let event_id = Uuid::from_u128(0x403);
    let following_event_id = Uuid::from_u128(0x404);
    let aggregate_id = Uuid::from_u128(0x405);
    insert_event(&db, event_id, tenant, EVENT_TYPE, Some(aggregate_id)).await?;
    insert_event(&db, following_event_id, tenant, EVENT_TYPE, Some(aggregate_id)).await?;

    let handler =
        RecordingHandler::new("test.outbox.permanent", &[EVENT_TYPE]).permanently_failing_before_success(1);
    let calls = Arc::clone(&handler.calls);
    let dispatcher = dispatcher(&db, handler, true);
    assert_eq!(dispatcher.drain_once().await?, 1);
    assert_eq!(dispatcher.drain_once().await?, 1);
    assert_eq!(dispatcher.drain_once().await?, 0);

    assert_eq!(calls.load(Ordering::SeqCst), 2);
    let (attempts, last_error, _) = outbox_attempt_state(&db, event_id).await?;
    assert_eq!(attempts, 1);
    assert_eq!(last_error.as_deref(), Some("ritma-dead-v1:handler_failed"));
    assert_eq!(count_dead_letter_attention(&db, tenant, event_id).await?, 1);
    let (_, following_last_error, following_dispatched_at) =
        outbox_attempt_state(&db, following_event_id).await?;
    assert_eq!(following_last_error, None);
    assert!(following_dispatched_at.is_some());
    assert_eq!(
        dispatcher.counters(),
        eq_core::outbox::DispatcherCounters {
            attempted: 2,
            dispatched: 1,
            failed: 1,
            retry_scheduled: 0,
            dead_lettered: 1,
        }
    );
    Ok(())
}

#[tokio::test]
async fn unknown_failure_marker_fails_closed() -> Result<()> {
    let (db, tenant) = fixture().await?;
    let event_id = Uuid::from_u128(0x405);
    insert_event(&db, event_id, tenant, EVENT_TYPE, None).await?;
    sqlx::query("UPDATE public.outbox SET last_error = $2 WHERE id = $1")
        .bind(event_id)
        .bind("unknown-marker-v0")
        .execute(&db.pool)
        .await?;

    let handler = RecordingHandler::new("test.outbox.unknown-marker", &[EVENT_TYPE]);
    let calls = Arc::clone(&handler.calls);
    let dispatcher = dispatcher(&db, handler, true);
    assert_eq!(dispatcher.drain_once().await?, 0);
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    let (attempts, last_error, dispatched_at) = outbox_attempt_state(&db, event_id).await?;
    assert_eq!(attempts, 0);
    assert_eq!(last_error.as_deref(), Some("unknown-marker-v0"));
    assert_eq!(dispatched_at, None);
    Ok(())
}

#[tokio::test]
async fn run_wakes_after_commit_and_stops_on_cancel() -> Result<()> {
    let (db, tenant) = fixture().await?;
    let handler = RecordingHandler::new("test.outbox.run", &[EVENT_TYPE]);
    let calls = Arc::clone(&handler.calls);
    let dispatcher = Arc::new(dispatcher(&db, handler, true));
    let shutdown = CancellationToken::new();
    let run_dispatcher = Arc::clone(&dispatcher);
    let run_shutdown = shutdown.clone();
    let run = tokio::spawn(async move { run_dispatcher.run(run_shutdown).await });
    tokio::time::sleep(Duration::from_millis(100)).await;

    insert_event(&db, Uuid::from_u128(0x404), tenant, EVENT_TYPE, None).await?;
    tokio::time::timeout(Duration::from_secs(5), async {
        while calls.load(Ordering::SeqCst) == 0 {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await?;
    shutdown.cancel();
    tokio::time::timeout(Duration::from_secs(5), run).await???;
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    Ok(())
}

#[test]
fn handler_error_debug_redacts_source_text() {
    let error = EventHandlerError::Retryable(anyhow::anyhow!("test-secret-handler-failure"));
    let debug = format!("{error:?}");
    assert!(!debug.contains("test-secret-handler-failure"));
    assert!(!error.to_string().contains("test-secret-handler-failure"));
}
