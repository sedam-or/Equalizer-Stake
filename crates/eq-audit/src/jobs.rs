//! Tenant-scoped audit maintenance jobs (AUD-04..07, ARC-09).

use crate::Sealer;
use crate::policy::AuditPolicy;
use crate::repo;
use anyhow::anyhow;
use chrono::{Months, TimeDelta};
use eq_core::db;
use eq_core::error::AppError;
use eq_core::ids::TenantId;
use eq_core::jobs::{JobCtx, JobError, JobHandler};
use eq_core::model::Severity;
use eq_core::module::ModuleId;
use serde_json::{Value, json};
use sqlx::PgPool;

/// AUD-07: anonymizes and purges entries using the tenant's versioned audit policy.
#[derive(Debug, Default)]
pub struct AuditRetentionJob;

#[async_trait::async_trait]
impl JobHandler for AuditRetentionJob {
    fn kind(&self) -> &'static str {
        "audit.retention"
    }

    fn module(&self) -> ModuleId {
        ModuleId::Audit
    }

    async fn run(&self, ctx: &JobCtx, _payload: &Value) -> Result<(), JobError> {
        let tenant = ctx
            .tenant
            .as_ref()
            .map(|tenant| tenant.id)
            .ok_or_else(|| JobError::Permanent(anyhow!("audit retention requires a tenant")))?;
        let policy_store = ctx
            .policy_store
            .as_ref()
            .ok_or_else(|| JobError::Permanent(anyhow!("audit retention policy service is unavailable")))?;

        let mut tx = db::begin_system(&ctx.pool, tenant).await.map_err(retryable)?;
        let policy = policy_store
            .get::<AuditPolicy>(&mut tx, tenant, ctx.now)
            .await
            .map_err(retryable)?
            .doc;
        let anonymize_months = u32::from(policy.anonymize_pii_after_years()) * 12;
        let retention_months = u32::from(policy.retention_years()) * 12;
        let anonymize_before = ctx
            .now
            .checked_sub_months(Months::new(anonymize_months))
            .ok_or_else(|| JobError::Permanent(anyhow!("audit anonymization cutoff is out of range")))?;
        let purge_before = ctx
            .now
            .checked_sub_months(Months::new(retention_months))
            .ok_or_else(|| JobError::Permanent(anyhow!("audit retention cutoff is out of range")))?;

        repo::anonymize(&mut tx, tenant, anonymize_before)
            .await
            .map_err(retryable)?;
        repo::purge_expired(&mut tx, tenant, purge_before)
            .await
            .map_err(retryable)?;
        tx.commit().await.map_err(retryable)
    }
}

/// AUD-04: checks the last 24 hours of one tenant's audit chain and raises attention on mismatch.
#[derive(Debug, Default)]
pub struct AuditVerifyRecentJob;

#[async_trait::async_trait]
impl JobHandler for AuditVerifyRecentJob {
    fn kind(&self) -> &'static str {
        "audit.verify_recent"
    }

    fn module(&self) -> ModuleId {
        ModuleId::Audit
    }

    async fn run(&self, ctx: &JobCtx, _payload: &Value) -> Result<(), JobError> {
        let tenant = ctx
            .tenant
            .as_ref()
            .map(|tenant| tenant.id)
            .ok_or_else(|| JobError::Permanent(anyhow!("audit verification requires a tenant")))?;
        let from_at = ctx
            .now
            .checked_sub_signed(TimeDelta::hours(24))
            .ok_or_else(|| JobError::Permanent(anyhow!("audit verification window is out of range")))?;

        let mut tx = db::begin_system(&ctx.pool, tenant).await.map_err(retryable)?;
        let bounds = repo::recent_sequence_bounds(&mut tx, tenant, from_at, ctx.now)
            .await
            .map_err(retryable)?;
        tx.commit().await.map_err(retryable)?;

        let (Some(first_seq), Some(last_seq)) = (bounds.first_seq, bounds.last_seq) else {
            return Ok(());
        };
        let report = Sealer::new(ctx.pool.clone())
            .verify(tenant, first_seq, last_seq)
            .await
            .map_err(retryable)?;
        if let Some(sequence) = report.first_mismatch {
            record_mismatch(&ctx.pool, tenant, sequence)
                .await
                .map_err(retryable)?;
        }
        Ok(())
    }
}

async fn record_mismatch(pool: &PgPool, tenant: TenantId, sequence: i64) -> Result<(), AppError> {
    let mut tx = db::begin_system(pool, tenant).await?;
    let dedupe_key = format!("audit_chain_mismatch:{sequence}");
    eq_core::attention::raise(
        &mut tx,
        ModuleId::Audit,
        "audit_chain_mismatch",
        json!({ "sequence": sequence }),
        Severity::High,
        &dedupe_key,
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

fn retryable(error: impl Into<anyhow::Error>) -> JobError {
    JobError::Retryable(error.into())
}
