//! IDN-01/03/04/06: tenant-aware HTTP login boundary.

use std::collections::BTreeSet;
use std::fmt;
use std::sync::Arc;

use crate::errors::codes;
use axum::Json;
use axum::extract::{Extension, State};
use axum::http::header::{CACHE_CONTROL, SET_COOKIE, USER_AGENT};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono_tz::Tz;
use eq_core::actor::ActorRole;
use eq_core::app_state::AppState;
use eq_core::clock::Clock;
use eq_core::db::{Tx, begin};
use eq_core::error::AppError;
use eq_core::fields::{FieldCtx, FieldKind, Password, PasswordInput};
use eq_core::ids::TenantId;
use eq_core::model::Role;
use eq_core::module::gate::EnabledModuleSet;
use eq_core::module::{ModuleRegistry, PermissionDef};
use eq_core::outbox::OutboxEventSink;
use eq_core::policy::types::Locale;
use eq_core::policy::{PolicyStore, PresetPatchSource, PresetSourceError};
use eq_core::ports::sinks::{AuditSink, EventSink};
use secrecy::ExposeSecret;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
use serde_json::Value;
use sqlx::PgPool;
use tokio::sync::OnceCell;
use utoipa::ToSchema;

use crate::policy::IdentityPolicy;
use crate::service::{LoginRequest, LoginService, LoginSuccess};

const ACCESS_TOKEN_TTL_SECONDS: u64 = 600; // IDN-03: technical access-token lifetime.
const SECONDS_PER_DAY: u64 = 86_400; // technical: convert the policy's absolute session days to cookie Max-Age.

/// IDN-06/POL-04: shared lazy policy runtime used by login routes.
pub struct LoginHttpState {
    pool: PgPool,
    clock: Arc<dyn Clock>,
    modules: Arc<ModuleRegistry>,
    enabled: Arc<dyn EnabledModuleSet>,
    policy_store: OnceCell<PolicyStore>,
}

impl LoginHttpState {
    /// IDN-06/POL-04: constructs one process-scoped runtime without requiring PostgreSQL at router startup.
    pub fn new(
        pool: PgPool,
        clock: Arc<dyn Clock>,
        modules: Arc<ModuleRegistry>,
        enabled: Arc<dyn EnabledModuleSet>,
    ) -> Self {
        Self {
            pool,
            clock,
            modules,
            enabled,
            policy_store: OnceCell::new(),
        }
    }

    async fn policy_store(&self) -> Result<&PolicyStore, AppError> {
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
        tenant: TenantId,
        role: Role,
        owner_without_admin: bool,
    ) -> Result<BTreeSet<String>, AppError> {
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
        Ok(permissions)
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
struct TotpCode(secrecy::SecretString);

impl<'de> Deserialize<'de> for TotpCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer).map(|value| Self(secrecy::SecretString::from(value)))
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
    pub id: eq_core::ids::PersonId,
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
    pub id: eq_core::ids::PersonId,
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
struct LoginResponseSchema {
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

async fn login_success_response(
    state: &AppState,
    runtime: &LoginHttpState,
    tenant: TenantId,
    tx: &mut Tx,
    policies: &PolicyStore,
    success: &LoginSuccess,
) -> Result<Response, AppError> {
    let owner_without_admin = if success.active_role() == Role::Owner {
        !state.auth.directory().has_active_admin(tx, tenant).await?
    } else {
        false
    };
    let permissions = runtime
        .permissions(tenant, success.active_role(), owner_without_admin)
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

fn has_role_grant(permission: &PermissionDef, role: ActorRole, owner_without_admin: bool) -> bool {
    permission.roles.contains(&role)
        || (owner_without_admin && role == ActorRole::Owner && permission.roles.contains(&ActorRole::Admin))
}

#[cfg(test)]
mod tests {
    use super::{LoginBody, has_role_grant, refresh_cookie};
    use eq_core::actor::ActorRole;
    use eq_core::module::PermissionDef;
    use serde_json::json;

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
}
