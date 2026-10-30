//! Tenant-scoped audit read, export and verification routes (AUD-07, SBD-40/46).

use crate::repo::{self, AuditReadFilters, AuditReadRow};
use crate::{AuditEvent, Category, Sealer, record};
use async_trait::async_trait;
use axum::Json;
use axum::body::{Body, Bytes};
use axum::extract::{Extension, Path, Query, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::Response;
use chrono::{DateTime, Utc};
use eq_core::actor::ActorCtx;
use eq_core::app_state::AppState;
use eq_core::auth::Actor;
use eq_core::db::{Tx, begin, begin_read_only};
use eq_core::error::{AppError, FieldError, MUST_BE_GTE, MUST_BE_LTE, codes};
use eq_core::ids::TenantId;
use eq_core::model::Role;
use eq_core::tenant::RequireTenant;
use futures_util::TryStreamExt;
use serde::Deserialize;
use serde_json::{Map, Value, json};
use std::io;
use utoipa_axum::router::OpenApiRouter;
use uuid::Uuid;

const DEFAULT_PAGE_SIZE: u16 = 50; // technical: default page for audit log reads
const MAX_PAGE_SIZE: u16 = 200; // technical: cap a single audit read response
const EXPORT_PAGE_SIZE: i64 = 200; // technical: bound each streamed CSV chunk
const MAX_FILTER_CHARS: usize = 128; // technical: bound audit filter parsing work

/// AUD-07: supplies the effective audit visibility policy from Core's policy store.
#[async_trait]
pub trait AuditPolicyProvider: Send + Sync {
    async fn admin_categories(&self, tx: &mut Tx, tenant: TenantId) -> Result<Vec<Category>, AppError>;
}

pub(crate) fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(utoipa_axum::routes!(list_entries))
        .routes(utoipa_axum::routes!(get_entry))
        .routes(utoipa_axum::routes!(export_entries))
        .routes(utoipa_axum::routes!(verify_entries))
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct AuditQuery {
    from: Option<DateTime<Utc>>,
    to: Option<DateTime<Utc>>,
    actor_id: Option<Uuid>,
    action: Option<String>,
    entity_type: Option<String>,
    entity_id: Option<Uuid>,
    category: Option<String>,
    limit: Option<u16>,
    cursor: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct VerifyQuery {
    from_seq: i64,
    to_seq: i64,
}

#[utoipa::path(
    get,
    path = "/api/v1/audit/entries",
    params(
        ("from" = Option<DateTime<Utc>>, Query),
        ("to" = Option<DateTime<Utc>>, Query),
        ("actor_id" = Option<Uuid>, Query),
        ("action" = Option<String>, Query),
        ("entity_type" = Option<String>, Query),
        ("entity_id" = Option<Uuid>, Query),
        ("category" = Option<String>, Query),
        ("limit" = Option<u16>, Query),
        ("cursor" = Option<String>, Query)
    ),
    responses((status = 200, description = "Tenant-scoped audit entries", body = Value)),
    security(("bearer" = [])),
    tag = "audit"
)]
async fn list_entries(
    State(state): State<AppState>,
    RequireTenant(tenant): RequireTenant,
    actor: Actor,
    Extension(policy): Extension<std::sync::Arc<dyn AuditPolicyProvider>>,
    Query(query): Query<AuditQuery>,
) -> Result<Json<Value>, AppError> {
    actor.require("audit.read")?;
    ensure_tenant(&actor, tenant.id)?;
    let (filters, limit) = parse_query(query)?;
    let mut tx = begin_read_only(&state.pool, tenant.id, actor.context()).await?;
    let categories = allowed_categories(&actor, policy.as_ref(), &mut tx, tenant.id).await?;
    let mut rows = repo::list_entries(&mut tx, tenant.id, &filters, categories, i64::from(limit) + 1).await?;
    tx.commit().await?;

    let has_more = rows.len() > usize::from(limit);
    rows.truncate(usize::from(limit));
    let next_cursor = if has_more {
        rows.last().map(|row| row.seq.to_string())
    } else {
        None
    };
    let entries = rows
        .iter()
        .map(|row| entry_value(row, &actor))
        .collect::<Vec<_>>();
    Ok(Json(json!({"entries": entries, "next_cursor": next_cursor})))
}

#[utoipa::path(
    get,
    path = "/api/v1/audit/entries/{id}",
    params(("id" = Uuid, Path, description = "Audit entry id")),
    responses((status = 200, description = "Audit entry", body = Value)),
    security(("bearer" = [])),
    tag = "audit"
)]
async fn get_entry(
    State(state): State<AppState>,
    RequireTenant(tenant): RequireTenant,
    actor: Actor,
    Extension(policy): Extension<std::sync::Arc<dyn AuditPolicyProvider>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    actor.require("audit.read")?;
    ensure_tenant(&actor, tenant.id)?;
    let mut tx = begin_read_only(&state.pool, tenant.id, actor.context()).await?;
    let categories = allowed_categories(&actor, policy.as_ref(), &mut tx, tenant.id).await?;
    let row = repo::find_entry(&mut tx, tenant.id, id, categories).await?;
    tx.commit().await?;
    let row = row.ok_or(AppError::NotFound {
        entity: "audit entry",
    })?;
    Ok(Json(entry_value(&row, &actor)))
}

#[utoipa::path(
    get,
    path = "/api/v1/audit/export",
    params(
        ("from" = Option<DateTime<Utc>>, Query),
        ("to" = Option<DateTime<Utc>>, Query),
        ("actor_id" = Option<Uuid>, Query),
        ("action" = Option<String>, Query),
        ("entity_type" = Option<String>, Query),
        ("entity_id" = Option<Uuid>, Query),
        ("category" = Option<String>, Query),
        ("limit" = Option<u16>, Query),
        ("cursor" = Option<String>, Query)
    ),
    responses((status = 200, description = "Streamed tenant audit CSV")),
    security(("bearer" = [])),
    tag = "audit"
)]
async fn export_entries(
    State(state): State<AppState>,
    RequireTenant(tenant): RequireTenant,
    actor: Actor,
    Extension(policy): Extension<std::sync::Arc<dyn AuditPolicyProvider>>,
    Query(query): Query<AuditQuery>,
) -> Result<Response, AppError> {
    actor.require("audit.read")?;
    ensure_tenant(&actor, tenant.id)?;
    let (mut filters, _) = parse_query(query)?;
    filters.before_seq = None;

    let mut tx = begin(&state.pool, tenant.id, actor.context()).await?;
    let categories = allowed_categories(&actor, policy.as_ref(), &mut tx, tenant.id).await?;
    if let Err(error) = record(
        &mut tx,
        AuditEvent {
            action: "audit.export",
            category: Category::Compliance,
            entity_type: "audit",
            entity_id: None,
            before: None,
            after: Some(json!({"format": "csv"})),
            actor: actor.context(),
        },
    )
    .await
    {
        tx.rollback().await?;
        return Err(error);
    }
    tx.commit().await?;

    let stream_state = CsvStreamState {
        app_state: state,
        tenant: tenant.id,
        actor: actor.context().clone(),
        filters,
        categories,
        owner: actor.role() == Role::Owner,
        read_pii: actor.has("identity.people.read_pii"),
        cursor: None,
        include_header: true,
        done: false,
    };
    let stream = futures_util::stream::try_unfold(stream_state, next_csv_chunk)
        .map_err(|error: AppError| io::Error::other(error.to_string()));
    Response::builder()
        .status(StatusCode::OK)
        .header(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/csv; charset=utf-8"),
        )
        .header(
            header::CONTENT_DISPOSITION,
            HeaderValue::from_static("attachment; filename=\"ritma-audit.csv\""),
        )
        .body(Body::from_stream(stream))
        .map_err(|error| AppError::Internal(error.into()))
}

#[utoipa::path(
    get,
    path = "/api/v1/audit/verify",
    params(
        ("from_seq" = i64, Query),
        ("to_seq" = i64, Query)
    ),
    responses((status = 200, description = "Audit chain verification report", body = Value)),
    security(("bearer" = [])),
    tag = "audit"
)]
async fn verify_entries(
    State(state): State<AppState>,
    RequireTenant(tenant): RequireTenant,
    actor: Actor,
    Query(query): Query<VerifyQuery>,
) -> Result<Json<Value>, AppError> {
    actor.require("audit.verify")?;
    ensure_tenant(&actor, tenant.id)?;
    if query.from_seq < 1 || query.to_seq < query.from_seq {
        return Err(AppError::Validation {
            errors: vec![field_error(
                "/from_seq",
                if query.from_seq < 1 {
                    MUST_BE_GTE
                } else {
                    MUST_BE_LTE
                },
            )],
        });
    }
    let report = Sealer::new(state.pool)
        .verify(tenant.id, query.from_seq, query.to_seq)
        .await?;
    Ok(Json(json!({
        "checked": report.checked,
        "anonymized": report.anonymized,
        "first_mismatch": report.first_mismatch,
    })))
}

fn parse_query(query: AuditQuery) -> Result<(AuditReadFilters, u16), AppError> {
    if query.from.zip(query.to).is_some_and(|(from, to)| from > to) {
        return Err(AppError::Validation {
            errors: vec![field_error("/from", MUST_BE_LTE)],
        });
    }
    for (pointer, value) in [
        ("/action", query.action.as_deref()),
        ("/entity_type", query.entity_type.as_deref()),
    ] {
        if value.is_some_and(|value| value.chars().count() > MAX_FILTER_CHARS) {
            return Err(AppError::Validation {
                errors: vec![field_error(pointer, MUST_BE_LTE)],
            });
        }
    }
    let limit = query.limit.unwrap_or(DEFAULT_PAGE_SIZE);
    if limit == 0 || limit > MAX_PAGE_SIZE {
        return Err(AppError::Validation {
            errors: vec![field_error(
                "/limit",
                if limit == 0 { MUST_BE_GTE } else { MUST_BE_LTE },
            )],
        });
    }
    let before_seq = query
        .cursor
        .as_deref()
        .map(str::parse::<i64>)
        .transpose()
        .map_err(|_| bad_query("cursor must be a positive audit sequence"))?;
    if before_seq.is_some_and(|seq| seq < 1) {
        return Err(bad_query("cursor must be a positive audit sequence"));
    }
    let category = query
        .category
        .as_deref()
        .map(parse_category)
        .transpose()?
        .map(|category| category.as_str().to_owned());
    Ok((
        AuditReadFilters {
            from: query.from,
            to: query.to,
            actor_id: query.actor_id,
            action_prefix: query.action,
            entity_type: query.entity_type,
            entity_id: query.entity_id,
            category,
            before_seq,
        },
        limit,
    ))
}

fn parse_category(value: &str) -> Result<Category, AppError> {
    match value {
        "operational" => Ok(Category::Operational),
        "security" => Ok(Category::Security),
        "compliance" => Ok(Category::Compliance),
        _ => Err(bad_query("category is not a registered audit category")),
    }
}

fn field_error(pointer: &str, code: &'static str) -> FieldError {
    FieldError {
        pointer: pointer.to_owned(),
        code: code.to_owned(),
        params: Map::new(),
    }
}

fn bad_query(detail: &str) -> AppError {
    AppError::BadRequest {
        code: &codes::BAD_REQUEST,
        detail: detail.to_owned(),
    }
}

fn ensure_tenant(actor: &Actor, tenant: TenantId) -> Result<(), AppError> {
    if actor.tenant() == tenant {
        Ok(())
    } else {
        Err(AppError::NotFound {
            entity: "audit entry",
        })
    }
}

async fn allowed_categories(
    actor: &Actor,
    provider: &dyn AuditPolicyProvider,
    tx: &mut Tx,
    tenant: TenantId,
) -> Result<Option<Vec<String>>, AppError> {
    if actor.role() == Role::Owner {
        return Ok(None);
    }
    Ok(Some(
        provider
            .admin_categories(tx, tenant)
            .await?
            .iter()
            .map(|category| category.as_str().to_owned())
            .collect(),
    ))
}

fn entry_value(row: &AuditReadRow, actor: &Actor) -> Value {
    let mut entry = json!({
        "id": row.id,
        "seq": row.seq,
        "at": row.at,
        "actor": {
            "id": row.actor_id,
            "display_name": Value::Null,
            "role": row.actor_role,
        },
        "action": row.action,
        "entity_type": row.entity_type,
        "entity_id": row.entity_id,
        "category": row.category,
        "before": if actor.has("identity.people.read_pii") { row.before_json.clone() } else { None },
        "after": if actor.has("identity.people.read_pii") { row.after_json.clone() } else { None },
        "request_id": row.request_id,
        "sealed": row.sealed,
    });
    if actor.role() == Role::Owner {
        entry["ip"] = json!(row.ip);
    }
    entry
}

struct CsvStreamState {
    app_state: AppState,
    tenant: TenantId,
    actor: ActorCtx,
    filters: AuditReadFilters,
    categories: Option<Vec<String>>,
    owner: bool,
    read_pii: bool,
    cursor: Option<i64>,
    include_header: bool,
    done: bool,
}

async fn next_csv_chunk(mut state: CsvStreamState) -> Result<Option<(Bytes, CsvStreamState)>, AppError> {
    if state.done {
        return Ok(None);
    }
    let mut filters = state.filters.clone();
    filters.before_seq = state.cursor;
    let mut tx = begin_read_only(&state.app_state.pool, state.tenant, &state.actor).await?;
    let rows = repo::list_entries(
        &mut tx,
        state.tenant,
        &filters,
        state.categories.clone(),
        EXPORT_PAGE_SIZE,
    )
    .await?;
    tx.commit().await?;
    if rows.is_empty() {
        return Ok(None);
    }

    let has_next = i64::try_from(rows.len()).ok() == Some(EXPORT_PAGE_SIZE);
    state.cursor = rows.last().map(|row| row.seq);
    state.done = !has_next;
    let mut writer = csv::Writer::from_writer(Vec::new());
    if state.include_header {
        write_csv_header(&mut writer)?;
        state.include_header = false;
    }
    for row in rows {
        writer
            .write_record(csv_row_record(row, state.owner, state.read_pii)?)
            .map_err(|error| AppError::Internal(error.into()))?;
    }
    let bytes = writer
        .into_inner()
        .map_err(|error| AppError::Internal(error.into_error().into()))?;
    Ok(Some((Bytes::from(bytes), state)))
}

fn write_csv_header(writer: &mut csv::Writer<Vec<u8>>) -> Result<(), AppError> {
    writer
        .write_record([
            "id",
            "seq",
            "at",
            "actor_id",
            "actor_role",
            "action",
            "entity_type",
            "entity_id",
            "category",
            "before",
            "after",
            "request_id",
            "sealed",
            "ip",
        ])
        .map_err(|error| AppError::Internal(error.into()))
}

fn csv_row_record(row: AuditReadRow, owner: bool, read_pii: bool) -> Result<[String; 14], AppError> {
    let before = if read_pii {
        row.before_json
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|error| AppError::Internal(error.into()))?
            .unwrap_or_default()
    } else {
        String::new()
    };
    let after = if read_pii {
        row.after_json
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|error| AppError::Internal(error.into()))?
            .unwrap_or_default()
    } else {
        String::new()
    };
    let ip = if owner && read_pii {
        row.ip.unwrap_or_default()
    } else {
        String::new()
    };
    Ok([
        row.id.to_string(),
        row.seq.to_string(),
        row.at.to_rfc3339(),
        row.actor_id.map(|id| id.to_string()).unwrap_or_default(),
        row.actor_role,
        row.action,
        row.entity_type,
        row.entity_id.map(|id| id.to_string()).unwrap_or_default(),
        row.category,
        before,
        after,
        row.request_id.map(|id| id.to_string()).unwrap_or_default(),
        row.sealed.to_string(),
        ip,
    ]
    .map(|value| eq_core::export::guard_cell(&value)))
}
