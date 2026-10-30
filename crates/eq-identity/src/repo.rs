//! Identity-owned credential and refresh-session persistence (IDN-06/ERD-04).

use crate::password::PasswordHash;
use chrono::{DateTime, Utc};
use eq_core::db::Tx;
use eq_core::error::AppError;
use eq_core::ids::{PersonId, TenantId};
use eq_core::model::Role;
use secrecy::SecretBox;
use std::net::IpAddr;
use uuid::Uuid;
use zeroize::Zeroizing;

/// SBD-50: credential data is intentionally not formatted or cloned.
pub(crate) struct Credentials {
    pub(crate) password_hash: Option<PasswordHash>,
    pub(crate) password_changed_at: Option<DateTime<Utc>>,
    pub(crate) totp_secret_enc: Option<SecretBox<Zeroizing<Vec<u8>>>>,
    pub(crate) totp_enabled: bool,
    pub(crate) failed_count: i32,
    pub(crate) failed_window_start: Option<DateTime<Utc>>,
    pub(crate) locked_until: Option<DateTime<Utc>>,
}

impl std::fmt::Debug for Credentials {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Credentials")
            .field("password_hash", &"[REDACTED]")
            .field("password_changed_at", &self.password_changed_at)
            .field("totp_secret_enc", &"[REDACTED]")
            .field("totp_enabled", &self.totp_enabled)
            .field("failed_count", &self.failed_count)
            .field("failed_window_start", &self.failed_window_start)
            .field("locked_until", &self.locked_until)
            .finish()
    }
}

pub(crate) struct RefreshSessionIdentity {
    person_id: PersonId,
    active_role: Role,
}

impl RefreshSessionIdentity {
    pub(crate) const fn person_id(&self) -> PersonId {
        self.person_id
    }

    pub(crate) const fn active_role(&self) -> Role {
        self.active_role
    }
}

pub(crate) struct RefreshSession {
    id: Uuid,
    person_id: PersonId,
    family_id: Uuid,
    created_at: DateTime<Utc>,
    active_role: Role,
    idle_expires_at: DateTime<Utc>,
    absolute_expires_at: DateTime<Utc>,
    revoked_at: Option<DateTime<Utc>>,
}

impl RefreshSession {
    pub(crate) const fn id(&self) -> Uuid {
        self.id
    }

    pub(crate) const fn person_id(&self) -> PersonId {
        self.person_id
    }

    pub(crate) const fn family_id(&self) -> Uuid {
        self.family_id
    }

    pub(crate) const fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub(crate) const fn active_role(&self) -> Role {
        self.active_role
    }

    pub(crate) const fn idle_expires_at(&self) -> DateTime<Utc> {
        self.idle_expires_at
    }

    pub(crate) const fn absolute_expires_at(&self) -> DateTime<Utc> {
        self.absolute_expires_at
    }

    pub(crate) const fn revoked_at(&self) -> Option<DateTime<Utc>> {
        self.revoked_at
    }
}

impl std::fmt::Debug for RefreshSessionIdentity {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RefreshSessionIdentity")
            .field("person_id", &"[REDACTED]")
            .field("active_role", &self.active_role)
            .finish()
    }
}

impl std::fmt::Debug for RefreshSession {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RefreshSession")
            .field("id", &"[REDACTED]")
            .field("person_id", &"[REDACTED]")
            .field("active_role", &self.active_role)
            .field("idle_expires_at", &self.idle_expires_at)
            .field("absolute_expires_at", &self.absolute_expires_at)
            .field("revoked_at", &self.revoked_at)
            .finish()
    }
}

/// STM-23: obtains and locks the tenant credential row before checking a password.
pub(crate) async fn credentials_for_update(
    tx: &mut Tx,
    tenant: TenantId,
    person: PersonId,
) -> Result<Option<Credentials>, AppError> {
    let row = sqlx::query!(
        "SELECT password_hash, password_changed_at, totp_secret_enc, totp_enabled, failed_count, failed_window_start, locked_until \
         FROM public.credentials WHERE tenant_id = $1 AND person_id = $2 FOR UPDATE",
        tenant.0,
        person.0
    )
    .fetch_optional(tx.conn())
    .await?;

    row.map(|row| {
        Ok(Credentials {
            password_hash: row
                .password_hash
                .map(PasswordHash::try_from)
                .transpose()
                .map_err(|error| {
                    AppError::Internal(anyhow::anyhow!("stored password hash is invalid: {error}"))
                })?,
            password_changed_at: row.password_changed_at,
            totp_secret_enc: row
                .totp_secret_enc
                .map(|ciphertext| SecretBox::new(Box::new(Zeroizing::new(ciphertext)))),
            totp_enabled: row.totp_enabled,
            failed_count: row.failed_count,
            failed_window_start: row.failed_window_start,
            locked_until: row.locked_until,
        })
    })
    .transpose()
}

/// STM-23: persists the counter and optional lock transition in the caller transaction.
pub(crate) async fn set_failures(
    tx: &mut Tx,
    tenant: TenantId,
    person: PersonId,
    failed_count: i32,
    failed_window_start: DateTime<Utc>,
    locked_until: Option<DateTime<Utc>>,
) -> Result<(), AppError> {
    let result = sqlx::query!(
        "UPDATE public.credentials SET failed_count = $3, failed_window_start = $4, locked_until = $5 \
         WHERE tenant_id = $1 AND person_id = $2",
        tenant.0,
        person.0,
        failed_count,
        failed_window_start,
        locked_until
    )
    .execute(tx.conn())
    .await?;
    if result.rows_affected() != 1 {
        return Err(AppError::Internal(anyhow::anyhow!(
            "locked identity credential row disappeared"
        )));
    }
    Ok(())
}

/// STM-23: clears counters after a successful password and required TOTP check.
pub(crate) async fn reset_failures(tx: &mut Tx, tenant: TenantId, person: PersonId) -> Result<(), AppError> {
    let result = sqlx::query!(
        "UPDATE public.credentials SET failed_count = 0, failed_window_start = NULL, locked_until = NULL \
         WHERE tenant_id = $1 AND person_id = $2",
        tenant.0,
        person.0
    )
    .execute(tx.conn())
    .await?;
    if result.rows_affected() != 1 {
        return Err(AppError::Internal(anyhow::anyhow!(
            "locked identity credential row disappeared"
        )));
    }
    Ok(())
}

/// IDN-03: reads the password-change epoch used to invalidate stale access tokens.
pub(crate) async fn password_version(
    tx: &mut Tx,
    tenant: TenantId,
    person: PersonId,
) -> Result<Option<i64>, AppError> {
    let row = sqlx::query!(
        "SELECT COALESCE(FLOOR(EXTRACT(EPOCH FROM password_changed_at))::bigint, 0) AS \"version!\" \
         FROM public.credentials WHERE tenant_id = $1 AND person_id = $2",
        tenant.0,
        person.0
    )
    .fetch_optional(tx.conn())
    .await?;
    Ok(row.map(|credentials| credentials.version))
}

/// IDN-04/ERD-04: resolves the person behind a canonical refresh digest before actor-scoped work.
pub(crate) async fn refresh_session_identity(
    tx: &mut Tx,
    tenant: TenantId,
    token_hash: &[u8],
) -> Result<Option<RefreshSessionIdentity>, AppError> {
    let row = sqlx::query!(
        "SELECT person_id, active_role AS \"active_role: Role\" \
         FROM public.refresh_sessions WHERE tenant_id = $1 AND token_hash = $2",
        tenant.0,
        token_hash
    )
    .fetch_optional(tx.conn())
    .await?;
    Ok(row.map(|row| RefreshSessionIdentity {
        person_id: PersonId(row.person_id),
        active_role: row.active_role,
    }))
}

/// STM-22: locks the matching refresh-session row before role validation or mutation.
pub(crate) async fn refresh_session_for_update(
    tx: &mut Tx,
    tenant: TenantId,
    token_hash: &[u8],
) -> Result<Option<RefreshSession>, AppError> {
    let row = sqlx::query!(
        "SELECT id, person_id, family_id, created_at, active_role AS \"active_role: Role\", idle_expires_at, \
                absolute_expires_at, revoked_at \
         FROM public.refresh_sessions \
         WHERE tenant_id = $1 AND token_hash = $2 FOR UPDATE",
        tenant.0,
        token_hash
    )
    .fetch_optional(tx.conn())
    .await?;
    Ok(row.map(|row| RefreshSession {
        id: row.id,
        person_id: PersonId(row.person_id),
        family_id: row.family_id,
        created_at: row.created_at,
        active_role: row.active_role,
        idle_expires_at: row.idle_expires_at,
        absolute_expires_at: row.absolute_expires_at,
        revoked_at: row.revoked_at,
    }))
}

/// IDN-04/STM-22: detects whether a revoked row has a successor in its family.
pub(crate) async fn refresh_family_has_successor(
    tx: &mut Tx,
    tenant: TenantId,
    family_id: Uuid,
    created_at: DateTime<Utc>,
) -> Result<bool, AppError> {
    let row = sqlx::query!(
        "SELECT EXISTS (SELECT 1 FROM public.refresh_sessions \
         WHERE tenant_id = $1 AND family_id = $2 AND created_at > $3) AS \"exists!\"",
        tenant.0,
        family_id,
        created_at
    )
    .fetch_one(tx.conn())
    .await?;
    Ok(row.exists)
}

/// IDN-04/STM-22: marks the predecessor revoked when its successor is inserted.
pub(crate) async fn revoke_refresh_session(
    tx: &mut Tx,
    tenant: TenantId,
    session_id: Uuid,
    token_hash: &[u8],
    now: DateTime<Utc>,
) -> Result<(), AppError> {
    let result = sqlx::query!(
        "UPDATE public.refresh_sessions SET revoked_at = $4, last_used_at = $4 \
         WHERE tenant_id = $1 AND id = $2 AND token_hash = $3 AND revoked_at IS NULL",
        tenant.0,
        session_id,
        token_hash,
        now
    )
    .execute(tx.conn())
    .await?;
    if result.rows_affected() != 1 {
        return Err(AppError::Internal(anyhow::anyhow!(
            "active refresh session disappeared during rotation"
        )));
    }
    Ok(())
}

/// IDN-04/STM-22: revokes every row in a replayed refresh-token family.
pub(crate) async fn revoke_refresh_family(
    tx: &mut Tx,
    tenant: TenantId,
    family_id: Uuid,
    now: DateTime<Utc>,
) -> Result<(), AppError> {
    let result = sqlx::query!(
        "UPDATE public.refresh_sessions SET revoked_at = COALESCE(revoked_at, $3) \
         WHERE tenant_id = $1 AND family_id = $2",
        tenant.0,
        family_id,
        now
    )
    .execute(tx.conn())
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::Internal(anyhow::anyhow!(
            "refresh-token family disappeared during reuse handling"
        )));
    }
    Ok(())
}

/// IDN-09: changes only the active role on the already-locked refresh-session row.
pub(crate) async fn set_refresh_session_role(
    tx: &mut Tx,
    tenant: TenantId,
    session_id: Uuid,
    token_hash: &[u8],
    active_role: Role,
) -> Result<(), AppError> {
    let role = role_name(active_role);
    let result = sqlx::query!(
        "UPDATE public.refresh_sessions SET active_role = $4 \
         WHERE tenant_id = $1 AND id = $2 AND token_hash = $3",
        tenant.0,
        session_id,
        token_hash,
        role
    )
    .execute(tx.conn())
    .await?;
    if result.rows_affected() != 1 {
        return Err(AppError::Internal(anyhow::anyhow!(
            "locked refresh session disappeared before role switch"
        )));
    }
    Ok(())
}

/// IDN-04: stores only the SHA-256 digest of the opaque refresh token.
#[expect(
    clippy::too_many_arguments,
    reason = "these columns are the complete mapped refresh_sessions insert contract"
)]
pub(crate) async fn create_refresh_session(
    tx: &mut Tx,
    id: Uuid,
    tenant: TenantId,
    person: PersonId,
    family_id: Uuid,
    token_hash: Vec<u8>,
    active_role: Role,
    idle_expires_at: DateTime<Utc>,
    absolute_expires_at: DateTime<Utc>,
    user_agent: Option<&str>,
    ip: Option<IpAddr>,
) -> Result<(), AppError> {
    let role = role_name(active_role);
    let ip = ip.map(|address| address.to_string());
    sqlx::query!(
        "INSERT INTO public.refresh_sessions \
         (id, tenant_id, person_id, family_id, token_hash, active_role, idle_expires_at, absolute_expires_at, user_agent, ip) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10::text::inet)",
        id,
        tenant.0,
        person.0,
        family_id,
        token_hash,
        role,
        idle_expires_at,
        absolute_expires_at,
        user_agent,
        ip
    )
    .execute(tx.conn())
    .await?;
    Ok(())
}

fn role_name(role: Role) -> &'static str {
    match role {
        Role::Owner => "owner",
        Role::Admin => "admin",
        Role::Instructor => "instructor",
        Role::Guardian => "guardian",
        Role::Student => "student",
    }
}

/// STM-21: serializes invite issuance for the same tenant/person/role before revoking prior tokens.
pub(crate) async fn lock_invite_series(
    tx: &mut Tx,
    tenant: TenantId,
    person: PersonId,
    role: Role,
) -> Result<(), AppError> {
    let key = format!("{}:{}:{}", tenant.0, person.0, role_name(role));
    sqlx::query!(
        "SELECT pg_advisory_xact_lock(hashtextextended($1, 0)) IS NULL AS \"locked!\"",
        key
    )
    .fetch_one(tx.conn())
    .await?;
    Ok(())
}

/// STM-21: expires earlier pending invites for one tenant/person/role.
pub(crate) async fn expire_pending_invites(
    tx: &mut Tx,
    tenant: TenantId,
    person: PersonId,
    role: Role,
    now: DateTime<Utc>,
) -> Result<(), AppError> {
    sqlx::query!(
        "UPDATE public.invites SET expires_at = $4 \
         WHERE tenant_id = $1 AND person_id = $2 AND role = $3 \
           AND accepted_at IS NULL AND expires_at > $4",
        tenant.0,
        person.0,
        role_name(role),
        now
    )
    .execute(tx.conn())
    .await?;
    Ok(())
}

/// IDN-07/ERD-04: stores the invite ID, SHA-256 token digest, and policy-derived expiry.
#[expect(
    clippy::too_many_arguments,
    reason = "this repository insert binds each column in the invite DDL explicitly"
)]
pub(crate) async fn insert_invite(
    tx: &mut Tx,
    id: Uuid,
    tenant: TenantId,
    person: PersonId,
    role: Role,
    token_hash: &[u8],
    expires_at: DateTime<Utc>,
    created_by: Option<PersonId>,
) -> Result<(), AppError> {
    sqlx::query!(
        "INSERT INTO public.invites (id, tenant_id, person_id, role, token_hash, expires_at, created_by) \
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
        id,
        tenant.0,
        person.0,
        role_name(role),
        token_hash,
        expires_at,
        created_by.map(|person| person.0)
    )
    .execute(tx.conn())
    .await?;
    Ok(())
}
