//! IDN-13/AUTH-05: authenticated self-profile and device-session boundaries.

use std::fmt;
use std::num::NonZeroI32;
use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, Path, State};
use axum::http::header::{CACHE_CONTROL, ETAG, IF_MATCH};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, Utc};
use eq_audit::{AuditEvent, Category, record};
use eq_core::app_state::AppState;
use eq_core::auth::Actor;
use eq_core::db::{Tx, begin};
use eq_core::error::{AppError, FieldError, codes};
use eq_core::fields::{DisplayName, FieldCtx, FieldIssue, FieldKind, Phone};
use eq_core::ids::PersonId;
use eq_core::model::Role;
use eq_core::policy::types::Locale;
use eq_core::tenant::{RequireTenant, TenantInfo};
use secrecy::ExposeSecret;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Map;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api_auth::{LinkedStudent, LoginHttpState};
use crate::repo;
use crate::service::PasswordChangeService;

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
#[expect(
    dead_code,
    reason = "the enum supplies the canonical role schema for the self-profile DTO"
)]
enum RoleSchema {
    Owner,
    Admin,
    Instructor,
    Guardian,
    Student,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "lowercase")]
#[expect(
    dead_code,
    reason = "the enum supplies the canonical locale schema for self-profile requests"
)]
enum LocaleSchema {
    En,
    Id,
}

/// API-07: self profile response with the current aggregate version.
#[derive(Serialize, ToSchema)]
pub(crate) struct MeResponse {
    #[schema(value_type = String, format = Uuid)]
    id: PersonId,
    display_name: String,
    email: Option<String>,
    phone: Option<String>,
    #[schema(value_type = Option<LocaleSchema>)]
    locale: Option<Locale>,
    avatar_url: Option<String>,
    #[schema(value_type = Vec<RoleSchema>)]
    roles: Vec<Role>,
    #[schema(value_type = RoleSchema)]
    active_role: Role,
    permissions: Vec<String>,
    #[schema(value_type = Option<Vec<LinkedStudent>>)]
    #[serde(skip_serializing_if = "Option::is_none")]
    linked_students: Option<Vec<LinkedStudent>>,
    version: i32,
}

impl fmt::Debug for MeResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MeResponse")
            .field("id", &"[REDACTED]")
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
            .field("version", &self.version)
            .finish()
    }
}

#[derive(ToSchema)]
#[serde(deny_unknown_fields)]
#[expect(
    dead_code,
    reason = "utoipa reads these fields to generate the PATCH request schema"
)]
struct ProfilePatchSchema {
    display_name: Option<String>,
    phone: Option<String>,
    #[schema(value_type = Option<LocaleSchema>)]
    locale: Option<LocaleSchema>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProfilePatchBody {
    #[serde(default)]
    display_name: PatchField<String>,
    #[serde(default)]
    phone: PatchField<String>,
    #[serde(default)]
    locale: PatchField<Locale>,
}

impl fmt::Debug for ProfilePatchBody {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProfilePatchBody")
            .field("display_name", &"[REDACTED]")
            .field("phone", &"[REDACTED]")
            .field("locale", &"[PRESENT]")
            .finish()
    }
}

#[derive(Default)]
enum PatchField<T> {
    #[default]
    Missing,
    Null,
    Value(T),
}

impl<'de, T> Deserialize<'de> for PatchField<T>
where
    T: DeserializeOwned,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(match Option::<T>::deserialize(deserializer)? {
            Some(value) => Self::Value(value),
            None => Self::Null,
        })
    }
}

#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct ChangePasswordBody {
    current_password: eq_core::fields::PasswordInput,
    new_password: eq_core::fields::PasswordInput,
}

impl fmt::Debug for ChangePasswordBody {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ChangePasswordBody")
            .field("current_password", &"[REDACTED]")
            .field("new_password", &"[REDACTED]")
            .finish()
    }
}

/// SBD-50: session S2 fields are omitted unless the active role may read PII.
#[derive(Serialize, ToSchema)]
pub(crate) struct DeviceSessionResponse {
    #[schema(value_type = String, format = Uuid)]
    id: Uuid,
    #[schema(value_type = RoleSchema)]
    active_role: Role,
    created_at: DateTime<Utc>,
    last_used_at: DateTime<Utc>,
    idle_expires_at: DateTime<Utc>,
    absolute_expires_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_agent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ip: Option<String>,
}

impl fmt::Debug for DeviceSessionResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DeviceSessionResponse")
            .field("id", &"[REDACTED]")
            .field("active_role", &self.active_role)
            .field("created_at", &self.created_at)
            .field("last_used_at", &self.last_used_at)
            .field("idle_expires_at", &self.idle_expires_at)
            .field("absolute_expires_at", &self.absolute_expires_at)
            .field("user_agent", &self.user_agent.as_ref().map(|_| "[REDACTED]"))
            .field("ip", &self.ip.as_ref().map(|_| "[REDACTED]"))
            .finish()
    }
}

#[derive(Serialize, ToSchema)]
pub(crate) struct DeviceSessionsResponse {
    sessions: Vec<DeviceSessionResponse>,
}

#[utoipa::path(
    get,
    path = "/api/v1/me",
    responses((status = 200, description = "Authenticated person's profile", body = MeResponse)),
    tag = "identity"
)]
pub(crate) async fn get_me(State(state): State<AppState>, actor: Actor) -> Result<Response, AppError> {
    actor.require("identity.people.read")?;
    let mut tx = begin(&state.pool, actor.tenant(), actor.context()).await?;
    match me_from_tx(&mut tx, &state, &actor).await {
        Ok(me) => {
            tx.commit().await?;
            response_with_etag(StatusCode::OK, &me, me.version)
        }
        Err(error) => {
            tx.rollback().await?;
            Err(error)
        }
    }
}

#[utoipa::path(
    patch,
    path = "/api/v1/me",
    request_body = ProfilePatchSchema,
    responses(
        (status = 200, description = "Updated self profile", body = MeResponse),
        (status = 400, description = "Malformed If-Match or profile field"),
        (status = 409, description = "Profile version conflict"),
        (status = 428, description = "If-Match is required")
    ),
    tag = "identity"
)]
pub(crate) async fn patch_me(
    State(state): State<AppState>,
    RequireTenant(tenant): RequireTenant,
    actor: Actor,
    headers: HeaderMap,
    Json(body): Json<ProfilePatchBody>,
) -> Result<Response, AppError> {
    actor.require("identity.people.read")?;
    let expected_version = parse_if_match(&headers)?;
    let field_ctx = field_context(&tenant, state.clock.now());
    let patch = parse_profile_patch(body, &field_ctx)?;
    let mut tx = begin(&state.pool, actor.tenant(), actor.context()).await?;
    let result = async {
        let current = me_from_tx(&mut tx, &state, &actor).await?;
        if current.version != expected_version {
            return Err(version_conflict(&current)?);
        }
        if patch.display_name.is_none() && patch.phone.is_none() && patch.locale.is_none() {
            return Ok(current);
        }
        let now = state.clock.now();
        let updated_version = state
            .auth
            .directory()
            .update_login_profile(
                &mut tx,
                actor.tenant(),
                actor.person_id(),
                eq_core::auth::directory::LoginProfileUpdate::new(
                    NonZeroI32::new(expected_version).ok_or_else(bad_if_match)?,
                    patch.display_name,
                    patch.phone,
                    patch.locale,
                    now,
                ),
                state.auth.keyring(),
            )
            .await?;
        let Some(updated_version) = updated_version else {
            let current = me_from_tx(&mut tx, &state, &actor).await?;
            return Err(version_conflict(&current)?);
        };
        record(
            &mut tx,
            AuditEvent {
                action: "core.person.updated",
                category: Category::Operational,
                entity_type: "person",
                entity_id: Some(actor.person_id().0),
                before: Some(serde_json::json!({ "version": current.version })),
                after: Some(serde_json::json!({ "version": updated_version })),
                actor: actor.context(),
            },
        )
        .await?;
        me_from_tx(&mut tx, &state, &actor).await
    }
    .await;
    match result {
        Ok(me) => {
            tx.commit().await?;
            response_with_etag(StatusCode::OK, &me, me.version)
        }
        Err(error) => {
            tx.rollback().await?;
            Err(error)
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/me/password",
    request_body = ChangePasswordBody,
    responses(
        (status = 204, description = "Password changed and other sessions revoked"),
        (status = 401, description = "Current password is invalid"),
        (status = 422, description = "New password does not meet policy")
    ),
    tag = "identity"
)]
pub(crate) async fn change_password(
    State(state): State<AppState>,
    Extension(runtime): Extension<Arc<LoginHttpState>>,
    RequireTenant(tenant): RequireTenant,
    actor: Actor,
    Json(body): Json<ChangePasswordBody>,
) -> Result<Response, AppError> {
    actor.require("identity.people.read")?;
    let field_ctx = field_context(&tenant, state.clock.now());
    let policies = runtime.policy_store().await?;
    PasswordChangeService::new(&state.pool, policies, state.clock.as_ref())
        .change(
            actor.tenant(),
            actor.person_id(),
            actor.session_id(),
            body.current_password,
            body.new_password,
            &field_ctx,
            actor.context(),
        )
        .await?;
    runtime
        .invalidate_password_version(actor.tenant(), actor.person_id())
        .await;
    Ok(no_store(StatusCode::NO_CONTENT.into_response()))
}

#[utoipa::path(
    get,
    path = "/api/v1/me/sessions",
    responses((status = 200, description = "Active device sessions", body = DeviceSessionsResponse)),
    tag = "identity"
)]
pub(crate) async fn list_sessions(State(state): State<AppState>, actor: Actor) -> Result<Response, AppError> {
    actor.require("identity.people.read")?;
    let mut tx = begin(&state.pool, actor.tenant(), actor.context()).await?;
    let sessions =
        repo::active_device_sessions(&mut tx, actor.tenant(), actor.person_id(), state.clock.now()).await;
    match sessions {
        Ok(sessions) => {
            tx.commit().await?;
            let include_pii = actor.has("identity.people.read_pii");
            let response = DeviceSessionsResponse {
                sessions: sessions
                    .into_iter()
                    .map(|session| DeviceSessionResponse {
                        id: session.id,
                        active_role: session.active_role,
                        created_at: session.created_at,
                        last_used_at: session.last_used_at,
                        idle_expires_at: session.idle_expires_at,
                        absolute_expires_at: session.absolute_expires_at,
                        user_agent: include_pii.then_some(session.user_agent).flatten(),
                        ip: include_pii.then_some(session.ip).flatten(),
                    })
                    .collect(),
            };
            Ok(no_store((StatusCode::OK, Json(response)).into_response()))
        }
        Err(error) => {
            tx.rollback().await?;
            Err(error)
        }
    }
}

#[utoipa::path(
    delete,
    path = "/api/v1/me/sessions/{id}",
    params(("id" = Uuid, Path, description = "Owned refresh-session id")),
    responses(
        (status = 204, description = "Session family revoked"),
        (status = 404, description = "Session not found")
    ),
    tag = "identity"
)]
pub(crate) async fn revoke_session(
    State(state): State<AppState>,
    actor: Actor,
    Path(session_id): Path<Uuid>,
) -> Result<Response, AppError> {
    actor.require("identity.people.read")?;
    let mut tx = begin(&state.pool, actor.tenant(), actor.context()).await?;
    let result = async {
        let Some(family_id) =
            repo::refresh_family_for_person(&mut tx, actor.tenant(), actor.person_id(), session_id).await?
        else {
            return Err(AppError::NotFound { entity: "session" });
        };
        let revoked = repo::revoke_refresh_family_for_person(
            &mut tx,
            actor.tenant(),
            actor.person_id(),
            family_id,
            state.clock.now(),
        )
        .await?;
        if revoked == 0 {
            return Err(AppError::NotFound { entity: "session" });
        }
        record(
            &mut tx,
            AuditEvent {
                action: "security.sessions_revoked",
                category: Category::Security,
                entity_type: "refresh_session",
                entity_id: Some(session_id),
                before: None,
                after: Some(serde_json::json!({ "revoked": true })),
                actor: actor.context(),
            },
        )
        .await?;
        Ok::<(), AppError>(())
    }
    .await;
    match result {
        Ok(()) => {
            tx.commit().await?;
            Ok(no_store(StatusCode::NO_CONTENT.into_response()))
        }
        Err(error) => {
            tx.rollback().await?;
            Err(error)
        }
    }
}

async fn me_from_tx(tx: &mut Tx, state: &AppState, actor: &Actor) -> Result<MeResponse, AppError> {
    let tenant = actor.tenant();
    let person = actor.person_id();
    let directory = state.auth.directory();
    let identity = directory
        .identity_by_person(tx, tenant, person)
        .await?
        .ok_or(AppError::NotFound { entity: "person" })?;
    let profile = directory
        .login_profile(
            tx,
            tenant,
            person,
            actor.has("identity.people.read_pii"),
            state.auth.keyring(),
        )
        .await?
        .ok_or(AppError::NotFound { entity: "person" })?;
    let linked_students = if actor.role() == Role::Guardian {
        Some(
            directory
                .linked_students(tx, tenant, person)
                .await?
                .into_iter()
                .map(|student| LinkedStudent {
                    id: student.person_id,
                    display_name: student.display_name,
                })
                .collect(),
        )
    } else {
        None
    };
    Ok(MeResponse {
        id: person,
        display_name: profile.display_name,
        email: profile.email.map(|value| value.expose_secret().to_owned()),
        phone: profile.phone.map(|value| value.expose_secret().to_owned()),
        locale: profile.locale,
        avatar_url: None,
        roles: identity.roles().to_vec(),
        active_role: actor.role(),
        permissions: actor.permissions().iter().cloned().collect(),
        linked_students,
        version: profile.version,
    })
}

fn response_with_etag<T: Serialize>(
    status: StatusCode,
    body: &T,
    version: i32,
) -> Result<Response, AppError> {
    let mut response = (status, Json(body)).into_response();
    let value = HeaderValue::from_str(&format!("\"v{version}\""))
        .map_err(|error| AppError::Internal(anyhow::Error::new(error)))?;
    response.headers_mut().insert(ETAG, value);
    response
        .headers_mut()
        .insert(CACHE_CONTROL, HeaderValue::from_static("no-store"));
    Ok(response)
}

fn no_store(mut response: Response) -> Response {
    response
        .headers_mut()
        .insert(CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}

fn parse_if_match(headers: &HeaderMap) -> Result<i32, AppError> {
    let Some(value) = headers.get(IF_MATCH) else {
        return Err(AppError::PreconditionRequired);
    };
    let value = value.to_str().map_err(|_| bad_if_match())?;
    let Some(version) = value
        .strip_prefix("\"v")
        .and_then(|value| value.strip_suffix('"'))
    else {
        return Err(bad_if_match());
    };
    let parsed = version.parse::<i32>().map_err(|_| bad_if_match())?;
    if parsed <= 0 || parsed.to_string() != version {
        return Err(bad_if_match());
    }
    Ok(parsed)
}

fn bad_if_match() -> AppError {
    AppError::BadRequest {
        code: &codes::BAD_REQUEST,
        detail: String::from("If-Match must be a strong version ETag"),
    }
}

fn version_conflict(current: &MeResponse) -> Result<AppError, AppError> {
    let mut extra = Map::new();
    extra.insert(
        "current".to_owned(),
        serde_json::to_value(current).map_err(anyhow::Error::new)?,
    );
    Ok(AppError::Conflict {
        code: &codes::VERSION_CONFLICT,
        detail: String::from("The profile has changed; reload it before retrying"),
        extra,
    })
}

struct ParsedProfilePatch {
    display_name: Option<DisplayName>,
    phone: Option<Option<Phone>>,
    locale: Option<Option<Locale>>,
}

fn parse_profile_patch(body: ProfilePatchBody, context: &FieldCtx) -> Result<ParsedProfilePatch, AppError> {
    let display_name = match body.display_name {
        PatchField::Missing => None,
        PatchField::Null => {
            return Err(validation_error(
                "/display_name",
                vec![FieldIssue::new(eq_core::errors::REQUIRED)],
            ));
        }
        PatchField::Value(value) => Some(
            DisplayName::parse(value, context).map_err(|issues| validation_error("/display_name", issues))?,
        ),
    };
    let phone = match body.phone {
        PatchField::Missing => None,
        PatchField::Null => Some(None),
        PatchField::Value(value) => {
            let phone = Phone::try_from(value).map_err(|_| {
                validation_error(
                    "/phone",
                    vec![FieldIssue::bound(
                        eq_core::errors::INVALID_FORMAT,
                        "format",
                        "phone",
                    )],
                )
            })?;
            Some(Some(phone))
        }
    };
    let locale = match body.locale {
        PatchField::Missing => None,
        PatchField::Null => Some(None),
        PatchField::Value(value) => Some(Some(value)),
    };
    Ok(ParsedProfilePatch {
        display_name,
        phone,
        locale,
    })
}

fn validation_error(pointer: &str, issues: Vec<FieldIssue>) -> AppError {
    let errors = issues
        .into_iter()
        .map(|issue| FieldError {
            pointer: issue
                .pointer
                .map(|child| format!("{pointer}{child}"))
                .unwrap_or_else(|| pointer.to_owned()),
            code: issue.code.to_owned(),
            params: issue.params,
        })
        .collect();
    AppError::Validation { errors }
}

fn field_context(tenant: &TenantInfo, now: DateTime<Utc>) -> FieldCtx {
    FieldCtx {
        country: None,
        tenant_currency: tenant.currency,
        tenant_locale: tenant.locale,
        today: now.date_naive(),
        password_min_length: None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ChangePasswordBody, DeviceSessionResponse, MeResponse, NonZeroI32, PatchField, ProfilePatchBody,
        parse_if_match,
    };
    use axum::http::HeaderMap;
    use axum::http::header::IF_MATCH;
    use chrono::{TimeZone, Utc};
    use eq_core::auth::directory::LoginProfileUpdate;
    use eq_core::fields::{DisplayName, FieldCtx, FieldKind, Phone};
    use eq_core::ids::PersonId;
    use eq_core::model::Role;
    use eq_core::money::Currency;
    use eq_core::policy::types::Locale;
    use uuid::Uuid;

    #[test]
    fn if_match_accepts_only_canonical_strong_versions() {
        let mut headers = HeaderMap::new();
        headers.insert(IF_MATCH, "\"v12\"".parse().expect("valid test ETag"));
        assert_eq!(parse_if_match(&headers).expect("canonical ETag"), 12);
        for invalid in ["*", "W/\"v1\"", "\"v0\"", "\"v01\"", "\"v1\", \"v2\""] {
            headers.insert(IF_MATCH, invalid.parse().expect("valid header value"));
            assert!(parse_if_match(&headers).is_err());
        }
        headers.remove(IF_MATCH);
        assert!(parse_if_match(&headers).is_err());
    }

    #[test]
    fn patch_field_distinguishes_missing_null_and_value() {
        let missing = serde_json::from_str::<ProfilePatchBody>("{}").expect("empty patch");
        let null: PatchField<String> = serde_json::from_str("null").expect("null patch field");
        let value: PatchField<String> =
            serde_json::from_str("\"test-secret-value\"").expect("value patch field");
        assert!(matches!(missing.display_name, PatchField::Missing));
        assert!(matches!(null, PatchField::Null));
        assert!(matches!(value, PatchField::Value(value) if value == "test-secret-value"));
    }

    #[test]
    fn self_profile_request_and_session_debug_redact_personal_values() {
        let password: ChangePasswordBody = serde_json::from_value(serde_json::json!({
            "current_password": "test-secret-current-password",
            "new_password": "test-secret-new-password"
        }))
        .expect("fake password fields deserialize");
        assert!(!format!("{password:?}").contains("test-secret-current-password"));
        assert!(!format!("{password:?}").contains("test-secret-new-password"));

        let field_context = FieldCtx {
            country: None,
            tenant_currency: Currency::USD,
            tenant_locale: Locale::En,
            today: Utc
                .timestamp_opt(0, 0)
                .single()
                .expect("valid test time")
                .date_naive(),
            password_min_length: None,
        };
        let display_name = DisplayName::parse(String::from("test-secret-display-name"), &field_context)
            .expect("valid test display name");
        let phone = Phone::try_from(String::from("+12025550123")).expect("valid test phone");
        let update = LoginProfileUpdate::new(
            NonZeroI32::new(1).expect("valid test version"),
            Some(display_name),
            Some(Some(phone)),
            None,
            Utc.timestamp_opt(0, 0).single().expect("valid test time"),
        );
        let update_debug = format!("{update:?}");
        assert!(!update_debug.contains("test-secret-display-name"));
        assert!(!update_debug.contains("+12025550123"));

        let profile = MeResponse {
            id: PersonId(Uuid::from_u128(1)),
            display_name: String::from("test-secret-display-name"),
            email: Some(String::from("test-secret-email@example.test")),
            phone: Some(String::from("+12025550123")),
            locale: Some(Locale::En),
            avatar_url: None,
            roles: vec![Role::Guardian],
            active_role: Role::Guardian,
            permissions: Vec::new(),
            linked_students: None,
            version: 1,
        };
        let profile_debug = format!("{profile:?}");
        assert!(!profile_debug.contains("test-secret-display-name"));
        assert!(!profile_debug.contains("test-secret-email@example.test"));
        assert!(!profile_debug.contains("+12025550123"));

        let session = DeviceSessionResponse {
            id: Uuid::from_u128(2),
            active_role: Role::Guardian,
            created_at: Utc
                .with_ymd_and_hms(2026, 1, 1, 0, 0, 0)
                .single()
                .expect("fixed test timestamp"),
            last_used_at: Utc
                .with_ymd_and_hms(2026, 1, 1, 0, 0, 0)
                .single()
                .expect("fixed test timestamp"),
            idle_expires_at: Utc
                .with_ymd_and_hms(2026, 1, 2, 0, 0, 0)
                .single()
                .expect("fixed test timestamp"),
            absolute_expires_at: Utc
                .with_ymd_and_hms(2026, 2, 1, 0, 0, 0)
                .single()
                .expect("fixed test timestamp"),
            user_agent: Some(String::from("test-secret-browser")),
            ip: Some(String::from("192.0.2.10")),
        };
        let session_debug = format!("{session:?}");
        assert!(!session_debug.contains("test-secret-browser"));
        assert!(!session_debug.contains("192.0.2.10"));
    }
}
