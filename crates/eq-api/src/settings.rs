//! POL-06/10/12: Core policy settings HTTP routes.

use axum::Json;
use axum::extract::{Extension, State};
use axum::http::{HeaderMap, StatusCode};
use chrono::{DateTime, Duration, Utc};
use eq_core::app_state::AppState;
use eq_core::auth::Actor;
use eq_core::db::{begin, begin_read_only};
use eq_core::error::{AppError, FieldError, MUST_BE_GTE, codes};
use eq_core::policy::{ConditionalPolicyWrite, CorePolicy, PolicyDoc, validate_document};
use eq_core::ports::CurrencyLockCheck;
use eq_core::tenant::RequireTenant;
use eq_identity::api_auth::LoginHttpState;
use serde::Deserialize;
use serde_json::{Map, Value, json};
use utoipa_axum::router::OpenApiRouter;

type SettingsExtensions = (
    std::sync::Arc<LoginHttpState>,
    std::sync::Arc<dyn CurrencyLockCheck>,
);

/// POL-06: mounts the assigned Core policy HTTP operations.
pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(utoipa_axum::routes!(list_policies))
        .routes(utoipa_axum::routes!(preview_core_policy))
        .routes(utoipa_axum::routes!(put_core_policy))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PreviewBody {
    document: Value,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PutBody {
    document: Value,
    effective_from: Option<DateTime<Utc>>,
    note: Option<String>,
}

/// POL-06: lists the current Core policy and any pending future version.
#[utoipa::path(
    get,
    path = "/api/v1/settings/policies",
    responses((status = 200, description = "Core policy and pending future version", body = Value)),
    security(("bearer" = [])),
    tag = "core"
)]
pub(crate) async fn list_policies(
    State(state): State<AppState>,
    RequireTenant(tenant): RequireTenant,
    actor: Actor,
    Extension((runtime, _currency_lock)): Extension<SettingsExtensions>,
) -> Result<Json<Value>, AppError> {
    actor.require("core.policy.read")?;
    if actor.tenant() != tenant.id {
        return Err(AppError::NotFound { entity: "tenant" });
    }
    let mut tx = begin_read_only(&state.pool, tenant.id, actor.context()).await?;
    let store = runtime.policy_store().await?;
    let history = store.history::<CorePolicy>(&mut tx, tenant.id).await?;
    let now = state.clock.now();
    let current = history
        .iter()
        .filter(|version| version.effective_from <= now)
        .max_by(|left, right| {
            left.effective_from
                .cmp(&right.effective_from)
                .then_with(|| left.version.cmp(&right.version))
        })
        .cloned();
    let current = match current {
        Some(current) => current,
        None => store.current::<CorePolicy>(&mut tx, tenant.id).await?,
    };
    let pending = history
        .iter()
        .filter(|version| version.effective_from > now)
        .min_by(|left, right| {
            left.effective_from
                .cmp(&right.effective_from)
                .then_with(|| left.version.cmp(&right.version))
        });
    let document = serde_json::to_value(&current.doc).map_err(|error| AppError::Internal(error.into()))?;
    let schema = eq_core::module::Module::policy_schema(&eq_core::module::CoreModule);
    let summary = current.doc.summarize();
    tx.commit().await?;
    let entry = json!({
        "module": "core",
        "version": current.version,
        "effective_from": current.effective_from,
        "schema": schema,
        "document": document,
        "summary": summary,
        "pending_future": pending.map(|future| json!({
            "version": future.version,
            "effective_from": future.effective_from,
        })),
    });
    Ok(Json(json!([entry])))
}

/// POL-10: validates and summarizes a Core policy without writing state.
#[utoipa::path(
    post,
    path = "/api/v1/settings/policies/{module}/preview",
    params(("module" = String, Path, description = "Policy module identifier")),
    request_body = Value,
    responses((status = 200, description = "Validation result", body = Value)),
    security(("bearer" = [])),
    tag = "core"
)]
async fn preview_core_policy(
    State(_state): State<AppState>,
    RequireTenant(tenant): RequireTenant,
    axum::extract::Path(module): axum::extract::Path<String>,
    actor: Actor,
    Json(body): Json<PreviewBody>,
) -> Result<Json<Value>, AppError> {
    actor.require("core.policy.write")?;
    if actor.tenant() != tenant.id {
        return Err(AppError::NotFound { entity: "tenant" });
    }
    if module != "core" {
        return Err(AppError::NotFound { entity: "policy" });
    }
    match validate_document::<CorePolicy>(body.document.clone()) {
        Ok(policy) => Ok(Json(json!({
            "valid": true,
            "errors": [],
            "summary": policy.summarize(),
        }))),
        Err(AppError::Validation { errors }) => Ok(Json(json!({
            "valid": false,
            "errors": errors,
            "summary": [],
        }))),
        Err(error) => Err(error),
    }
}

/// POL-03/12/STM-13: appends the next Core policy version after permission, ETag and lock checks.
#[utoipa::path(
    put,
    path = "/api/v1/settings/policies/{module}",
    params(("module" = String, Path, description = "Policy module identifier")),
    request_body = Value,
    responses((status = 200, description = "Saved Core policy version", body = Value)),
    security(("bearer" = [])),
    tag = "core"
)]
async fn put_core_policy(
    State(state): State<AppState>,
    RequireTenant(tenant): RequireTenant,
    Extension((runtime, currency_lock)): Extension<SettingsExtensions>,
    axum::extract::Path(module): axum::extract::Path<String>,
    actor: Actor,
    headers: HeaderMap,
    Json(body): Json<PutBody>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    actor.require("core.policy.write")?;
    if actor.tenant() != tenant.id {
        return Err(AppError::NotFound { entity: "tenant" });
    }
    if module != "core" {
        return Err(AppError::NotFound { entity: "policy" });
    }
    let expected_version = if let Some(value) = headers.get(axum::http::header::IF_MATCH) {
        parse_if_match(value.to_str().map_err(|_| bad_etag())?)?
    } else {
        return Err(AppError::PreconditionRequired);
    };
    let policy = validate_document::<CorePolicy>(body.document)?;
    let now = state.clock.now();
    let effective_from = body.effective_from.unwrap_or(now);
    if effective_from < now - Duration::minutes(5) {
        return Err(AppError::Validation {
            errors: vec![FieldError {
                pointer: "/effective_from".to_owned(),
                code: MUST_BE_GTE.to_owned(),
                params: Map::new(),
            }],
        });
    }

    let mut tx = begin(&state.pool, tenant.id, actor.context()).await?;
    let result = async {
        let store = runtime.policy_store().await?;
        let current = store.current::<CorePolicy>(&mut tx, tenant.id).await?;
        if current.doc.currency != policy.currency && currency_lock.is_locked(&mut tx, tenant.id).await? {
            return Err(AppError::PolicyViolation {
                rule: "core.currency.locked".to_owned(),
                detail: "tenant currency is locked by an existing invoice".to_owned(),
                extra: Map::new(),
            });
        }
        let saved = store
            .put_if_current(
                &mut tx,
                tenant.id,
                policy,
                effective_from,
                actor.context(),
                ConditionalPolicyWrite {
                    note: body.note,
                    expected_version,
                },
            )
            .await?;
        store
            .update_core_tenant_defaults(&mut tx, tenant.id, &saved.doc)
            .await?;
        Ok::<_, AppError>(saved)
    }
    .await;
    let saved = match result {
        Ok(saved) => saved,
        Err(error) => {
            tx.rollback().await?;
            return Err(error);
        }
    };
    tx.commit().await?;
    let document = serde_json::to_value(saved.doc).map_err(|error| AppError::Internal(error.into()))?;
    Ok((
        StatusCode::OK,
        Json(json!({
            "module": "core",
            "version": saved.version,
            "effective_from": saved.effective_from,
            "document": document,
        })),
    ))
}

fn parse_if_match(value: &str) -> Result<i32, AppError> {
    let Some(version) = value
        .strip_prefix("\"v")
        .and_then(|value| value.strip_suffix('"'))
    else {
        return Err(bad_etag());
    };
    version.parse().map_err(|_| bad_etag())
}

fn bad_etag() -> AppError {
    AppError::BadRequest {
        code: &codes::BAD_REQUEST,
        detail: "If-Match must be a quoted policy version".to_owned(),
    }
}
