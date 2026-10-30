//! IDN-01/03/04/06: tenant-aware HTTP login boundary.

use std::collections::BTreeSet;
use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use crate::errors::codes;
use axum::Json;
use axum::extract::{Extension, Request, State};
use axum::http::header::{AUTHORIZATION, CACHE_CONTROL, COOKIE, ORIGIN, SET_COOKIE, USER_AGENT};
use axum::http::{HeaderMap, HeaderValue, Method, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use chrono_tz::Tz;
use eq_core::actor::{ActorCtx, ActorRole};
use eq_core::app_state::AppState;
use eq_core::auth::{Actor, AuthState};
use eq_core::clock::Clock;
use eq_core::db::{Tx, begin};
use eq_core::error::AppError;
use eq_core::fields::{FieldCtx, FieldKind, Password, PasswordInput};
use eq_core::ids::{PersonId, TenantId};
use eq_core::model::Role;
use eq_core::module::gate::EnabledModuleSet;
use eq_core::module::{ModuleRegistry, PermissionDef};
use eq_core::outbox::OutboxEventSink;
use eq_core::policy::types::Locale;
use eq_core::policy::{PolicyStore, PresetPatchSource, PresetSourceError};
use eq_core::ports::sinks::{AuditSink, EventSink};
use eq_core::tenant::{TenantCtx, TenantResolver, client_ip};
use moka::future::Cache;
use secrecy::{ExposeSecret, SecretString};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
use serde_json::Value;
use sqlx::PgPool;
use tokio::sync::OnceCell;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::policy::IdentityPolicy;
use crate::service::{
    LoginRequest, LoginService, LoginSuccess, RefreshOutcome, RefreshRequest, RefreshService, RefreshSuccess,
    RoleSwitchRequest, RoleSwitchService, RoleSwitchSuccess, refresh_token_digest,
};

const ACCESS_TOKEN_TTL_SECONDS: u64 = 600; // IDN-03: technical access-token lifetime.
const SECONDS_PER_DAY: u64 = 86_400; // technical: convert the policy's absolute session days to cookie Max-Age.
const PERMISSION_CACHE_TTL: Duration = Duration::from_secs(30); // technical: IDN-11 permission cache bound.
const PASSWORD_VERSION_CACHE_TTL: Duration = Duration::from_secs(60); // technical: IDN-03 credential-version cache bound.

/// IDN-06/POL-04: shared lazy policy runtime used by login routes.
pub struct LoginHttpState {
    pool: PgPool,
    clock: Arc<dyn Clock>,
    modules: Arc<ModuleRegistry>,
    enabled: Arc<dyn EnabledModuleSet>,
    auth: Arc<AuthState>,
    tenant_resolver: TenantResolver,
    policy_store: OnceCell<PolicyStore>,
    permissions: Cache<(TenantId, PersonId, &'static str), Arc<BTreeSet<String>>>,
    password_versions: Cache<(TenantId, PersonId), i64>,
}

impl LoginHttpState {
    /// IDN-06/POL-04: constructs one process-scoped runtime without requiring PostgreSQL at router startup.
    pub fn new(
        pool: PgPool,
        clock: Arc<dyn Clock>,
        modules: Arc<ModuleRegistry>,
        enabled: Arc<dyn EnabledModuleSet>,
        auth: Arc<AuthState>,
        tenant_resolver: TenantResolver,
    ) -> Self {
        Self {
            pool,
            clock,
            modules,
            enabled,
            auth,
            tenant_resolver,
            policy_store: OnceCell::new(),
            permissions: Cache::builder().time_to_live(PERMISSION_CACHE_TTL).build(),
            password_versions: Cache::builder().time_to_live(PASSWORD_VERSION_CACHE_TTL).build(),
        }
    }

    /// POL-06: shares the lazy process policy runtime with the Core settings adapter.
    pub async fn policy_store(&self) -> Result<&PolicyStore, AppError> {
        self.policy_store
            .get_or_try_init(|| async {
                let presets: Arc<dyn PresetPatchSource> = Arc::new(UnavailablePresetPatches);
                let audits: Arc<dyn AuditSink> = Arc::new(eq_audit::AuditModule);
                let events: Arc<dyn EventSink> = Arc::new(OutboxEventSink);
                PolicyStore::new(
                    self.pool.clone(),
                    Arc::clone(&self.modules),
                    presets,
                    audits,
                    events,
                    Arc::clone(&self.clock),
                )
                .await
            })
            .await
    }

    pub(crate) async fn permissions(
        &self,
        tx: &mut Tx,
        tenant: TenantId,
        person: PersonId,
        role: Role,
    ) -> Result<BTreeSet<String>, AppError> {
        let cache_key = (tenant, person, role_name(role));
        if let Some(permissions) = self.permissions.get(&cache_key).await {
            return Ok(permissions.as_ref().clone());
        }

        let owner_without_admin = if role == Role::Owner {
            !self.auth.directory().has_active_admin(tx, tenant).await?
        } else {
            false
        };
        let actor_role = actor_role(role);
        let mut permissions = BTreeSet::new();
        for module in self.modules.modules() {
            if !self.enabled.is_enabled(tenant, module.manifest().id).await? {
                continue;
            }
            for permission in module.permissions() {
                if has_role_grant(permission, actor_role, owner_without_admin) {
                    permissions.insert(permission.key.to_owned());
                }
            }
        }
        self.permissions
            .insert(cache_key, Arc::new(permissions.clone()))
            .await;
        Ok(permissions)
    }

    /// IDN-03: invalidates the cached password version after a password change.
    pub async fn invalidate_password_version(&self, tenant: TenantId, person: PersonId) {
        self.password_versions.invalidate(&(tenant, person)).await;
    }

    async fn authenticate_access_token(
        &self,
        tenant: TenantId,
        token: &str,
        request_actor: ActorCtx,
    ) -> Result<Actor, AppError> {
        let claims = crate::jwt::verify_access_token(token, self.auth.keys(), tenant, self.clock.as_ref())
            .map_err(|_| AppError::Unauthenticated {
                code: &eq_core::error::codes::UNAUTHENTICATED,
            })?;
        let actor_ctx = ActorCtx {
            person_id: Some(claims.sub()),
            role: actor_role(claims.role()),
            ip: request_actor.ip,
            request_id: request_actor.request_id,
        };
        let mut tx = begin(&self.pool, tenant, &actor_ctx).await?;
        let result = self.authenticated_actor(&mut tx, tenant, claims, actor_ctx).await;
        match result {
            Ok(actor) => {
                tx.commit().await?;
                Ok(actor)
            }
            Err(error) => {
                tx.rollback().await?;
                Err(error)
            }
        }
    }

    async fn authenticated_actor(
        &self,
        tx: &mut Tx,
        tenant: TenantId,
        claims: crate::jwt::AccessClaims,
        actor_ctx: ActorCtx,
    ) -> Result<Actor, AppError> {
        if !self
            .auth
            .directory()
            .has_active_role(tx, tenant, claims.sub(), claims.role())
            .await?
        {
            return Err(AppError::Unauthenticated {
                code: &eq_core::error::codes::UNAUTHENTICATED,
            });
        }

        let version_key = (tenant, claims.sub());
        let password_version = if let Some(version) = self.password_versions.get(&version_key).await {
            version
        } else {
            let version = crate::repo::password_version(tx, tenant, claims.sub())
                .await?
                .ok_or(AppError::Unauthenticated {
                    code: &eq_core::error::codes::UNAUTHENTICATED,
                })?;
            self.password_versions.insert(version_key, version).await;
            version
        };
        if password_version != claims.ver() {
            return Err(AppError::Unauthenticated {
                code: &eq_core::error::codes::UNAUTHENTICATED,
            });
        }

        let permissions = self.permissions(tx, tenant, claims.sub(), claims.role()).await?;
        Ok(Actor::from_verified_token(
            claims.sub(),
            tenant,
            claims.role(),
            claims.sid(),
            permissions,
            actor_ctx,
        ))
    }
}

impl fmt::Debug for LoginHttpState {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LoginHttpState")
            .field("pool", &"[configured]")
            .field("clock", &self.clock)
            .field("modules", &self.modules)
            .field("enabled", &"[tenant module set]")
            .field("policy_store", &self.policy_store.get().is_some())
            .finish()
    }
}

/// IDN-03/AUTH-05: inserts an Actor only after bearer token and active membership checks.
pub async fn actor_auth_middleware(
    State(runtime): State<Arc<LoginHttpState>>,
    request: Request,
    next: Next,
) -> Response {
    let (mut parts, body) = request.into_parts();
    let Some(token) = parts.headers.get(AUTHORIZATION).and_then(parse_bearer_token) else {
        if parts.headers.contains_key(AUTHORIZATION) {
            return unauthenticated_response();
        }
        return next.run(Request::from_parts(parts, body)).await;
    };
    let Some(TenantCtx::Resolved(tenant)) = parts.extensions.get::<TenantCtx>() else {
        return unauthenticated_response();
    };
    let request_actor = ActorCtx::anonymous(
        client_ip(&parts, runtime.tenant_resolver.trusted_proxies()),
        parts.extensions.get::<Uuid>().copied(),
    );
    let actor = match runtime
        .authenticate_access_token(tenant.id, token.expose_secret(), request_actor)
        .await
    {
        Ok(actor) => actor,
        Err(error) => return error.into_response(),
    };
    parts.extensions.insert(actor);
    next.run(Request::from_parts(parts, body)).await
}

fn parse_bearer_token(value: &HeaderValue) -> Option<SecretString> {
    let token = value.to_str().ok()?.strip_prefix("Bearer ")?;
    if token.is_empty() || token.bytes().any(|byte| byte.is_ascii_whitespace()) {
        return None;
    }
    Some(SecretString::from(String::from(token)))
}

fn unauthenticated_response() -> Response {
    AppError::Unauthenticated {
        code: &eq_core::error::codes::UNAUTHENTICATED,
    }
    .into_response()
}

struct UnavailablePresetPatches;

impl PresetPatchSource for UnavailablePresetPatches {
    fn policy_patch(
        &self,
        _preset_id: &str,
        _preset_version: i32,
        _module: eq_core::module::ModuleId,
    ) -> Result<Option<Value>, PresetSourceError> {
        Err(PresetSourceError)
    }
}

#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct LoginBody {
    email: String,
    password: PasswordInput,
    #[serde(default)]
    totp: Option<TotpCode>,
}

#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct SwitchRoleBody {
    #[schema(value_type = RoleSchema)]
    role: Role,
}

impl fmt::Debug for SwitchRoleBody {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SwitchRoleBody")
            .field("role", &self.role)
            .finish()
    }
}

impl fmt::Debug for LoginBody {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LoginBody")
            .field("email", &"[REDACTED]")
            .field("password", &"[REDACTED]")
            .field("totp", &"[REDACTED]")
            .finish()
    }
}

#[derive(ToSchema)]
#[schema(value_type = String)]
struct TotpCode(SecretString);

impl<'de> Deserialize<'de> for TotpCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer).map(|value| Self(SecretString::from(value)))
    }
}

impl fmt::Debug for TotpCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("TotpCode([REDACTED])")
    }
}

/// IDN-01: the `PersonMe` values returned at login.
#[derive(Serialize, ToSchema)]
pub struct PersonMe {
    #[schema(value_type = String, format = Uuid)]
    pub id: PersonId,
    pub display_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    #[schema(value_type = Option<LocaleSchema>)]
    pub locale: Option<Locale>,
    pub avatar_url: Option<String>,
    #[schema(value_type = Vec<RoleSchema>)]
    pub roles: Vec<Role>,
    #[schema(value_type = RoleSchema)]
    pub active_role: Role,
    pub permissions: Vec<String>,
    #[schema(value_type = Option<Vec<LinkedStudent>>)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linked_students: Option<Vec<LinkedStudent>>,
}

#[derive(ToSchema)]
#[schema(rename_all = "snake_case")]
#[expect(dead_code, reason = "OpenAPI schema mirrors the canonical Role wire values")]
enum RoleSchema {
    Owner,
    Admin,
    Instructor,
    Guardian,
    Student,
}

#[derive(ToSchema)]
#[schema(rename_all = "lowercase")]
#[expect(dead_code, reason = "OpenAPI schema mirrors the canonical Locale wire values")]
enum LocaleSchema {
    En,
    Id,
}

impl fmt::Debug for PersonMe {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PersonMe")
            .field("id", &self.id)
            .field("display_name", &"[REDACTED]")
            .field("email", &self.email.as_ref().map(|_| "[REDACTED]"))
            .field("phone", &self.phone.as_ref().map(|_| "[REDACTED]"))
            .field("locale", &self.locale)
            .field("avatar_url", &self.avatar_url.as_ref().map(|_| "[REDACTED]"))
            .field("roles", &self.roles)
            .field("active_role", &self.active_role)
            .field("permissions", &self.permissions)
            .field(
                "linked_students",
                &self.linked_students.as_ref().map(|_| "[REDACTED]"),
            )
            .finish()
    }
}

/// IDN-01: optional guardian-linked student summary.
#[derive(Serialize, ToSchema)]
pub struct LinkedStudent {
    #[schema(value_type = String, format = Uuid)]
    pub id: PersonId,
    pub display_name: String,
}

impl fmt::Debug for LinkedStudent {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LinkedStudent")
            .field("id", &self.id)
            .field("display_name", &"[REDACTED]")
            .finish()
    }
}

#[derive(ToSchema)]
#[expect(
    dead_code,
    reason = "OpenAPI response schema describes the secret-safe manual serializer"
)]
pub(crate) struct LoginResponseSchema {
    access_token: String,
    expires_in: u64,
    person: PersonMe,
    roles: Vec<RoleSchema>,
    active_role: RoleSchema,
    needs_totp: Option<bool>,
}

struct LoginResponse<'a> {
    access_token: &'a crate::jwt::AccessToken,
    expires_in: u64,
    person: PersonMe,
    roles: Vec<Role>,
    active_role: Role,
    needs_totp: Option<bool>,
}

impl Serialize for LoginResponse<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut response = serializer.serialize_struct("LoginResponse", 6)?;
        response.serialize_field("access_token", self.access_token.expose_secret())?;
        response.serialize_field("expires_in", &self.expires_in)?;
        response.serialize_field("person", &self.person)?;
        response.serialize_field("roles", &self.roles)?;
        response.serialize_field("active_role", &self.active_role)?;
        if let Some(needs_totp) = self.needs_totp {
            response.serialize_field("needs_totp", &needs_totp)?;
        }
        response.end()
    }
}

impl fmt::Debug for LoginResponse<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LoginResponse")
            .field("access_token", &"[REDACTED]")
            .field("expires_in", &self.expires_in)
            .field("person", &self.person)
            .field("roles", &self.roles)
            .field("active_role", &self.active_role)
            .field("needs_totp", &self.needs_totp)
            .finish()
    }
}

/// IDN-01/03/04: authenticates one tenant user and sets the refresh token only as `eq_rt`.
#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    request_body = LoginBody,
    responses(
        (status = 200, description = "Login succeeded", body = LoginResponseSchema),
        (status = 401, description = "Invalid credentials or TOTP required"),
        (status = 423, description = "Account is locked after correct credentials")
    ),
    tag = "identity"
)]
pub(crate) async fn login(
    State(state): State<AppState>,
    Extension(runtime): Extension<Arc<LoginHttpState>>,
    eq_core::tenant::RequireTenant(tenant): eq_core::tenant::RequireTenant,
    eq_core::tenant::RequestActor(actor): eq_core::tenant::RequestActor,
    headers: HeaderMap,
    Json(body): Json<LoginBody>,
) -> Result<Response, AppError> {
    let timezone = tenant
        .timezone
        .parse::<Tz>()
        .map_err(|_| AppError::Internal(anyhow::anyhow!("resolved tenant timezone is invalid")))?;
    let field_ctx = FieldCtx {
        country: None,
        tenant_currency: tenant.currency,
        tenant_locale: tenant.locale,
        today: state.clock.now().with_timezone(&timezone).date_naive(),
        password_min_length: None,
    };
    let password = Password::parse(body.password, &field_ctx).map_err(|_| AppError::Unauthenticated {
        code: &codes::INVALID_CREDENTIALS,
    })?;
    let user_agent = headers
        .get(USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    let policies = runtime.policy_store().await?;
    let mut tx = begin(&state.pool, tenant.id, &actor).await?;
    let service = LoginService::new(
        state.auth.directory(),
        policies,
        state.auth.keys(),
        state.auth.keyring(),
        state.clock.as_ref(),
    );
    let login = service
        .login(
            &mut tx,
            LoginRequest::new(
                tenant.id,
                &actor,
                &field_ctx,
                &body.email,
                &password,
                body.totp.as_ref().map(|code| code.0.expose_secret()),
                user_agent.as_deref(),
                actor.ip,
            ),
        )
        .await;
    let success = match login {
        Ok(success) => success,
        Err(error @ (AppError::Unauthenticated { .. } | AppError::Locked { .. })) => {
            tx.commit().await?;
            return Err(error);
        }
        Err(error) => {
            tx.rollback().await?;
            return Err(error);
        }
    };

    let response = login_success_response(&state, &runtime, tenant.id, &mut tx, policies, &success).await?;
    tx.commit().await?;
    Ok(response)
}

/// IDN-05/09: changes the active role on the existing cookie-authenticated refresh session.
#[utoipa::path(
    post,
    path = "/api/v1/auth/switch-role",
    request_body = SwitchRoleBody,
    responses(
        (status = 200, description = "Role switched", body = LoginResponseSchema),
        (status = 401, description = "Refresh session is missing, invalid or expired"),
        (status = 403, description = "CSRF check failed or selected role is not active")
    ),
    security(("cookie" = [])),
    tag = "identity"
)]
pub(crate) async fn switch_role(
    State(state): State<AppState>,
    Extension(runtime): Extension<Arc<LoginHttpState>>,
    eq_core::tenant::RequireTenant(tenant): eq_core::tenant::RequireTenant,
    eq_core::tenant::RequestActor(request_actor): eq_core::tenant::RequestActor,
    headers: HeaderMap,
    Json(body): Json<SwitchRoleBody>,
) -> Result<Response, AppError> {
    let expected_origin = state.config.public_url.origin().ascii_serialization();
    if !valid_csrf(&headers, &expected_origin) {
        return Err(forbidden_request("CSRF validation failed"));
    }
    let refresh_token = parse_refresh_cookie(&headers).ok_or_else(unauthenticated_request)?;
    let token_hash = refresh_token_digest(&refresh_token).ok_or_else(unauthenticated_request)?;

    let mut lookup_tx = begin(&state.pool, tenant.id, &request_actor).await?;
    let identity = match crate::repo::refresh_session_identity(&mut lookup_tx, tenant.id, &token_hash).await?
    {
        Some(identity) => {
            lookup_tx.commit().await?;
            identity
        }
        None => {
            lookup_tx.rollback().await?;
            return Err(unauthenticated_request());
        }
    };
    let actor = ActorCtx {
        person_id: Some(identity.person_id()),
        role: actor_role(identity.active_role()),
        ip: request_actor.ip,
        request_id: request_actor.request_id,
    };
    let mut tx = begin(&state.pool, tenant.id, &actor).await?;
    let service = RoleSwitchService::new(state.auth.directory(), state.auth.keys(), state.clock.as_ref());
    let switched = match service
        .switch_role(
            &mut tx,
            RoleSwitchRequest::new(tenant.id, identity.person_id(), &token_hash, body.role, &actor),
        )
        .await
    {
        Ok(switched) => switched,
        Err(error) => {
            tx.rollback().await?;
            return Err(error);
        }
    };

    let response = match role_switch_success_response(&state, &runtime, tenant.id, &mut tx, &switched).await {
        Ok(response) => response,
        Err(error) => {
            tx.rollback().await?;
            return Err(error);
        }
    };
    tx.commit().await?;
    Ok(response)
}

/// IDN-04/05: rotates a cookie-authenticated refresh session and returns a new access token.
#[utoipa::path(
    post,
    path = "/api/v1/auth/refresh",
    responses(
        (status = 200, description = "Refresh succeeded", body = LoginResponseSchema),
        (status = 401, description = "Refresh session is missing, invalid, expired or replayed"),
        (status = 403, description = "CSRF check failed")
    ),
    security(("refreshCookie" = [])),
    tag = "identity"
)]
pub(crate) async fn refresh(
    State(state): State<AppState>,
    Extension(runtime): Extension<Arc<LoginHttpState>>,
    eq_core::tenant::RequireTenant(tenant): eq_core::tenant::RequireTenant,
    eq_core::tenant::RequestActor(request_actor): eq_core::tenant::RequestActor,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let expected_origin = state.config.public_url.origin().ascii_serialization();
    if !valid_csrf(&headers, &expected_origin) {
        return Err(forbidden_request("CSRF validation failed"));
    }
    let refresh_token = parse_refresh_cookie(&headers).ok_or_else(session_expired_request)?;
    let token_hash = refresh_token_digest(&refresh_token).ok_or_else(session_expired_request)?;

    let identity = lookup_refresh_identity(&state, tenant.id, &request_actor, &token_hash).await?;
    let actor = ActorCtx {
        person_id: Some(identity.person_id()),
        role: actor_role(identity.active_role()),
        ip: request_actor.ip,
        request_id: request_actor.request_id,
    };
    let mut tx = begin(&state.pool, tenant.id, &actor).await?;
    let policies = match runtime.policy_store().await {
        Ok(policies) => policies,
        Err(error) => {
            tx.rollback().await?;
            return Err(error);
        }
    };
    let policy = match policies.current::<IdentityPolicy>(&mut tx, tenant.id).await {
        Ok(policy) => policy,
        Err(error) => {
            tx.rollback().await?;
            return Err(error);
        }
    };
    let user_agent = headers
        .get(USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    let service = RefreshService::new(state.auth.directory(), state.auth.keys(), state.clock.as_ref());
    let outcome = match service
        .refresh(
            &mut tx,
            RefreshRequest::new(
                tenant.id,
                identity.person_id(),
                &token_hash,
                &policy.doc,
                &actor,
                user_agent.as_deref(),
            ),
        )
        .await
    {
        Ok(outcome) => outcome,
        Err(error) => {
            tx.rollback().await?;
            return Err(error);
        }
    };
    match outcome {
        RefreshOutcome::Success(success) => {
            let response =
                match refresh_success_response(&state, &runtime, tenant.id, &mut tx, &success).await {
                    Ok(response) => response,
                    Err(error) => {
                        tx.rollback().await?;
                        return Err(error);
                    }
                };
            tx.commit().await?;
            Ok(response)
        }
        RefreshOutcome::Reused => {
            tx.commit().await?;
            Err(session_expired_request())
        }
        RefreshOutcome::Expired => {
            tx.rollback().await?;
            Err(session_expired_request())
        }
    }
}

/// IDN-05/ARC-04: rejects cross-site cookie-auth requests before tenant resolution can query PostgreSQL.
pub async fn role_switch_csrf_guard(
    State(expected_origin): State<String>,
    request: Request,
    next: Next,
) -> Response {
    let cookie_auth_path = matches!(
        request.uri().path(),
        "/api/v1/auth/switch-role" | "/api/v1/auth/refresh"
    );
    if request.method() == Method::POST
        && cookie_auth_path
        && !valid_csrf(request.headers(), &expected_origin)
    {
        return forbidden_request("CSRF validation failed").into_response();
    }
    next.run(request).await
}

async fn role_switch_success_response(
    state: &AppState,
    runtime: &LoginHttpState,
    tenant: TenantId,
    tx: &mut Tx,
    success: &RoleSwitchSuccess,
) -> Result<Response, AppError> {
    authenticated_response(AuthenticatedResponse {
        state,
        runtime,
        tenant,
        tx,
        person_id: success.person(),
        roles: success.roles(),
        active_role: success.active_role(),
        access_token: success.access_token(),
        needs_totp: None,
    })
    .await
}

async fn refresh_success_response(
    state: &AppState,
    runtime: &LoginHttpState,
    tenant: TenantId,
    tx: &mut Tx,
    success: &RefreshSuccess,
) -> Result<Response, AppError> {
    let remaining = success
        .absolute_expires_at()
        .signed_duration_since(state.clock.now())
        .num_seconds()
        .max(0);
    let max_age = u64::try_from(remaining).map_err(|error| AppError::Internal(anyhow::Error::new(error)))?;
    let secure = !(state.config.public_url.scheme() == "http"
        && state.config.public_url.host_str() == Some("localhost"));
    let cookie = refresh_cookie(success.refresh_token().expose_secret(), max_age, secure);
    let cookie =
        HeaderValue::from_str(&cookie).map_err(|error| AppError::Internal(anyhow::Error::new(error)))?;
    let mut response = authenticated_response(AuthenticatedResponse {
        state,
        runtime,
        tenant,
        tx,
        person_id: success.person(),
        roles: success.roles(),
        active_role: success.active_role(),
        access_token: success.access_token(),
        needs_totp: None,
    })
    .await?;
    response.headers_mut().append(SET_COOKIE, cookie);
    Ok(response)
}

struct AuthenticatedResponse<'a> {
    state: &'a AppState,
    runtime: &'a LoginHttpState,
    tenant: TenantId,
    tx: &'a mut Tx,
    person_id: PersonId,
    roles: &'a [Role],
    active_role: Role,
    access_token: &'a crate::jwt::AccessToken,
    needs_totp: Option<bool>,
}

async fn authenticated_response(input: AuthenticatedResponse<'_>) -> Result<Response, AppError> {
    let AuthenticatedResponse {
        state,
        runtime,
        tenant,
        tx,
        person_id,
        roles,
        active_role,
        access_token,
        needs_totp,
    } = input;
    let permissions = runtime.permissions(tx, tenant, person_id, active_role).await?;
    let include_pii = permissions.contains("identity.people.read_pii");
    let profile = state
        .auth
        .directory()
        .login_profile(tx, tenant, person_id, include_pii, state.auth.keyring())
        .await?
        .ok_or(AppError::NotFound { entity: "person" })?;
    let person = PersonMe {
        id: person_id,
        display_name: profile.display_name,
        email: profile.email.map(|value| value.expose_secret().to_owned()),
        phone: profile.phone.map(|value| value.expose_secret().to_owned()),
        locale: profile.locale,
        avatar_url: None,
        roles: roles.to_vec(),
        active_role,
        permissions: permissions.into_iter().collect(),
        linked_students: None,
    };
    let response_body = LoginResponse {
        access_token,
        expires_in: ACCESS_TOKEN_TTL_SECONDS,
        person,
        roles: roles.to_vec(),
        active_role,
        needs_totp,
    };
    let mut response = (StatusCode::OK, Json(response_body)).into_response();
    response
        .headers_mut()
        .insert(CACHE_CONTROL, HeaderValue::from_static("no-store"));
    Ok(response)
}

async fn lookup_refresh_identity(
    state: &AppState,
    tenant: TenantId,
    request_actor: &ActorCtx,
    token_hash: &[u8],
) -> Result<crate::repo::RefreshSessionIdentity, AppError> {
    let mut tx = begin(&state.pool, tenant, request_actor).await?;
    match crate::repo::refresh_session_identity(&mut tx, tenant, token_hash).await? {
        Some(identity) => {
            tx.commit().await?;
            Ok(identity)
        }
        None => {
            tx.rollback().await?;
            Err(session_expired_request())
        }
    }
}

fn valid_csrf(headers: &HeaderMap, expected_origin: &str) -> bool {
    let mut csrf_values = headers.get_all("x-eq-csrf").iter();
    let Some(csrf) = csrf_values.next() else {
        return false;
    };
    if csrf_values.next().is_some() || csrf.as_bytes() != b"1" {
        return false;
    }

    let mut origin_values = headers.get_all(ORIGIN).iter();
    let Some(origin) = origin_values.next() else {
        return false;
    };
    if origin_values.next().is_some() {
        return false;
    }
    let Ok(origin) = origin.to_str() else {
        return false;
    };
    origin == expected_origin
}

fn parse_refresh_cookie(headers: &HeaderMap) -> Option<SecretString> {
    let mut refresh_token = None;
    for header in headers.get_all(COOKIE) {
        let raw = header.to_str().ok()?;
        for pair in raw.split(';') {
            let Some((name, value)) = pair.trim().split_once('=') else {
                continue;
            };
            if name == "eq_rt" {
                if refresh_token.is_some() || value.is_empty() {
                    return None;
                }
                refresh_token = Some(SecretString::from(value.to_owned()));
            }
        }
    }
    refresh_token
}

fn unauthenticated_request() -> AppError {
    AppError::Unauthenticated {
        code: &eq_core::error::codes::UNAUTHENTICATED,
    }
}

fn session_expired_request() -> AppError {
    AppError::Unauthenticated {
        code: &codes::SESSION_EXPIRED,
    }
}

fn forbidden_request(detail: &'static str) -> AppError {
    AppError::ForbiddenRule {
        code: &eq_core::error::codes::FORBIDDEN,
        detail: String::from(detail),
    }
}

pub(crate) async fn login_success_response(
    state: &AppState,
    runtime: &LoginHttpState,
    tenant: TenantId,
    tx: &mut Tx,
    policies: &PolicyStore,
    success: &LoginSuccess,
) -> Result<Response, AppError> {
    let permissions = runtime
        .permissions(tx, tenant, success.person(), success.active_role())
        .await?;
    let include_pii = permissions.contains("identity.people.read_pii");
    let profile = state
        .auth
        .directory()
        .login_profile(tx, tenant, success.person(), include_pii, state.auth.keyring())
        .await?
        .ok_or(AppError::NotFound { entity: "person" })?;
    let policy = policies.current::<IdentityPolicy>(tx, tenant).await?;
    let max_age = u64::from(policy.doc.absolute_session_days())
        .checked_mul(SECONDS_PER_DAY)
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("absolute session cookie age overflowed")))?;
    let secure = !(state.config.public_url.scheme() == "http"
        && state.config.public_url.host_str() == Some("localhost"));
    let cookie = refresh_cookie(success.refresh_token().expose_secret(), max_age, secure);
    let cookie =
        HeaderValue::from_str(&cookie).map_err(|error| AppError::Internal(anyhow::Error::new(error)))?;
    let person = PersonMe {
        id: success.person(),
        display_name: profile.display_name,
        email: profile.email.map(|value| value.expose_secret().to_owned()),
        phone: profile.phone.map(|value| value.expose_secret().to_owned()),
        locale: profile.locale,
        avatar_url: None,
        roles: success.roles().to_vec(),
        active_role: success.active_role(),
        permissions: permissions.into_iter().collect(),
        linked_students: None,
    };
    let response_body = LoginResponse {
        access_token: success.access_token(),
        expires_in: ACCESS_TOKEN_TTL_SECONDS,
        person,
        roles: success.roles().to_vec(),
        active_role: success.active_role(),
        needs_totp: success.needs_totp_setup().then_some(true),
    };
    let mut response = (StatusCode::OK, Json(response_body)).into_response();
    response.headers_mut().append(SET_COOKIE, cookie);
    response
        .headers_mut()
        .insert(CACHE_CONTROL, HeaderValue::from_static("no-store"));
    Ok(response)
}

fn refresh_cookie(token: &str, max_age: u64, secure: bool) -> String {
    let secure_attribute = if secure { "; Secure" } else { "" };
    format!(
        "eq_rt={token}; HttpOnly{secure_attribute}; SameSite=Strict; Path=/api/v1/auth; Max-Age={max_age}"
    )
}

fn actor_role(role: Role) -> ActorRole {
    match role {
        Role::Owner => ActorRole::Owner,
        Role::Admin => ActorRole::Admin,
        Role::Instructor => ActorRole::Instructor,
        Role::Guardian => ActorRole::Guardian,
        Role::Student => ActorRole::Student,
    }
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

fn has_role_grant(permission: &PermissionDef, role: ActorRole, owner_without_admin: bool) -> bool {
    permission.roles.contains(&role)
        || (owner_without_admin && role == ActorRole::Owner && permission.roles.contains(&ActorRole::Admin))
}

#[cfg(test)]
mod tests {
    use super::{
        LoginBody, has_role_grant, parse_bearer_token, parse_refresh_cookie, refresh_cookie, valid_csrf,
    };
    use axum::extract::Request;
    use axum::http::{Method, StatusCode};
    use axum::middleware::Next;
    use axum::response::Response;
    use base64::Engine;
    use eq_core::actor::ActorRole;
    use eq_core::module::PermissionDef;
    use secrecy::ExposeSecret;
    use serde_json::json;
    use tower::ServiceExt;

    async fn tenant_resolution_marker(request: Request, next: Next) -> Response {
        let mut response = next.run(request).await;
        response.headers_mut().insert(
            "x-test-tenant-resolved",
            axum::http::HeaderValue::from_static("1"),
        );
        response
    }

    #[test]
    fn refresh_cookie_uses_canonical_attributes() {
        let cookie = refresh_cookie("test-secret-refresh", 2_592_000, true);
        assert_eq!(
            cookie,
            "eq_rt=test-secret-refresh; HttpOnly; Secure; SameSite=Strict; Path=/api/v1/auth; Max-Age=2592000"
        );
        let local_cookie = refresh_cookie("test-secret-refresh", 2_592_000, false);
        assert!(!local_cookie.contains("Secure"));
    }

    #[test]
    fn owner_without_admin_inherits_admin_permission_grants() {
        static ADMIN_ROLES: &[ActorRole] = &[ActorRole::Admin];
        let permission = PermissionDef {
            key: "identity.people.read_pii",
            roles: ADMIN_ROLES,
        };
        assert!(has_role_grant(&permission, ActorRole::Owner, true));
        assert!(!has_role_grant(&permission, ActorRole::Owner, false));
        assert!(has_role_grant(&permission, ActorRole::Admin, false));
    }

    #[test]
    fn login_request_debug_redacts_credentials() {
        let parsed = serde_json::from_value::<LoginBody>(json!({
            "email": "test-secret-person@example.test",
            "password": "test-secret-password",
            "totp": "123456"
        }));
        assert!(parsed.is_ok());
        let Ok(body) = parsed else {
            return;
        };
        let formatted = format!("{body:?}");
        assert!(!formatted.contains("test-secret-person"));
        assert!(!formatted.contains("test-secret-password"));
        assert!(!formatted.contains("123456"));
    }

    #[test]
    fn bearer_token_debug_redacts_credentials() {
        let header = axum::http::HeaderValue::from_static("Bearer test-secret-access-token");
        let Some(token) = parse_bearer_token(&header) else {
            panic!("canonical bearer token parses");
        };
        assert!(!format!("{token:?}").contains("test-secret-access-token"));
    }

    #[test]
    fn malformed_bearer_header_rejected() {
        for value in [
            "bearer test-secret-access-token",
            "Bearer ",
            "Bearer  test-secret-access-token",
            "Bearer test-secret-access-token trailing",
        ] {
            let header = axum::http::HeaderValue::from_static(value);
            assert!(parse_bearer_token(&header).is_none());
        }
    }

    #[test]
    fn role_switch_csrf_requires_exact_header_and_origin() {
        let missing = axum::http::HeaderMap::new();
        assert!(!valid_csrf(&missing, "https://example.test"));

        let mut valid = axum::http::HeaderMap::new();
        valid.insert("x-eq-csrf", axum::http::HeaderValue::from_static("1"));
        valid.insert(
            axum::http::header::ORIGIN,
            axum::http::HeaderValue::from_static("https://example.test"),
        );
        assert!(valid_csrf(&valid, "https://example.test"));

        valid.append("x-eq-csrf", axum::http::HeaderValue::from_static("1"));
        assert!(!valid_csrf(&valid, "https://example.test"));
        valid.remove("x-eq-csrf");
        valid.insert("x-eq-csrf", axum::http::HeaderValue::from_static("1"));
        valid.append(
            axum::http::header::ORIGIN,
            axum::http::HeaderValue::from_static("https://example.test"),
        );
        assert!(!valid_csrf(&valid, "https://example.test"));
        valid.remove(axum::http::header::ORIGIN);
        valid.insert(
            axum::http::header::ORIGIN,
            axum::http::HeaderValue::from_static("https://example.test"),
        );

        valid.insert("x-eq-csrf", axum::http::HeaderValue::from_static("01"));
        assert!(!valid_csrf(&valid, "https://example.test"));
        valid.insert("x-eq-csrf", axum::http::HeaderValue::from_static("1"));
        valid.insert(
            axum::http::header::ORIGIN,
            axum::http::HeaderValue::from_static("https://example.test/"),
        );
        assert!(!valid_csrf(&valid, "https://example.test"));
    }

    #[tokio::test]
    async fn role_switch_csrf_guard_runs_before_tenant_resolution() {
        let router = axum::Router::new()
            .route(
                "/api/v1/auth/switch-role",
                axum::routing::post(|| async { StatusCode::NO_CONTENT }),
            )
            .route(
                "/api/v1/auth/refresh",
                axum::routing::post(|| async { StatusCode::NO_CONTENT }),
            )
            .layer(axum::middleware::from_fn(tenant_resolution_marker))
            .layer(axum::middleware::from_fn_with_state(
                String::from("https://example.test"),
                super::role_switch_csrf_guard,
            ));
        for path in ["/api/v1/auth/switch-role", "/api/v1/auth/refresh"] {
            let request = axum::http::Request::builder()
                .method(Method::POST)
                .uri(path)
                .header(axum::http::header::ORIGIN, "https://attacker.example")
                .header("x-eq-csrf", "1")
                .body(axum::body::Body::empty())
                .expect("fixed test request is valid");
            let response = router
                .clone()
                .oneshot(request)
                .await
                .expect("router accepts request");

            assert_eq!(response.status(), StatusCode::FORBIDDEN);
            assert!(!response.headers().contains_key("x-test-tenant-resolved"));
        }
    }

    #[test]
    fn refresh_cookie_secret_debug_is_redacted_and_canonical_digest_is_required() {
        let fake_secret = "test-secret-refresh-cookie";
        let mut headers = axum::http::HeaderMap::new();
        assert!(parse_refresh_cookie(&headers).is_none());
        headers.insert(
            axum::http::header::COOKIE,
            axum::http::HeaderValue::from_static("eq_rt"),
        );
        assert!(parse_refresh_cookie(&headers).is_none());
        headers.insert(
            axum::http::header::COOKIE,
            axum::http::HeaderValue::from_static("eq_rt=test-secret-refresh-cookie"),
        );
        let Some(token) = parse_refresh_cookie(&headers) else {
            panic!("refresh cookie parses");
        };
        assert!(!format!("{token:?}").contains(fake_secret));
        assert!(crate::service::refresh_token_digest(&token).is_none());

        let canonical = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode([7_u8; 32]);
        headers.insert(
            axum::http::header::COOKIE,
            axum::http::HeaderValue::from_str(&format!("eq_rt={canonical}"))
                .expect("test token is a valid header"),
        );
        let Some(token) = parse_refresh_cookie(&headers) else {
            panic!("canonical refresh cookie parses");
        };
        assert_eq!(token.expose_secret(), canonical);
        assert_eq!(
            crate::service::refresh_token_digest(&token).map(|hash| hash.len()),
            Some(32)
        );

        headers.append(
            axum::http::header::COOKIE,
            axum::http::HeaderValue::from_str(&format!("eq_rt={canonical}"))
                .expect("test token is a valid header"),
        );
        assert!(parse_refresh_cookie(&headers).is_none());
    }
}
