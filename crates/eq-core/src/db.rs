//! PostgreSQL connection and tenant transaction helpers (ARC-04, ARC-06, SEC-05).

use crate::actor::{ActorCtx, ActorRole};
use crate::config::Config;
use crate::ids::TenantId;
use anyhow::Context;
use secrecy::ExposeSecret;
use sqlx::migrate::Migrator;
use sqlx::pool::PoolConnection;
use sqlx::postgres::{PgAdvisoryLock, PgAdvisoryLockKey, PgConnection, PgPoolOptions};
use sqlx::{PgPool, Postgres, Transaction};

const MIGRATION_LOCK_KEY: i64 = 7_000_001;
const GRANT_MIGRATION_STATUS: &str = "DO $$ BEGIN IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'eq_app') THEN EXECUTE 'GRANT SELECT ON TABLE _sqlx_migrations TO eq_app'; END IF; END $$";
static MIGRATOR: Migrator = sqlx::migrate!("../../migrations");

/// ARC-06: creates the application pool using the least-privileged configured role.
pub async fn connect(config: &Config) -> anyhow::Result<PgPool> {
    PgPoolOptions::new()
        .max_connections(config.db_pool)
        .after_release(|connection, _metadata| {
            Box::pin(async move {
                sqlx::query!("RESET ALL")
                    .execute(connection)
                    .await
                    .map(|_| true)
                    .or(Ok(false))
            })
        })
        // SBD-50: expose the credential only at the SQLx connection boundary.
        .connect(config.database_url.expose_secret())
        .await
        .context("connect to the configured PostgreSQL database")
}

/// ADR-0012: runs the single migration set under the process-wide PostgreSQL lock.
pub async fn run_migrations(owner_url: &str) -> anyhow::Result<()> {
    let pool = PgPoolOptions::new()
        .connect(owner_url)
        .await
        .context("connect as the migration owner")?;
    let lock = PgAdvisoryLock::with_key(PgAdvisoryLockKey::BigInt(MIGRATION_LOCK_KEY));
    let connection: PoolConnection<Postgres> = pool.acquire().await?;
    let guard = lock.acquire(connection).await?;

    let migration_result = MIGRATOR.run(&pool).await;
    let connection = guard
        .release_now()
        .await
        .context("release the migration advisory lock")?;
    drop(connection);
    if let Err(error) = migration_result {
        pool.close().await;
        return Err(error).context("apply PostgreSQL migrations");
    }

    let grant_result = sqlx::query(GRANT_MIGRATION_STATUS)
        .execute(&pool)
        .await
        .context("grant the application role read-only migration status");
    pool.close().await;
    grant_result?;
    Ok(())
}

/// ARC-04: a transaction scoped to one tenant and actor.
pub struct Tx {
    transaction: Transaction<'static, Postgres>,
    tenant: TenantId,
    actor: ActorCtx,
}

impl std::fmt::Debug for Tx {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_tuple("Tx").field(&"<transaction>").finish()
    }
}

impl Tx {
    /// ARC-04: returns the active transaction connection without exposing deref access.
    pub fn conn(&mut self) -> &mut PgConnection {
        &mut self.transaction
    }

    /// ARC-04: returns the tenant captured when the transaction began.
    pub(crate) fn tenant(&self) -> TenantId {
        self.tenant
    }

    /// ARC-04: returns the actor captured when the transaction began.
    pub(crate) fn actor(&self) -> &ActorCtx {
        &self.actor
    }

    /// ARC-04: commits the transaction.
    pub async fn commit(self) -> Result<(), sqlx::Error> {
        self.transaction.commit().await
    }

    /// ARC-04: rolls the transaction back.
    pub async fn rollback(self) -> Result<(), sqlx::Error> {
        self.transaction.rollback().await
    }
}

/// SEC-05: sets transaction-local tenant and actor context in one round trip.
pub async fn begin(pool: &PgPool, tenant: TenantId, actor: &ActorCtx) -> Result<Tx, crate::error::AppError> {
    let mut transaction = pool.begin().await?;
    let tenant_id = tenant.0.to_string();
    let actor_id = if actor.role == ActorRole::System {
        String::new()
    } else {
        actor
            .person_id
            .map(|person_id| person_id.0.to_string())
            .unwrap_or_default()
    };

    sqlx::query!(
        "SELECT set_config('app.tenant_id', $1, true) AS tenant_id, \
                set_config('app.actor_id', $2, true) AS actor_id",
        tenant_id,
        actor_id
    )
    .fetch_one(&mut *transaction)
    .await?;

    Ok(Tx {
        transaction,
        tenant,
        actor: actor.clone(),
    })
}

/// FLW-02/SEC-05: starts a tenant-scoped read-only transaction for list and detail requests.
pub async fn begin_read_only(
    pool: &PgPool,
    tenant: TenantId,
    actor: &ActorCtx,
) -> Result<Tx, crate::error::AppError> {
    let mut transaction = pool.begin().await?;
    sqlx::query!("SET TRANSACTION READ ONLY")
        .execute(&mut *transaction)
        .await?;
    let tenant_id = tenant.0.to_string();
    let actor_id = if actor.role == ActorRole::System {
        String::new()
    } else {
        actor
            .person_id
            .map(|person_id| person_id.0.to_string())
            .unwrap_or_default()
    };
    sqlx::query!(
        "SELECT set_config('app.tenant_id', $1, true) AS tenant_id, \
                set_config('app.actor_id', $2, true) AS actor_id",
        tenant_id,
        actor_id
    )
    .fetch_one(&mut *transaction)
    .await?;

    Ok(Tx {
        transaction,
        tenant,
        actor: actor.clone(),
    })
}

/// SEC-05: starts a tenant transaction with a personless system actor.
pub async fn begin_system(pool: &PgPool, tenant: TenantId) -> Result<Tx, crate::error::AppError> {
    begin(pool, tenant, &ActorCtx::system("system")).await
}
