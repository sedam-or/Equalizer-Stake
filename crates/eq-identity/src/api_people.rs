//! IDN-10/11: people, role and guardian-link HTTP boundaries.

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use eq_core::app_state::AppState;
use eq_core::auth::Actor;
use eq_core::db::{Tx, begin, begin_read_only};
use eq_core::error::{AppError, FieldError};
use eq_core::fields::{FieldCtx, FieldIssue, FieldKind, ShortText};
use eq_core::http::validated::{ValidatedJson, ValidatedQuery, parse_path_id};
use eq_core::ids::PersonId;
use eq_core::model::{MembershipStatus, Role};
use eq_core::tenant::{RequireTenant, TenantInfo};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::people;

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
#[expect(
    dead_code,
    reason = "the enum supplies the canonical role schema for people responses"
)]
enum RoleSchema {
    Owner,
    Admin,
    Instructor,
    Guardian,
    Student,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
#[expect(
    dead_code,
    reason = "the enum supplies the canonical membership status schema"
)]
enum MembershipStatusSchema {
    Invited,
    Active,
    Deactivated,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PeopleQuery {
    role: Option<Role>,
}

#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct RoleBody {
    #[schema(value_type = RoleSchema)]
    role: Role,
}

#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct GuardianLinkBody {
    #[schema(value_type = String, format = Uuid)]
    student_id: Uuid,
    relationship: Option<String>,
    is_primary: bool,
}

#[derive(Serialize, ToSchema)]
pub(crate) struct PersonSummaryResponse {
    #[schema(value_type = String, format = Uuid)]
    id: PersonId,
    display_name: String,
    #[schema(value_type = Vec<RoleSchema>)]
    roles: Vec<Role>,
    #[schema(value_type = MembershipStatusSchema)]
    status: MembershipStatus,
    avatar_url: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub(crate) struct PeoplePage {
    items: Vec<PersonSummaryResponse>,
    next_cursor: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub(crate) struct PersonLinkResponse {
    #[schema(value_type = String, format = Uuid)]
    id: PersonId,
    display_name: String,
    relationship: Option<String>,
    is_primary: bool,
}

#[derive(Serialize, ToSchema)]
pub(crate) struct PersonResponse {
    #[serde(flatten)]
    summary: PersonSummaryResponse,
    legal_name: Option<String>,
    email: Option<String>,
    phone: Option<String>,
    birth_date: Option<String>,
    locale: Option<String>,
    guardians: Vec<PersonLinkResponse>,
    students: Vec<PersonLinkResponse>,
    version: i32,
}

/// IDN-10/AUTH-04: lists only the rows Core's tenant-scoped SQL exposes to this Actor.
#[utoipa::path(
    get,
    path = "/api/v1/people",
    params(("role" = Option<String>, Query, description = "Filter by active tenant role")),
    responses((status = 200, description = "Scoped people page", body = PeoplePage)),
    security(("bearer" = [])),
    tag = "identity"
)]
pub(crate) async fn list_people(
    State(state): State<AppState>,
    RequireTenant(tenant): RequireTenant,
    actor: Actor,
    ValidatedQuery(query): ValidatedQuery<PeopleQuery>,
) -> Result<Json<PeoplePage>, AppError> {
    actor.require("identity.people.read")?;
    let mut tx = begin_read_only(&state.pool, tenant.id, actor.context()).await?;
    match people::list_people(state.auth.directory(), &mut tx, &actor, query.role).await {
        Ok(people) => {
            tx.commit().await?;
            Ok(Json(PeoplePage {
                items: people.into_iter().map(summary_response).collect(),
                next_cursor: None,
            }))
        }
        Err(error) => {
            tx.rollback().await?;
            Err(error)
        }
    }
}

/// IDN-10/AUTH-04: returns 404 when the Core-scoped query cannot see the requested person.
#[utoipa::path(
    get,
    path = "/api/v1/people/{id}",
    params(("id" = String, Path, description = "Person id")),
    responses(
        (status = 200, description = "Scoped person detail", body = PersonResponse),
        (status = 404, description = "Person is not visible in this scope")
    ),
    security(("bearer" = [])),
    tag = "identity"
)]
pub(crate) async fn get_person(
    State(state): State<AppState>,
    RequireTenant(tenant): RequireTenant,
    actor: Actor,
    Path(raw_id): Path<String>,
) -> Result<Json<PersonResponse>, AppError> {
    actor.require("identity.people.read")?;
    let person = PersonId(parse_path_id(&raw_id)?);
    let mut tx = begin_read_only(&state.pool, tenant.id, actor.context()).await?;
    match people::person_in_scope(state.auth.directory(), &mut tx, &actor, person).await {
        Ok(Some(person)) => {
            tx.commit().await?;
            Ok(Json(person_response(person)))
        }
        Ok(None) => {
            tx.rollback().await?;
            Err(AppError::NotFound { entity: "person" })
        }
        Err(error) => {
            tx.rollback().await?;
            Err(error)
        }
    }
}

/// IDN-11/STM-20: grants a role only after permission and owner-role checks.
#[utoipa::path(
    post,
    path = "/api/v1/people/{id}/roles",
    params(("id" = String, Path, description = "Person id")),
    request_body = RoleBody,
    responses(
        (status = 204, description = "Role granted"),
        (status = 403, description = "Role grant is not permitted"),
        (status = 404, description = "Person not found")
    ),
    security(("bearer" = [])),
    tag = "identity"
)]
pub(crate) async fn grant_role(
    State(state): State<AppState>,
    RequireTenant(tenant): RequireTenant,
    actor: Actor,
    Path(raw_id): Path<String>,
    ValidatedJson(body): ValidatedJson<RoleBody>,
) -> Result<Response, AppError> {
    actor.require("identity.roles.write")?;
    if matches!(body.role, Role::Owner | Role::Admin) {
        actor.require("identity.admins.manage")?;
    }
    let person = PersonId(parse_path_id(&raw_id)?);
    let mut tx = begin(&state.pool, tenant.id, actor.context()).await?;
    let result = people::grant_role(state.auth.directory(), &mut tx, &actor, person, body.role).await;
    finish_mutation(tx, result).await
}

/// STM-20: deactivates one role and protects the tenant's final active owner.
#[utoipa::path(
    delete,
    path = "/api/v1/people/{id}/roles/{role}",
    params(
        ("id" = String, Path, description = "Person id"),
        ("role" = String, Path, description = "Role to remove")
    ),
    responses(
        (status = 204, description = "Role removed"),
        (status = 403, description = "Role removal is not permitted"),
        (status = 404, description = "Membership not found"),
        (status = 409, description = "The last active owner cannot be removed")
    ),
    security(("bearer" = [])),
    tag = "identity"
)]
pub(crate) async fn remove_role(
    State(state): State<AppState>,
    RequireTenant(tenant): RequireTenant,
    actor: Actor,
    Path((raw_id, raw_role)): Path<(String, String)>,
) -> Result<Response, AppError> {
    actor.require("identity.roles.write")?;
    let person = PersonId(parse_path_id(&raw_id)?);
    let role = parse_role(&raw_role)?;
    if matches!(role, Role::Owner | Role::Admin) {
        actor.require("identity.admins.manage")?;
    }
    let mut tx = begin(&state.pool, tenant.id, actor.context()).await?;
    let result = people::remove_role(
        state.auth.directory(),
        &mut tx,
        &actor,
        person,
        role,
        state.clock.now(),
    )
    .await;
    finish_mutation(tx, result).await
}

/// STM-27: creates a scoped guardian/student link and maps the unique primary constraint.
#[utoipa::path(
    post,
    path = "/api/v1/people/{id}/links",
    params(("id" = String, Path, description = "Guardian person id")),
    request_body = GuardianLinkBody,
    responses(
        (status = 204, description = "Guardian link created"),
        (status = 409, description = "Student already has a primary guardian"),
        (status = 422, description = "Guardian or student role is invalid")
    ),
    security(("bearer" = [])),
    tag = "identity"
)]
pub(crate) async fn create_guardian_link(
    State(state): State<AppState>,
    RequireTenant(tenant): RequireTenant,
    actor: Actor,
    Path(raw_id): Path<String>,
    ValidatedJson(body): ValidatedJson<GuardianLinkBody>,
) -> Result<Response, AppError> {
    actor.require("identity.people.write")?;
    let guardian = PersonId(parse_path_id(&raw_id)?);
    let student = PersonId(body.student_id);
    let context = field_context(&tenant, state.clock.now());
    let relationship = body
        .relationship
        .map(|raw| {
            ShortText::parse(raw, &context)
                .map(|value| value.as_ref().to_owned())
                .map_err(|issues| validation_error("/relationship", issues))
        })
        .transpose()?;
    let mut tx = begin(&state.pool, tenant.id, actor.context()).await?;
    let result = people::create_guardian_link(
        state.auth.directory(),
        &mut tx,
        &actor,
        guardian,
        student,
        relationship,
        body.is_primary,
    )
    .await;
    finish_mutation(tx, result).await
}

fn summary_response(person: eq_core::auth::directory::DirectoryPersonSummary) -> PersonSummaryResponse {
    PersonSummaryResponse {
        id: person.person_id,
        display_name: person.display_name,
        roles: person.roles,
        status: MembershipStatus::Active,
        avatar_url: None,
    }
}

fn person_response(person: eq_core::auth::directory::DirectoryPersonSummary) -> PersonResponse {
    let version = person.version;
    PersonResponse {
        summary: summary_response(person),
        legal_name: None,
        email: None,
        phone: None,
        birth_date: None,
        locale: None,
        guardians: Vec::new(),
        students: Vec::new(),
        version,
    }
}

async fn finish_mutation(tx: Tx, result: Result<(), AppError>) -> Result<Response, AppError> {
    match result {
        Ok(()) => {
            tx.commit().await?;
            Ok(StatusCode::NO_CONTENT.into_response())
        }
        Err(error) => {
            tx.rollback().await?;
            Err(error)
        }
    }
}

fn parse_role(raw: &str) -> Result<Role, AppError> {
    serde_json::from_value(Value::String(raw.to_owned())).map_err(|_| AppError::Validation {
        errors: vec![FieldError {
            pointer: String::from("/role"),
            code: eq_core::errors::NOT_ALLOWED.to_owned(),
            params: Map::new(),
        }],
    })
}

fn field_context(tenant: &TenantInfo, now: chrono::DateTime<chrono::Utc>) -> FieldCtx {
    FieldCtx {
        country: None,
        tenant_currency: tenant.currency,
        tenant_locale: tenant.locale,
        today: now.date_naive(),
        password_min_length: None,
    }
}

fn validation_error(pointer: &str, issues: Vec<FieldIssue>) -> AppError {
    AppError::Validation {
        errors: issues
            .into_iter()
            .map(|issue| FieldError {
                pointer: issue
                    .pointer
                    .map(|child| format!("{pointer}{child}"))
                    .unwrap_or_else(|| pointer.to_owned()),
                code: issue.code.to_owned(),
                params: issue.params,
            })
            .collect(),
    }
}
