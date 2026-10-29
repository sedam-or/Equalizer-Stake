//! Tenant-scoped login and lockout transitions (IDN-01/03/06/12, STM-23).

use crate::errors::codes;
use crate::jwt::{AccessToken, AccessTokenSubject, issue_access_token};
use crate::password::{
    PasswordError, PasswordHash, create_dummy_password_hash, verify_dummy_password, verify_password,
};
use crate::policy::{IdentityPolicy, TotpPolicy};
use crate::repo;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{DateTime, Duration, Utc};
use eq_audit::{AuditEvent, Category, record};
use eq_core::actor::{ActorCtx, ActorRole};
use eq_core::auth::{CoreIdentityDirectory, LoginIdentity};
use eq_core::clock::Clock;
use eq_core::crypto::{Aad, CryptoError, Keyring, Keys};
use eq_core::db::Tx;
use eq_core::error::AppError;
use eq_core::fields::{Email, FieldCtx, FieldKind, Password};
use eq_core::ids::{PersonId, TenantId};
use eq_core::model::Role;
use eq_core::outbox::{DomainEvent, emit};
use eq_core::policy::PolicyStore;
use rand::RngCore;
use rand::rngs::OsRng;
use secrecy::{ExposeSecret, SecretString};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fmt;
use std::net::IpAddr;
use std::sync::OnceLock;
use totp_rs::{Algorithm, Builder, Totp};
use uuid::Uuid;
use zeroize::Zeroizing;

static DUMMY_PASSWORD_HASH: OnceLock<Result<PasswordHash, PasswordError>> = OnceLock::new();

/// IDN-06/CA-30: login dependencies, with core identity lookup injected through its port.
pub struct LoginService<'a> {
    directory: &'a dyn CoreIdentityDirectory,
    policies: &'a PolicyStore,
    keys: &'a Keys,
    keyring: &'a Keyring,
    clock: &'a dyn Clock,
}

impl fmt::Debug for LoginService<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LoginService")
            .field("directory", &"[core identity port]")
            .field("policies", &"[tenant policy store]")
            .field("keys", &"[REDACTED]")
            .field("keyring", &"[REDACTED]")
            .field("clock", &self.clock)
            .finish()
    }
}

impl<'a> LoginService<'a> {
    /// IDN-06/CA-30: builds a login service from process-owned cryptography and ports.
    pub fn new(
        directory: &'a dyn CoreIdentityDirectory,
        policies: &'a PolicyStore,
        keys: &'a Keys,
        keyring: &'a Keyring,
        clock: &'a dyn Clock,
    ) -> Self {
        Self {
            directory,
            policies,
            keys,
            keyring,
            clock,
        }
    }

    /// IDN-01/03/06/12: verifies one login and writes all successful or lockout effects in tx.
    pub async fn login(&self, tx: &mut Tx, request: LoginRequest<'_>) -> Result<LoginSuccess, AppError> {
        let now = self.clock.now();
        let policy = self
            .policies
            .current::<IdentityPolicy>(tx, request.tenant)
            .await?
            .doc;
        let Ok(email) = Email::parse(request.email.to_owned(), request.field_ctx) else {
            verify_dummy(request.password)?;
            return reject_login(tx, &request, None).await;
        };
        let Some(identity) = self.directory.by_email(tx, &email).await? else {
            verify_dummy(request.password)?;
            return reject_login(tx, &request, None).await;
        };
        let person = identity.person_id();
        let Some(credentials) = repo::credentials_for_update(tx, request.tenant, person).await? else {
            verify_dummy(request.password)?;
            return reject_login(tx, &request, Some(person)).await;
        };
        self.finish_login(tx, &request, identity, credentials, &policy, now)
            .await
    }

    async fn finish_login(
        &self,
        tx: &mut Tx,
        request: &LoginRequest<'_>,
        identity: LoginIdentity,
        credentials: repo::Credentials,
        policy: &IdentityPolicy,
        now: DateTime<Utc>,
    ) -> Result<LoginSuccess, AppError> {
        let person = identity.person_id();
        let Some(password_hash) = credentials.password_hash.as_ref() else {
            verify_dummy(request.password)?;
            return reject_login(tx, request, Some(person)).await;
        };
        let password_matches = verify_password(request.password, password_hash)
            .map_err(|error| AppError::Internal(anyhow::anyhow!("password verification failed: {error}")))?;

        if let Some(until) = credentials.locked_until.filter(|until| *until > now) {
            if password_matches {
                return Err(AppError::Locked { until: Some(until) });
            }
            audit_login_failure(tx, request, Some(person)).await?;
            return Err(invalid_credentials());
        }

        if !password_matches {
            return self
                .record_password_failure(tx, request, policy, &credentials, person, now)
                .await;
        }

        self.complete_login(tx, request, &identity, &credentials, policy, now)
            .await
    }

    async fn complete_login(
        &self,
        tx: &mut Tx,
        request: &LoginRequest<'_>,
        identity: &LoginIdentity,
        credentials: &repo::Credentials,
        policy: &IdentityPolicy,
        now: DateTime<Utc>,
    ) -> Result<LoginSuccess, AppError> {
        if credentials.totp_enabled {
            self.verify_totp(tx, request, identity, credentials, now).await?;
        }

        self.create_login_session(tx, request, identity, credentials, policy, now)
            .await
    }

    async fn verify_totp(
        &self,
        tx: &mut Tx,
        request: &LoginRequest<'_>,
        identity: &LoginIdentity,
        credentials: &repo::Credentials,
        now: DateTime<Utc>,
    ) -> Result<(), AppError> {
        let person = identity.person_id();
        let Some(secret_enc) = credentials.totp_secret_enc.as_ref() else {
            return Err(AppError::Internal(anyhow::anyhow!(
                "enabled TOTP credential has no encrypted secret"
            )));
        };
        let Some(code) = request.totp_code else {
            audit_login_failure(tx, request, Some(person)).await?;
            return Err(AppError::Unauthenticated {
                code: &codes::TOTP_REQUIRED,
            });
        };
        let aad = Aad {
            tenant: request.tenant,
            table: "credentials",
            column: "totp_secret_enc",
            row_id: person.0,
        };
        let secret = self
            .keyring
            .decrypt_field(tx, &aad, secret_enc.expose_secret().as_slice())
            .await
            .map_err(crypto_error)?;
        if !totp_verify(&secret, code, now)? {
            audit_login_failure(tx, request, Some(person)).await?;
            return Err(invalid_credentials());
        }
        Ok(())
    }

    async fn create_login_session(
        &self,
        tx: &mut Tx,
        request: &LoginRequest<'_>,
        identity: &LoginIdentity,
        credentials: &repo::Credentials,
        policy: &IdentityPolicy,
        now: DateTime<Utc>,
    ) -> Result<LoginSuccess, AppError> {
        let person = identity.person_id();
        let active_role = identity.roles().first().copied().ok_or_else(|| {
            AppError::Internal(anyhow::anyhow!("active identity has no active membership role"))
        })?;
        let needs_totp_setup = policy.totp() == TotpPolicy::RequiredForOwnerAdmin
            && matches!(active_role, Role::Owner | Role::Admin)
            && !credentials.totp_enabled;
        repo::reset_failures(tx, request.tenant, person).await?;
        let refresh = new_refresh_token()?;
        let session_id = Uuid::now_v7();
        let family_id = Uuid::now_v7();
        let idle_minutes = match active_role {
            Role::Owner => policy.idle_timeout_minutes().owner,
            Role::Admin => policy.idle_timeout_minutes().admin,
            Role::Instructor => policy.idle_timeout_minutes().instructor,
            Role::Guardian => policy.idle_timeout_minutes().guardian,
            Role::Student => policy.idle_timeout_minutes().student,
        };
        let idle_expires_at = now
            .checked_add_signed(Duration::minutes(i64::from(idle_minutes)))
            .ok_or_else(|| {
                AppError::Internal(anyhow::anyhow!("idle expiration is outside timestamp range"))
            })?;
        let absolute_expires_at = now
            .checked_add_signed(Duration::days(i64::from(policy.absolute_session_days())))
            .ok_or_else(|| {
                AppError::Internal(anyhow::anyhow!("session expiration is outside timestamp range"))
            })?;
        repo::create_refresh_session(
            tx,
            session_id,
            request.tenant,
            person,
            family_id,
            refresh.digest,
            active_role,
            idle_expires_at,
            absolute_expires_at,
            request.user_agent,
            request.ip,
        )
        .await?;
        let password_version = credentials
            .password_changed_at
            .map(|changed_at| changed_at.timestamp())
            .unwrap_or(0);
        let access = issue_access_token(
            self.keys,
            self.clock,
            AccessTokenSubject::new(person, request.tenant, active_role, session_id, password_version),
        )
        .map_err(|error| AppError::Internal(anyhow::anyhow!("access token issuance failed: {error}")))?;
        audit_login_success(tx, request, person).await?;

        Ok(LoginSuccess {
            person,
            roles: identity.roles().to_vec(),
            active_role,
            access_token: access,
            refresh_token: refresh.token,
            needs_totp_setup,
        })
    }

    async fn record_password_failure(
        &self,
        tx: &mut Tx,
        request: &LoginRequest<'_>,
        policy: &IdentityPolicy,
        credentials: &repo::Credentials,
        person: PersonId,
        now: DateTime<Utc>,
    ) -> Result<LoginSuccess, AppError> {
        let lockout = policy.lockout();
        let window = Duration::minutes(i64::from(lockout.window_minutes()));
        let reset_window = credentials
            .failed_window_start
            .is_none_or(|started| now >= started + window);
        let window_start = if reset_window {
            now
        } else {
            credentials.failed_window_start.unwrap_or(now)
        };
        let failed_count = if reset_window {
            1
        } else {
            credentials.failed_count.max(0).saturating_add(1)
        };
        let is_locked = failed_count >= i32::from(lockout.max_failed());
        let locked_until = if is_locked {
            Some(
                now.checked_add_signed(Duration::minutes(i64::from(lockout.lock_minutes())))
                    .ok_or_else(|| {
                        AppError::Internal(anyhow::anyhow!("lock expiration is outside timestamp range"))
                    })?,
            )
        } else {
            None
        };
        repo::set_failures(
            tx,
            request.tenant,
            person,
            failed_count,
            window_start,
            locked_until,
        )
        .await?;
        if let Some(until) = locked_until {
            audit_account_locked(tx, request, person, until).await?;
            emit(
                tx,
                request.tenant,
                &AccountLockedEvent {
                    person_id: person,
                    until,
                },
            )
            .await?;
        } else {
            audit_login_failure(tx, request, Some(person)).await?;
        }
        Err(invalid_credentials())
    }
}

/// IDN-05/09: dependencies for switching one cookie-authenticated session's active role.
pub(crate) struct RoleSwitchService<'a> {
    directory: &'a dyn CoreIdentityDirectory,
    keys: &'a Keys,
    clock: &'a dyn Clock,
}

impl fmt::Debug for RoleSwitchService<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RoleSwitchService")
            .field("directory", &"[core identity port]")
            .field("keys", &"[REDACTED]")
            .field("clock", &self.clock)
            .finish()
    }
}

impl<'a> RoleSwitchService<'a> {
    /// IDN-05/09: builds a role-switch service from the core membership port and process auth state.
    pub(crate) const fn new(
        directory: &'a dyn CoreIdentityDirectory,
        keys: &'a Keys,
        clock: &'a dyn Clock,
    ) -> Self {
        Self {
            directory,
            keys,
            clock,
        }
    }

    /// IDN-09/STM-22: changes only the active role on the locked refresh session.
    pub(crate) async fn switch_role(
        &self,
        tx: &mut Tx,
        request: RoleSwitchRequest<'_>,
    ) -> Result<RoleSwitchSuccess, AppError> {
        let session = repo::refresh_session_for_update(tx, request.tenant, request.token_hash)
            .await?
            .ok_or_else(unauthenticated)?;
        if session.person_id() != request.person {
            return Err(unauthenticated());
        }

        let now = self.clock.now();
        if session.revoked_at().is_some()
            || session.idle_expires_at() <= now
            || session.absolute_expires_at() <= now
        {
            return Err(unauthenticated());
        }

        let roles = self
            .active_membership_roles(tx, request.tenant, session.person_id())
            .await?;
        if !roles.contains(&request.target_role) {
            return Err(forbidden_role());
        }

        let password_version = repo::password_version(tx, request.tenant, session.person_id())
            .await?
            .ok_or_else(unauthenticated)?;
        let access_token = issue_access_token(
            self.keys,
            self.clock,
            AccessTokenSubject::new(
                session.person_id(),
                request.tenant,
                request.target_role,
                session.id(),
                password_version,
            ),
        )
        .map_err(|error| AppError::Internal(anyhow::anyhow!("access token issuance failed: {error}")))?;

        repo::set_refresh_session_role(
            tx,
            request.tenant,
            session.id(),
            request.token_hash,
            request.target_role,
        )
        .await?;

        audit_role_switch(tx, &session, request.target_role, request.actor).await?;

        Ok(RoleSwitchSuccess {
            person: session.person_id(),
            roles,
            active_role: request.target_role,
            access_token,
        })
    }

    async fn active_membership_roles(
        &self,
        tx: &mut Tx,
        tenant: TenantId,
        person: PersonId,
    ) -> Result<Vec<Role>, AppError> {
        let mut roles = Vec::with_capacity(5);
        for role in [
            Role::Owner,
            Role::Admin,
            Role::Instructor,
            Role::Guardian,
            Role::Student,
        ] {
            if self.directory.has_active_role(tx, tenant, person, role).await? {
                roles.push(role);
            }
        }
        Ok(roles)
    }
}

/// SBD-50: parsed refresh credentials are used only through a canonical digest.
pub(crate) fn refresh_token_digest(token: &SecretString) -> Option<Vec<u8>> {
    let encoded = token.expose_secret();
    let raw = URL_SAFE_NO_PAD.decode(encoded).ok()?;
    if raw.len() != 32 || URL_SAFE_NO_PAD.encode(&raw) != encoded {
        return None;
    }
    let raw = Zeroizing::new(raw);
    Some(Sha256::digest(raw.as_slice()).to_vec())
}

/// IDN-09: boundary values for one role-switch request; the token hash is redacted.
pub(crate) struct RoleSwitchRequest<'a> {
    tenant: TenantId,
    person: PersonId,
    token_hash: &'a [u8],
    target_role: Role,
    actor: &'a ActorCtx,
}

impl<'a> RoleSwitchRequest<'a> {
    pub(crate) const fn new(
        tenant: TenantId,
        person: PersonId,
        token_hash: &'a [u8],
        target_role: Role,
        actor: &'a ActorCtx,
    ) -> Self {
        Self {
            tenant,
            person,
            token_hash,
            target_role,
            actor,
        }
    }
}

impl fmt::Debug for RoleSwitchRequest<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RoleSwitchRequest")
            .field("tenant", &"[REDACTED]")
            .field("person", &"[REDACTED]")
            .field("token_hash", &"[REDACTED]")
            .field("target_role", &self.target_role)
            .field("actor", &"[REDACTED]")
            .finish()
    }
}

/// IDN-03/09: safe response data after a successful role switch.
pub(crate) struct RoleSwitchSuccess {
    person: PersonId,
    roles: Vec<Role>,
    active_role: Role,
    access_token: AccessToken,
}

impl RoleSwitchSuccess {
    pub(crate) const fn person(&self) -> PersonId {
        self.person
    }

    pub(crate) fn roles(&self) -> &[Role] {
        &self.roles
    }

    pub(crate) const fn active_role(&self) -> Role {
        self.active_role
    }

    pub(crate) const fn access_token(&self) -> &AccessToken {
        &self.access_token
    }
}

impl fmt::Debug for RoleSwitchSuccess {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RoleSwitchSuccess")
            .field("person", &"[REDACTED]")
            .field("roles", &self.roles)
            .field("active_role", &self.active_role)
            .field("access_token", &"[REDACTED]")
            .finish()
    }
}

fn unauthenticated() -> AppError {
    AppError::Unauthenticated {
        code: &eq_core::error::codes::UNAUTHENTICATED,
    }
}

fn forbidden_role() -> AppError {
    AppError::ForbiddenRule {
        code: &eq_core::error::codes::FORBIDDEN,
        detail: String::from("The selected role is not an active membership"),
    }
}

fn actor_role_for_audit(role: Role) -> ActorRole {
    match role {
        Role::Owner => ActorRole::Owner,
        Role::Admin => ActorRole::Admin,
        Role::Instructor => ActorRole::Instructor,
        Role::Guardian => ActorRole::Guardian,
        Role::Student => ActorRole::Student,
    }
}

async fn audit_role_switch(
    tx: &mut Tx,
    session: &repo::RefreshSession,
    target_role: Role,
    request_actor: &ActorCtx,
) -> Result<(), AppError> {
    let actor = ActorCtx {
        person_id: Some(session.person_id()),
        role: actor_role_for_audit(session.active_role()),
        ip: request_actor.ip,
        request_id: request_actor.request_id,
    };
    record(
        tx,
        AuditEvent {
            action: "security.role_switched",
            category: Category::Security,
            entity_type: "refresh_session",
            entity_id: Some(session.id()),
            before: Some(serde_json::json!({ "active_role": session.active_role() })),
            after: Some(serde_json::json!({ "active_role": target_role })),
            actor: &actor,
        },
    )
    .await
}

/// IDN-01/03/06/12: request values passed from the tenant-aware HTTP boundary.
pub struct LoginRequest<'a> {
    tenant: TenantId,
    actor: &'a ActorCtx,
    field_ctx: &'a FieldCtx,
    email: &'a str,
    password: &'a Password,
    totp_code: Option<&'a str>,
    user_agent: Option<&'a str>,
    ip: Option<IpAddr>,
}

impl<'a> LoginRequest<'a> {
    /// Creates a request using only the tenant and anonymous actor resolved by the HTTP boundary.
    #[expect(
        clippy::too_many_arguments,
        reason = "the request carries the exact login fields and boundary metadata"
    )]
    pub const fn new(
        tenant: TenantId,
        actor: &'a ActorCtx,
        field_ctx: &'a FieldCtx,
        email: &'a str,
        password: &'a Password,
        totp_code: Option<&'a str>,
        user_agent: Option<&'a str>,
        ip: Option<IpAddr>,
    ) -> Self {
        Self {
            tenant,
            actor,
            field_ctx,
            email,
            password,
            totp_code,
            user_agent,
            ip,
        }
    }
}

impl fmt::Debug for LoginRequest<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LoginRequest")
            .field("tenant", &self.tenant)
            .field("actor", &"[REDACTED]")
            .field("email", &"[REDACTED]")
            .field("password", &"[REDACTED]")
            .field("totp_code", &"[REDACTED]")
            .field("user_agent", &"[REDACTED]")
            .field("ip", &"[REDACTED]")
            .finish()
    }
}

/// SBD-50: opaque refresh token secret exposed only when the HTTP handler sets the cookie.
pub struct RefreshToken(SecretString);

impl RefreshToken {
    /// IDN-04/SBD-50: returns the wire token to the HTTP cookie boundary.
    pub fn expose_secret(&self) -> &str {
        self.0.expose_secret()
    }
}

impl fmt::Debug for RefreshToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RefreshToken([REDACTED])")
    }
}

/// IDN-03/04: completed login credentials and safe membership facts for the HTTP layer.
pub struct LoginSuccess {
    person: PersonId,
    roles: Vec<Role>,
    active_role: Role,
    access_token: AccessToken,
    refresh_token: RefreshToken,
    needs_totp_setup: bool,
}

impl LoginSuccess {
    /// Returns the authenticated person identifier.
    pub const fn person(&self) -> PersonId {
        self.person
    }

    /// Returns the tenant's active role list in the IDN-09 privilege order.
    pub fn roles(&self) -> &[Role] {
        &self.roles
    }

    /// Returns the most privileged active membership role selected at login.
    pub const fn active_role(&self) -> Role {
        self.active_role
    }

    /// Returns the access token for the response body.
    pub const fn access_token(&self) -> &AccessToken {
        &self.access_token
    }

    /// Returns the refresh token for the eq_rt cookie.
    pub const fn refresh_token(&self) -> &RefreshToken {
        &self.refresh_token
    }

    /// IDN-12: reports that the owner/admin must set up TOTP after this login.
    pub const fn needs_totp_setup(&self) -> bool {
        self.needs_totp_setup
    }
}

impl fmt::Debug for LoginSuccess {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LoginSuccess")
            .field("person", &self.person)
            .field("roles", &self.roles)
            .field("active_role", &self.active_role)
            .field("access_token", &"[REDACTED]")
            .field("refresh_token", &"[REDACTED]")
            .field("needs_totp_setup", &self.needs_totp_setup)
            .finish()
    }
}

struct NewRefreshToken {
    token: RefreshToken,
    digest: Vec<u8>,
}

impl fmt::Debug for NewRefreshToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("NewRefreshToken")
            .field("token", &"[REDACTED]")
            .field("digest", &"[REDACTED]")
            .finish()
    }
}

fn new_refresh_token() -> Result<NewRefreshToken, AppError> {
    let mut raw = Zeroizing::new([0_u8; 32]);
    OsRng
        .try_fill_bytes(&mut raw[..])
        .map_err(|_| AppError::Internal(anyhow::anyhow!("secure random source failed")))?;
    let encoded = URL_SAFE_NO_PAD.encode(&raw[..]);
    let digest = Sha256::digest(&raw[..]).to_vec();
    Ok(NewRefreshToken {
        token: RefreshToken(SecretString::from(encoded)),
        digest,
    })
}

/// IDN-12: verifies RFC 6238 codes against the injected timestamp, including ±1 step.
pub fn totp_verify(secret: &[u8], code: &str, now: DateTime<Utc>) -> Result<bool, AppError> {
    let totp: Totp = Builder::new()
        .with_algorithm(Algorithm::SHA1)
        .with_digits(6)
        .with_skew(1)
        .with_step_duration(30)
        .with_secret(secret.to_vec())
        .build()
        .map_err(|_| AppError::Internal(anyhow::anyhow!("stored TOTP secret is invalid")))?;
    let timestamp = u64::try_from(now.timestamp())
        .map_err(|_| AppError::Internal(anyhow::anyhow!("TOTP timestamp is outside Unix range")))?;
    Ok(totp.check(code, timestamp).is_some())
}

#[derive(Serialize)]
struct AccountLockedEvent {
    person_id: PersonId,
    until: DateTime<Utc>,
}

impl DomainEvent for AccountLockedEvent {
    const TYPE: &'static str = "identity.account_locked.v1";

    fn aggregate_id(&self) -> Option<Uuid> {
        Some(self.person_id.0)
    }
}

async fn reject_login(
    tx: &mut Tx,
    request: &LoginRequest<'_>,
    person: Option<PersonId>,
) -> Result<LoginSuccess, AppError> {
    audit_login_failure(tx, request, person).await?;
    Err(invalid_credentials())
}

fn verify_dummy(password: &Password) -> Result<(), AppError> {
    let dummy = DUMMY_PASSWORD_HASH
        .get_or_init(create_dummy_password_hash)
        .as_ref()
        .map_err(|error| {
            AppError::Internal(anyhow::anyhow!("dummy password hash is unavailable: {error}"))
        })?;
    verify_dummy_password(password, dummy)
        .map_err(|_| AppError::Internal(anyhow::anyhow!("dummy password verification failed")))
}

fn invalid_credentials() -> AppError {
    AppError::Unauthenticated {
        code: &codes::INVALID_CREDENTIALS,
    }
}

fn crypto_error(error: CryptoError) -> AppError {
    AppError::Internal(anyhow::anyhow!("TOTP secret decryption failed: {error}"))
}

fn audit_actor(actor: &ActorCtx) -> ActorCtx {
    ActorCtx {
        person_id: None,
        role: ActorRole::System,
        ip: actor.ip,
        request_id: actor.request_id,
    }
}

async fn audit_login_failure(
    tx: &mut Tx,
    request: &LoginRequest<'_>,
    person: Option<PersonId>,
) -> Result<(), AppError> {
    let actor = audit_actor(request.actor);
    record(
        tx,
        AuditEvent {
            action: "security.login.failed",
            category: Category::Security,
            entity_type: if person.is_some() { "person" } else { "login" },
            entity_id: person.map(|id| id.0),
            before: None,
            after: None,
            actor: &actor,
        },
    )
    .await
}

async fn audit_login_success(
    tx: &mut Tx,
    request: &LoginRequest<'_>,
    person: PersonId,
) -> Result<(), AppError> {
    let actor = audit_actor(request.actor);
    record(
        tx,
        AuditEvent {
            action: "security.login.success",
            category: Category::Security,
            entity_type: "person",
            entity_id: Some(person.0),
            before: None,
            after: None,
            actor: &actor,
        },
    )
    .await
}

async fn audit_account_locked(
    tx: &mut Tx,
    request: &LoginRequest<'_>,
    person: PersonId,
    until: DateTime<Utc>,
) -> Result<(), AppError> {
    let actor = audit_actor(request.actor);
    record(
        tx,
        AuditEvent {
            action: "security.account.locked",
            category: Category::Security,
            entity_type: "person",
            entity_id: Some(person.0),
            before: None,
            after: Some(serde_json::json!({ "locked_until": until })),
            actor: &actor,
        },
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::{LoginRequest, LoginService, RoleSwitchRequest, audit_actor, totp_verify};
    use crate::IdentityModule;
    use crate::password::hash_password;
    use anyhow::Result;
    use chrono::{DateTime, Duration, TimeZone, Utc};
    use eq_core::actor::{ActorCtx, ActorRole};
    use eq_core::auth::PgIdentityDirectory;
    use eq_core::clock::Clock;
    use eq_core::crypto::{Keyring, Keys, bidx_email};
    use eq_core::db;
    use eq_core::error::AppError;
    use eq_core::fields::{Email, FieldCtx, FieldKind, Password};
    use eq_core::ids::{PersonId, TenantId};
    use eq_core::model::Role;
    use eq_core::module::{CoreModule, Module, ModuleId, ModuleRegistry};
    use eq_core::policy::{PolicyStore, PresetPatchSource, PresetSourceError};
    use serde_json::Value;
    use std::net::IpAddr;
    use std::sync::Arc;
    use totp_rs::{Algorithm, Builder, Totp};
    use uuid::Uuid;

    use eq_testkit::TestDb;
    use eq_testkit::module_sinks::ModuleSinkDoubles;

    const MASTER: &[u8] = b"test-secret-login-service-master-key";
    const EMAIL: &str = "teacher@example.test";
    const CORRECT_PASSWORD: &str = "test-secret-correct-password";
    const WRONG_PASSWORD: &str = "test-secret-incorrect-password";
    // technical: deterministic calendar fixture for service tests.
    const FIXED_TEST_YEAR: i32 = 2026;

    #[test]
    fn role_switch_request_debug_redacts_sensitive_values() {
        let person = PersonId(Uuid::nil());
        let tenant = TenantId(Uuid::nil());
        let actor = ActorCtx {
            person_id: Some(person),
            role: ActorRole::Admin,
            ip: None,
            request_id: Some(Uuid::nil()),
        };
        let token_hash = [0xA5; 32];
        let request = RoleSwitchRequest::new(tenant, person, &token_hash, Role::Guardian, &actor);

        let debug = format!("{request:?}");

        assert!(debug.contains("token_hash: \"[REDACTED]\""));
        assert!(!debug.contains(&person.0.to_string()));
        assert!(!debug.contains(&tenant.0.to_string()));
        assert!(!debug.contains(&format!("{token_hash:?}")));
    }

    #[derive(Clone, Debug)]
    struct TestClock(DateTime<Utc>);

    impl Clock for TestClock {
        fn now(&self) -> DateTime<Utc> {
            self.0
        }
    }

    #[derive(Debug)]
    struct TestPatchSource;

    impl PresetPatchSource for TestPatchSource {
        fn policy_patch(
            &self,
            _preset_id: &str,
            _preset_version: i32,
            _module: ModuleId,
        ) -> std::result::Result<Option<Value>, PresetSourceError> {
            Ok(None)
        }
    }

    struct Fixture {
        db: TestDb,
        tenant: TenantId,
        person: PersonId,
        store: PolicyStore,
        keys: Arc<Keys>,
        keyring: Keyring,
        directory: PgIdentityDirectory,
        clock: Arc<TestClock>,
    }

    impl Fixture {
        fn service(&self) -> LoginService<'_> {
            LoginService::new(
                &self.directory,
                &self.store,
                &self.keys,
                &self.keyring,
                self.clock.as_ref(),
            )
        }

        async fn login(
            &self,
            email: &str,
            password: &Password,
            totp_code: Option<&str>,
        ) -> Result<std::result::Result<super::LoginSuccess, AppError>> {
            let actor = ActorCtx::anonymous(None, Some(Uuid::now_v7()));
            let field_ctx = test_field_ctx();
            let mut tx = db::begin(&self.db.app_pool, self.tenant, &actor).await?;
            let result = self
                .service()
                .login(
                    &mut tx,
                    LoginRequest::new(
                        self.tenant,
                        &actor,
                        &field_ctx,
                        email,
                        password,
                        totp_code,
                        Some("ritma-test/1"),
                        None,
                    ),
                )
                .await;
            match result {
                Ok(success) => {
                    tx.commit().await?;
                    Ok(Ok(success))
                }
                Err(error @ AppError::Unauthenticated { .. }) | Err(error @ AppError::Locked { .. }) => {
                    tx.commit().await?;
                    Ok(Err(error))
                }
                Err(error) => {
                    tx.rollback().await?;
                    Err(error.into())
                }
            }
        }
    }

    async fn fixture() -> Result<Fixture> {
        let db = TestDb::new().await;
        let _test_urls = (&db.url, &db.app_url);
        let tenant = TenantId(Uuid::now_v7());
        let person = PersonId(Uuid::now_v7());
        let clock = Arc::new(TestClock(
            Utc.with_ymd_and_hms(FIXED_TEST_YEAR, 1, 2, 3, 4, 5)
                .single()
                .ok_or_else(|| anyhow::anyhow!("fixed test timestamp is valid"))?,
        ));
        let keys = Arc::new(Keys::derive(MASTER, None));
        let email = parsed_email()?;
        let email_bidx = bidx_email(&keys, tenant, email.as_ref())?;
        seed_tenant(&db, tenant).await?;
        seed_person(&db, tenant, person, email_bidx, Role::Admin).await?;
        let password = parse_password(CORRECT_PASSWORD)?;
        let hash = hash_password(&password)?;
        seed_credentials(&db, tenant, person, hash.expose_secret(), 0, None, None).await?;

        let sinks = ModuleSinkDoubles::default();
        let modules = Arc::new(ModuleRegistry::new(vec![
            Arc::new(CoreModule) as Arc<dyn Module>,
            Arc::new(IdentityModule) as Arc<dyn Module>,
        ]));
        let store = PolicyStore::new(
            db.app_pool.clone(),
            modules,
            Arc::new(TestPatchSource),
            sinks.audit_sink(),
            sinks.event_sink(),
            clock.clone(),
        )
        .await?;

        Ok(Fixture {
            db,
            tenant,
            person,
            store,
            keyring: Keyring::new(Keys::derive(MASTER, None)),
            directory: PgIdentityDirectory::new(Arc::clone(&keys)),
            keys,
            clock,
        })
    }

    fn parsed_email() -> Result<Email> {
        Email::parse(EMAIL.to_owned(), &test_field_ctx())
            .map_err(|_| anyhow::anyhow!("fake test email is valid"))
    }

    fn test_field_ctx() -> FieldCtx {
        FieldCtx {
            country: Some(celes::Country::from_alpha2("ID").expect("known test country")),
            tenant_currency: eq_core::money::Currency::IDR,
            tenant_locale: eq_core::policy::types::Locale::Id,
            today: chrono::NaiveDate::from_ymd_opt(FIXED_TEST_YEAR, 1, 1).expect("valid test date"),
            password_min_length: None,
        }
    }

    fn parse_password(value: &str) -> Result<Password> {
        let input = serde_json::from_value(Value::String(value.to_owned()))?;
        Password::parse(input, &test_field_ctx()).map_err(|_| anyhow::anyhow!("fake test password is valid"))
    }

    async fn seed_tenant(db: &TestDb, tenant: TenantId) -> Result<()> {
        let slug = format!("login-{}", tenant.0.simple());
        sqlx::query!(
            "INSERT INTO public.tenants (id, slug, name, country, preset_id, preset_version) \
             VALUES ($1, $2, 'Test School', 'ID', 'test', 1)",
            tenant.0,
            slug
        )
        .execute(&db.pool)
        .await?;
        Ok(())
    }

    async fn seed_person(
        db: &TestDb,
        tenant: TenantId,
        person: PersonId,
        email_bidx: [u8; 16],
        role: Role,
    ) -> Result<()> {
        sqlx::query!(
            "INSERT INTO public.persons (id, tenant_id, display_name, email_bidx) VALUES ($1, $2, 'Test Person', $3)",
            person.0,
            tenant.0,
            email_bidx.as_slice()
        )
        .execute(&db.pool)
        .await?;
        sqlx::query!(
            "INSERT INTO public.memberships (tenant_id, person_id, role, status) VALUES ($1, $2, $3, 'active')",
            tenant.0,
            person.0,
            role as _
        )
        .execute(&db.pool)
        .await?;
        Ok(())
    }

    async fn seed_credentials(
        db: &TestDb,
        tenant: TenantId,
        person: PersonId,
        password_hash: &str,
        failed_count: i32,
        failed_window_start: Option<DateTime<Utc>>,
        locked_until: Option<DateTime<Utc>>,
    ) -> Result<()> {
        sqlx::query!(
            "INSERT INTO public.credentials (tenant_id, person_id, password_hash, failed_count, failed_window_start, locked_until) \
             VALUES ($1, $2, $3, $4, $5, $6)",
            tenant.0,
            person.0,
            password_hash,
            failed_count,
            failed_window_start,
            locked_until
        )
        .execute(&db.pool)
        .await?;
        Ok(())
    }

    #[tokio::test]
    async fn lockout_threshold_emits_and_audits() -> Result<()> {
        let fixture = fixture().await?;
        let wrong = parse_password(WRONG_PASSWORD)?;
        for _ in 0..crate::policy::IdentityPolicy::default().lockout().max_failed() {
            let result = fixture.login(EMAIL, &wrong, None).await?;
            assert!(
                matches!(result, Err(AppError::Unauthenticated { code }) if code.code == "invalid_credentials")
            );
        }

        let correct = parse_password(CORRECT_PASSWORD)?;
        let locked = fixture.login(EMAIL, &correct, None).await?;
        let until = match locked {
            Err(AppError::Locked { until: Some(until) }) => until,
            other => anyhow::bail!("correct credentials during lock should return 423, got {other:?}"),
        };
        let row = sqlx::query!(
            "SELECT failed_count, locked_until FROM public.credentials WHERE tenant_id = $1 AND person_id = $2",
            fixture.tenant.0,
            fixture.person.0
        )
        .fetch_one(&fixture.db.pool)
        .await?;
        assert_eq!(row.failed_count, 5);
        assert_eq!(row.locked_until, Some(until));

        let audit = sqlx::query!(
            "SELECT count(*)::bigint AS \"count!\" FROM audit.entries WHERE tenant_id = $1 AND entity_id = $2 AND action = 'security.account.locked'",
            fixture.tenant.0,
            fixture.person.0
        )
        .fetch_one(&fixture.db.pool)
        .await?;
        assert_eq!(audit.count, 1);
        let event = sqlx::query!(
            "SELECT payload FROM public.outbox WHERE tenant_id = $1 AND event_type = 'identity.account_locked.v1'",
            fixture.tenant.0
        )
        .fetch_one(&fixture.db.pool)
        .await?;
        assert_eq!(
            event.payload["payload"]["person_id"],
            fixture.person.0.to_string()
        );
        assert!(event.payload["payload"]["until"].is_string());
        Ok(())
    }

    #[tokio::test]
    async fn locked_wrong_password_is_generic() -> Result<()> {
        let fixture = fixture().await?;
        let locked_until = fixture.clock.now() + Duration::minutes(10);
        sqlx::query!(
            "UPDATE public.credentials SET failed_count = 5, failed_window_start = $3, locked_until = $4 \
             WHERE tenant_id = $1 AND person_id = $2",
            fixture.tenant.0,
            fixture.person.0,
            fixture.clock.now(),
            locked_until
        )
        .execute(&fixture.db.pool)
        .await?;

        let wrong = parse_password(WRONG_PASSWORD)?;
        let result = fixture.login(EMAIL, &wrong, None).await?;
        assert!(
            matches!(result, Err(AppError::Unauthenticated { code }) if code.code == "invalid_credentials")
        );
        let row = sqlx::query!(
            "SELECT failed_count, locked_until FROM public.credentials WHERE tenant_id = $1 AND person_id = $2",
            fixture.tenant.0,
            fixture.person.0
        )
        .fetch_one(&fixture.db.pool)
        .await?;
        assert_eq!(row.failed_count, 5);
        assert_eq!(row.locked_until, Some(locked_until));
        Ok(())
    }

    #[tokio::test]
    async fn successful_login_resets_failures() -> Result<()> {
        let fixture = fixture().await?;
        sqlx::query!(
            "UPDATE public.credentials SET failed_count = 3, failed_window_start = $3 \
             WHERE tenant_id = $1 AND person_id = $2",
            fixture.tenant.0,
            fixture.person.0,
            fixture.clock.now() - Duration::minutes(1)
        )
        .execute(&fixture.db.pool)
        .await?;

        let correct = parse_password(CORRECT_PASSWORD)?;
        let success = fixture
            .login(EMAIL, &correct, None)
            .await?
            .map_err(|error| anyhow::anyhow!("valid credentials should authenticate: {error}"))?;
        assert_eq!(success.person(), fixture.person);
        assert_eq!(success.active_role(), Role::Admin);
        assert_eq!(success.roles(), &[Role::Admin]);
        assert!(!success.needs_totp_setup());
        assert!(!format!("{success:?}").contains(success.access_token().expose_secret()));
        assert!(!format!("{success:?}").contains(success.refresh_token().expose_secret()));

        let row = sqlx::query!(
            "SELECT failed_count, failed_window_start, locked_until FROM public.credentials WHERE tenant_id = $1 AND person_id = $2",
            fixture.tenant.0,
            fixture.person.0
        )
        .fetch_one(&fixture.db.pool)
        .await?;
        assert_eq!(row.failed_count, 0);
        assert_eq!(row.failed_window_start, None);
        assert_eq!(row.locked_until, None);
        let session = sqlx::query!(
            "SELECT token_hash, active_role, idle_expires_at, absolute_expires_at FROM public.refresh_sessions WHERE tenant_id = $1 AND person_id = $2",
            fixture.tenant.0,
            fixture.person.0
        )
        .fetch_one(&fixture.db.pool)
        .await?;
        assert_eq!(session.token_hash.len(), 32);
        assert_eq!(session.active_role, "admin");
        assert!(session.idle_expires_at > fixture.clock.now());
        assert!(session.absolute_expires_at > session.idle_expires_at);
        Ok(())
    }

    #[tokio::test]
    async fn unknown_email_runs_dummy_hash() -> Result<()> {
        let fixture = fixture().await?;
        let password = parse_password(WRONG_PASSWORD)?;
        let result = fixture.login("unknown@example.test", &password, None).await?;
        assert!(
            matches!(result, Err(AppError::Unauthenticated { code }) if code.code == "invalid_credentials")
        );
        let audit = sqlx::query!(
            "SELECT entity_type, entity_id FROM audit.entries WHERE tenant_id = $1 AND action = 'security.login.failed' ORDER BY seq DESC LIMIT 1",
            fixture.tenant.0
        )
        .fetch_one(&fixture.db.pool)
        .await?;
        assert_eq!(audit.entity_type, "login");
        assert_eq!(audit.entity_id, None);
        Ok(())
    }

    #[test]
    fn totp_verify_uses_injected_time_and_rfc_window() -> Result<()> {
        let secret = b"test-secret-totp-seed-0001".to_vec();
        let totp: Totp = Builder::new()
            .with_algorithm(Algorithm::SHA1)
            .with_digits(6)
            .with_skew(1)
            .with_step_duration(30)
            .with_secret(secret.clone())
            .build()
            .map_err(|_| anyhow::anyhow!("fake TOTP secret is valid"))?;
        let now = Utc
            .with_ymd_and_hms(FIXED_TEST_YEAR, 1, 2, 3, 4, 5)
            .single()
            .ok_or_else(|| anyhow::anyhow!("fixed TOTP timestamp is valid"))?;
        let timestamp = u64::try_from(now.timestamp())?;
        let current_code = totp.generate(timestamp).to_string();
        assert!(totp_verify(&secret, &current_code, now)?);
        Ok(())
    }

    #[test]
    fn unauthenticated_audit_role_keeps_request_context_without_person() {
        let actor = ActorCtx {
            person_id: None,
            role: ActorRole::Anonymous,
            ip: Some("192.0.2.10".parse::<IpAddr>().expect("test IP parses")),
            request_id: Some(Uuid::now_v7()),
        };
        let audit = audit_actor(&actor);
        assert_eq!(audit.person_id, None);
        assert_eq!(audit.role, ActorRole::System);
        assert_eq!(audit.ip, actor.ip);
        assert_eq!(audit.request_id, actor.request_id);
    }
}
