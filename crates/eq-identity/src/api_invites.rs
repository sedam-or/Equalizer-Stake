//! IDN-07: public preview and acceptance HTTP boundaries for identity invites.

use std::fmt;
use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, Path, State};
use axum::http::header::USER_AGENT;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono_tz::Tz;
use eq_core::app_state::AppState;
use eq_core::db::begin;
use eq_core::error::AppError;
use eq_core::fields::{FieldCtx, FieldKind, Password, PasswordInput};
use eq_core::tenant::{RequestActor, RequireTenant};
use secrecy::SecretString;
use serde::Deserialize;
use serde_json::Map;
use utoipa::ToSchema;

use crate::api_auth::{LoginHttpState, LoginResponseSchema, login_success_response};
use crate::errors::codes::PASSWORD_WEAK;
use crate::invites::{InviteAcceptanceRequest, InvitePreview, InviteService};
use crate::policy::IdentityPolicy;
use crate::service::LoginService;

#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct AcceptInviteBody {
    password: PasswordInput,
}

impl fmt::Debug for AcceptInviteBody {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AcceptInviteBody")
            .field("password", &"[REDACTED]")
            .finish()
    }
}

/// IDN-07/STM-21: previews only pending invitations within the resolved tenant.
#[utoipa::path(
    get,
    path = "/api/v1/auth/invites/{token}",
    params(("token" = String, Path, description = "Opaque single-use invite token")),
    responses(
        (status = 200, description = "Invite preview", body = InvitePreview),
        (status = 404, description = "Invite not found"),
        (status = 410, description = "Invite expired or already accepted")
    ),
    security(()),
    tag = "identity"
)]
pub(crate) async fn preview(
    State(state): State<AppState>,
    RequireTenant(tenant): RequireTenant,
    RequestActor(actor): RequestActor,
    Path(token): Path<String>,
) -> Result<Response, AppError> {
    let mut tx = begin(&state.pool, tenant.id, &actor).await?;
    let token = SecretString::from(token);
    let service = InviteService::new(state.auth.directory(), state.auth.keyring(), state.clock.as_ref());
    let preview = match service.preview_invite(&mut tx, tenant.id, &token).await {
        Ok(preview) => preview,
        Err(error) => {
            tx.rollback().await?;
            return Err(error);
        }
    };
    tx.commit().await?;
    Ok((StatusCode::OK, Json(preview)).into_response())
}

/// IDN-02/07/STM-21: consumes a pending invitation, sets the password and starts a session atomically.
#[utoipa::path(
    post,
    path = "/api/v1/auth/invites/{token}/accept",
    params(("token" = String, Path, description = "Opaque single-use invite token")),
    request_body = AcceptInviteBody,
    responses(
        (status = 200, description = "Invite accepted and login session created", body = LoginResponseSchema),
        (status = 404, description = "Invite not found"),
        (status = 410, description = "Invite expired or already accepted"),
        (status = 401, description = "TOTP verification is required for an enrolled identity"),
        (status = 422, description = "Password does not meet the current policy")
    ),
    security(()),
    tag = "identity"
)]
pub(crate) async fn accept(
    State(state): State<AppState>,
    Extension(runtime): Extension<Arc<LoginHttpState>>,
    RequireTenant(tenant): RequireTenant,
    RequestActor(actor): RequestActor,
    Path(token): Path<String>,
    headers: HeaderMap,
    Json(body): Json<AcceptInviteBody>,
) -> Result<Response, AppError> {
    let timezone = tenant
        .timezone
        .parse::<Tz>()
        .map_err(|_| AppError::Internal(anyhow::anyhow!("resolved tenant timezone is invalid")))?;
    let policies = runtime.policy_store().await?;
    let token = SecretString::from(token);
    let invite = InviteService::new(state.auth.directory(), state.auth.keyring(), state.clock.as_ref());
    let mut lookup_tx = begin(&state.pool, tenant.id, &actor).await?;
    let acceptance_actor = match invite
        .acceptance_actor(&mut lookup_tx, tenant.id, &token, &actor)
        .await
    {
        Ok(actor) => actor,
        Err(error) => {
            lookup_tx.rollback().await?;
            return Err(error);
        }
    };
    lookup_tx.rollback().await?;

    // STM-20 identifies the invitee as the state-transition actor. Resolve the bearer token,
    // then bind the atomic mutation transaction to that invitee and revalidate the invite.
    let mut tx = begin(&state.pool, tenant.id, &acceptance_actor).await?;
    let policy = policies.current::<IdentityPolicy>(&mut tx, tenant.id).await?.doc;
    let field_ctx = FieldCtx {
        country: None,
        tenant_currency: tenant.currency,
        tenant_locale: tenant.locale,
        today: state.clock.now().with_timezone(&timezone).date_naive(),
        password_min_length: Some(usize::from(policy.password_min_length())),
    };
    let password = Password::parse(body.password, &field_ctx).map_err(|_| password_weak())?;
    let user_agent = headers
        .get(USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    let login = LoginService::new(
        state.auth.directory(),
        policies,
        state.auth.keys(),
        state.auth.keyring(),
        state.clock.as_ref(),
    );
    let result = invite
        .accept_invite(
            &mut tx,
            InviteAcceptanceRequest::new(
                tenant.id,
                &acceptance_actor,
                token,
                &password,
                &policy,
                user_agent.as_deref(),
                actor.ip,
            ),
            &login,
        )
        .await;
    let success = match result {
        Ok(success) => success,
        Err(error) => {
            tx.rollback().await?;
            return Err(error);
        }
    };
    let response =
        match login_success_response(&state, &runtime, tenant.id, &mut tx, policies, &success).await {
            Ok(response) => response,
            Err(error) => {
                tx.rollback().await?;
                return Err(error);
            }
        };
    tx.commit().await?;
    Ok(response)
}

fn password_weak() -> AppError {
    AppError::Unprocessable {
        code: &PASSWORD_WEAK,
        detail: String::from("password does not meet the current identity policy"),
        extra: Map::new(),
    }
}
