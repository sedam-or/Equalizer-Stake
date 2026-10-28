//! Parameterized persistence for the core-owned job and cron state tables (ARC-09).

use crate::clock::Timestamp;
use crate::db::{self, Tx};
use crate::error::AppError;
use crate::ids::TenantId;
use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug)]
pub(super) struct ClaimedJob {
    pub id: Uuid,
    pub tenant_id: Option<TenantId>,
    pub kind: String,
    pub payload: Value,
    pub attempts: i32,
    pub max_attempts: i32,
}

pub(super) async fn enqueue(
    tx: &mut Tx,
    tenant: Option<TenantId>,
    kind: &str,
    payload: Value,
    run_at: Timestamp,
    dedupe_key: Option<String>,
) -> Result<Uuid, AppError> {
    let id = Uuid::now_v7();
    let row = sqlx::query!(
        "INSERT INTO public.jobs (id, tenant_id, kind, payload, run_at, dedupe_key) \
         VALUES ($1, $2, $3, $4, $5, $6) \
         ON CONFLICT (dedupe_key) DO UPDATE SET dedupe_key = EXCLUDED.dedupe_key \
         RETURNING id",
        id,
        tenant.map(|id| id.0),
        kind,
        payload,
        run_at,
        dedupe_key
    )
    .fetch_one(tx.conn())
    .await?;
    Ok(row.id)
}

pub(super) async fn reap_expired(pool: &PgPool, now: Timestamp) -> Result<(), AppError> {
    sqlx::query!(
        "UPDATE public.jobs SET status = 'queued', locked_until = NULL \
         WHERE status = 'running' AND locked_until <= $1",
        now
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub(super) async fn claim_due(
    pool: &PgPool,
    tenants: &[TenantId],
    now: Timestamp,
    limit: i64,
    lease_seconds: i64,
) -> Result<Vec<ClaimedJob>, AppError> {
    let tenant_ids = tenants.iter().map(|tenant| tenant.0).collect::<Vec<_>>();
    let rows = sqlx::query!(
        "WITH candidates AS ( \
           SELECT id FROM public.jobs \
           WHERE status = 'queued' AND run_at <= $1 \
             AND (tenant_id IS NULL OR tenant_id = ANY($2)) \
           ORDER BY run_at, id \
           FOR UPDATE SKIP LOCKED LIMIT $3 \
         ) \
         UPDATE public.jobs AS job \
         SET status = 'running', locked_until = $1 + make_interval(secs => $4::double precision), \
             attempts = job.attempts + 1 \
         FROM candidates WHERE job.id = candidates.id \
         RETURNING job.id, job.tenant_id, job.kind, job.payload, job.attempts, job.max_attempts",
        now,
        &tenant_ids,
        limit,
        lease_seconds as f64
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| ClaimedJob {
            id: row.id,
            tenant_id: row.tenant_id.map(TenantId),
            kind: row.kind,
            payload: row.payload,
            attempts: row.attempts,
            max_attempts: row.max_attempts,
        })
        .collect())
}

pub(super) async fn mark_done(
    pool: &PgPool,
    id: Uuid,
    now: Timestamp,
    last_error: Option<&str>,
) -> Result<(), AppError> {
    sqlx::query!(
        "UPDATE public.jobs SET status = 'done', locked_until = NULL, finished_at = $2, last_error = $3 \
         WHERE id = $1 AND status = 'running'",
        id,
        now,
        last_error
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub(super) async fn mark_dead(
    pool: &PgPool,
    job: &ClaimedJob,
    now: Timestamp,
    last_error: &str,
) -> Result<(), AppError> {
    if let Some(tenant) = job.tenant_id {
        let mut tx = db::begin_system(pool, tenant).await?;
        sqlx::query!(
            "UPDATE public.jobs SET status = 'dead', locked_until = NULL, finished_at = $2, last_error = $3 \
             WHERE id = $1 AND status = 'running'",
            job.id,
            now,
            last_error
        )
        .execute(tx.conn())
        .await?;
        let payload = json!({ "job_id": job.id });
        let dedupe_key = format!(
            "{}{job_id}",
            super::JOB_DEAD_LETTER_DEDUPE_PREFIX,
            job_id = job.id
        );
        sqlx::query!(
            "INSERT INTO public.attention_items \
                 (id, tenant_id, module_id, kind, ref_type, ref_id, payload, severity, dedupe_key) \
             VALUES ($1, $2, 'core', $3, 'job', $4, $5, 'high', $6) \
             ON CONFLICT (tenant_id, dedupe_key) DO UPDATE SET \
                 payload = EXCLUDED.payload, severity = EXCLUDED.severity, resolved_at = NULL, resolved_by = NULL",
            Uuid::now_v7(),
            tenant.0,
            super::JOB_DEAD_LETTER_KIND,
            job.id,
            payload,
            dedupe_key
        )
        .execute(tx.conn())
        .await?;
        tx.commit().await?;
    } else {
        sqlx::query!(
            "UPDATE public.jobs SET status = 'dead', locked_until = NULL, finished_at = $2, last_error = $3 \
             WHERE id = $1 AND status = 'running'",
            job.id,
            now,
            last_error
        )
        .execute(pool)
        .await?;
    }
    Ok(())
}

pub(super) async fn mark_retry(
    pool: &PgPool,
    job: &ClaimedJob,
    now: Timestamp,
    last_error: &str,
) -> Result<(), AppError> {
    let exponent = u32::try_from(job.attempts.saturating_sub(1))
        .unwrap_or(62)
        .min(62);
    let backoff_seconds =
        super::JOB_RETRY_BASE_SECONDS.saturating_mul(1_i64.checked_shl(exponent).unwrap_or(i64::MAX));
    sqlx::query!(
        "UPDATE public.jobs \
         SET status = 'queued', locked_until = NULL, run_at = $2::timestamptz + make_interval(secs => $3::double precision), last_error = $4 \
         WHERE id = $1 AND status = 'running'",
        job.id,
        now,
        backoff_seconds as f64,
        last_error
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub(super) async fn cron_last_fire(
    tx: &mut Tx,
    tenant: TenantId,
    kind: &str,
) -> Result<Option<Timestamp>, AppError> {
    Ok(sqlx::query_scalar!(
        "SELECT last_fire_at FROM public.cron_state WHERE tenant_id = $1 AND kind = $2",
        tenant.0,
        kind
    )
    .fetch_optional(tx.conn())
    .await?)
}

pub(super) async fn enqueue_cron(
    tx: &mut Tx,
    tenant: TenantId,
    kind: &str,
    run_at: Timestamp,
    dedupe_key: &str,
) -> Result<bool, AppError> {
    let id = Uuid::now_v7();
    let row = sqlx::query!(
        "INSERT INTO public.jobs (id, tenant_id, kind, payload, run_at, dedupe_key) \
         VALUES ($1, $2, $3, '{}'::jsonb, $4, $5) \
         ON CONFLICT (dedupe_key) DO NOTHING RETURNING id",
        id,
        tenant.0,
        kind,
        run_at,
        dedupe_key
    )
    .fetch_optional(tx.conn())
    .await?;
    Ok(row.is_some())
}

pub(super) async fn upsert_cron_state(
    tx: &mut Tx,
    tenant: TenantId,
    kind: &str,
    last_tick: Timestamp,
) -> Result<(), AppError> {
    sqlx::query!(
        "INSERT INTO public.cron_state (tenant_id, kind, last_fire_at) \
         VALUES ($1, $2, $3) \
         ON CONFLICT (tenant_id, kind) DO UPDATE \
         SET last_fire_at = GREATEST(public.cron_state.last_fire_at, EXCLUDED.last_fire_at)",
        tenant.0,
        kind,
        last_tick
    )
    .execute(tx.conn())
    .await?;
    Ok(())
}
