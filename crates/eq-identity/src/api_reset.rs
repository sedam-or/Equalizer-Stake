//! IDN-08: non-enumerating password recovery HTTP boundaries.

#![expect(
    unused_qualifications,
    reason = "utoipa path macros emit crate-qualified request DTO references in generated OpenAPI metadata"
)]

use std::borrow::Cow;
use std::fmt;
use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use chrono_tz::Tz;
use eq_core::app_state::AppState;
use eq_core::error::AppError;
use eq_core::fields::{FieldCtx, Profile, SchemaBounds, string_schema};
use eq_core::tenant::RequireTenant;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Deserializer};
use utoipa::{PartialSchema, ToSchema};

use crate::api_auth::LoginHttpState;
use crate::service::{PasswordResetService, ResetToken};

/// SBD-50: untrusted reset token input stays in secret memory and is redacted by Debug.
struct ResetTokenInput(SecretString);

impl<'de> Deserialize<'de> for ResetTokenInput {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer).map(|value| Self(SecretString::from(value)))
    }
}

impl PartialSchema for ResetTokenInput {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        string_schema(
            "ResetToken",
            Profile::Secret,
            SchemaBounds {
                min_length: Some(43),
                max_length: Some(43),
                minimum: None,
                maximum: None,
            },
            Some("^[A-Za-z0-9_-]{43}$"),
        )
        .into()
    }
}

impl ToSchema for ResetTokenInput {
    fn name() -> Cow<'static, str> {
        Cow::Borrowed("ResetToken")
    }

    fn schemas(schemas: &mut Vec<(String, utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>)>) {
        schemas.push((Self::name().into_owned(), Self::schema()));
    }
}

impl fmt::Debug for ResetTokenInput {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ResetTokenInput([REDACTED])")
    }
}

#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct ForgotBody {
    email: String,
}

impl fmt::Debug for ForgotBody {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ForgotBody")
            .field("email", &"[REDACTED]")
            .finish()
    }
}

#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct ResetBody {
    token: ResetTokenInput,
    new_password: eq_core::fields::PasswordInput,
}

impl fmt::Debug for ResetBody {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ResetBody")
            .field("token", &"[REDACTED]")
            .field("new_password", &"[REDACTED]")
            .finish()
    }
}

/// IDN-08/AUTH-06: returns the same empty 202 response for valid, unknown and malformed addresses.
#[utoipa::path(
    post,
    path = "/api/v1/auth/forgot",
    request_body = ForgotBody,
    responses((status = 202, description = "Recovery request accepted")),
    security(()),
    tag = "identity"
)]
pub(crate) async fn forgot(
    State(state): State<AppState>,
    Extension(runtime): Extension<Arc<LoginHttpState>>,
    RequireTenant(tenant): RequireTenant,
    Json(body): Json<ForgotBody>,
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
    let policies = runtime.policy_store().await?;
    PasswordResetService::new(
        &state.pool,
        state.auth.directory(),
        policies,
        state.clock.as_ref(),
    )
    .forgot(tenant.id, &body.email, &field_ctx)
    .await?;
    Ok(StatusCode::ACCEPTED.into_response())
}

/// IDN-08/STM-24: validates and consumes one reset token, revoking all tenant refresh sessions.
#[utoipa::path(
    post,
    path = "/api/v1/auth/reset",
    request_body = ResetBody,
    responses(
        (status = 204, description = "Password reset completed"),
        (status = 400, description = "Reset token is invalid"),
        (status = 422, description = "Password does not meet the current policy")
    ),
    security(()),
    tag = "identity"
)]
pub(crate) async fn reset(
    State(state): State<AppState>,
    Extension(runtime): Extension<Arc<LoginHttpState>>,
    RequireTenant(tenant): RequireTenant,
    Json(body): Json<ResetBody>,
) -> Result<Response, AppError> {
    let token =
        ResetToken::try_from(body.token.0.expose_secret().to_owned()).map_err(|_| AppError::BadRequest {
            code: &crate::errors::codes::TOKEN_INVALID,
            detail: String::from("reset token is invalid"),
        })?;
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
    let policies = runtime.policy_store().await?;
    let person = PasswordResetService::new(
        &state.pool,
        state.auth.directory(),
        policies,
        state.clock.as_ref(),
    )
    .reset(tenant.id, token, body.new_password, &field_ctx)
    .await?;
    runtime.invalidate_password_version(tenant.id, person).await;
    Ok(StatusCode::NO_CONTENT.into_response())
}

#[cfg(test)]
mod tests {
    use super::{ForgotBody, ResetTokenInput};
    use secrecy::SecretString;

    #[test]
    fn request_debug_redacts_email_and_reset_token() {
        let forgot = ForgotBody {
            email: "test-secret-person@example.test".to_owned(),
        };
        let token = ResetTokenInput(SecretString::from("test-secret-reset-token"));
        assert!(!format!("{forgot:?}").contains("test-secret-person@example.test"));
        assert!(!format!("{token:?}").contains("test-secret-reset-token"));
    }
}
