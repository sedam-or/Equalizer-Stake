//! IDN-10/11: people, role and guardian-link HTTP boundaries.

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{DateTime, Utc};
use eq_core::app_state::AppState;
use eq_core::auth::Actor;
use eq_core::auth::directory::{
    DirectoryPersonSummary, NewDirectoryPerson, PeopleCursor, PeoplePageQuery, PeopleSort,
};
use eq_core::db::{Tx, begin, begin_read_only};
use eq_core::error::{AppError, FieldError};
use eq_core::fields::{
    BirthDate, Collector, DisplayName, Email, FieldCtx, FieldIssue, FieldKind, LegalName, Phone, Raw,
    ShortText,
};
use eq_core::http::validated::{ValidatedJson, ValidatedQuery, parse_path_id};
use eq_core::ids::PersonId;
use eq_core::model::{MembershipStatus, Role};
use eq_core::tenant::{RequireTenant, TenantInfo};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::people;

const DEFAULT_PAGE_SIZE: u16 = 50; // technical: API-05 page size when the caller omits limit.
const MAX_CURSOR_BYTES: usize = 1024; // technical: cap decoded cursor work to the compact {k,id} API-05 payload.

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
    limit: Option<u16>,
    cursor: Option<String>,
    sort: Option<String>,
}

#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct CreatePersonBody {
    display_name: String,
    legal_name: Option<String>,
    email: Option<String>,
    phone: Option<String>,
    birth_date: Option<String>,
    #[schema(value_type = Vec<RoleSchema>)]
    roles: Vec<Role>,
    invite: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CursorPayload {
    k: String,
    id: Uuid,
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
    let context = field_context(&tenant, state.clock.now())?;
    let sort = parse_sort(query.sort.as_deref())?;
    let cursor = query
        .cursor
        .as_deref()
        .map(|cursor| decode_cursor(cursor, sort, &context))
        .transpose()?;
    let page_query = PeoplePageQuery::new(query.role, query.limit.unwrap_or(DEFAULT_PAGE_SIZE), sort, cursor)
        .ok_or_else(|| invalid_field("/limit"))?;
    let mut tx = begin_read_only(&state.pool, tenant.id, actor.context()).await?;
    match people::list_people(state.auth.directory(), &mut tx, &actor, &page_query).await {
        Ok(page) => {
            let next_cursor = if page.has_more {
                page.people
                    .last()
                    .map(|person| encode_cursor(sort, person))
                    .transpose()?
            } else {
                None
            };
            tx.commit().await?;
            Ok(Json(PeoplePage {
                items: page.people.into_iter().map(summary_response).collect(),
                next_cursor,
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
    let include_pii = actor.has("identity.people.read_pii");
    let now = state.clock.now();
    let transaction = if include_pii {
        begin(&state.pool, tenant.id, actor.context()).await?
    } else {
        begin_read_only(&state.pool, tenant.id, actor.context()).await?
    };
    let mut tx = transaction;
    match people::person_detail_in_scope(
        state.auth.directory(),
        &mut tx,
        &actor,
        people::PersonDetailRequest {
            person,
            include_pii,
            tenant_timezone: &tenant.timezone,
            now,
        },
        state.auth.keyring(),
    )
    .await
    {
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

/// IDN-10/STM-20: creates a person and the requested tenant memberships.
#[utoipa::path(
    post,
    path = "/api/v1/people",
    request_body = CreatePersonBody,
    responses(
        (status = 201, description = "Person created", body = PersonResponse),
        (status = 403, description = "Person creation is not permitted"),
        (status = 409, description = "Email is already used in this tenant")
    ),
    security(("bearer" = [])),
    tag = "identity"
)]
pub(crate) async fn create_person(
    State(state): State<AppState>,
    RequireTenant(tenant): RequireTenant,
    actor: Actor,
    ValidatedJson(body): ValidatedJson<CreatePersonBody>,
) -> Result<Response, AppError> {
    actor.require("identity.people.write")?;
    if body
        .roles
        .iter()
        .any(|role| matches!(role, Role::Owner | Role::Admin))
    {
        actor.require("identity.admins.manage")?;
    }
    let input = parse_new_person(body, &tenant, state.clock.now())?;
    let mut tx = begin(&state.pool, tenant.id, actor.context()).await?;
    match people::create_person(
        state.auth.directory(),
        &mut tx,
        &actor,
        input,
        state.auth.keyring(),
    )
    .await
    {
        Ok(person) => {
            tx.commit().await?;
            Ok((StatusCode::CREATED, Json(person_response_from_summary(person))).into_response())
        }
        Err(error) => {
            tx.rollback().await?;
            Err(error)
        }
    }
}

/// IDN-13/STM-20: deactivates every membership and revokes the person's refresh sessions.
#[utoipa::path(
    post,
    path = "/api/v1/people/{id}/deactivate",
    params(("id" = String, Path, description = "Person id")),
    responses(
        (status = 204, description = "Person deactivated"),
        (status = 403, description = "Person deactivation is not permitted"),
        (status = 404, description = "Person not found"),
        (status = 409, description = "The last active owner cannot be deactivated")
    ),
    security(("bearer" = [])),
    tag = "identity"
)]
pub(crate) async fn deactivate_person(
    State(state): State<AppState>,
    RequireTenant(tenant): RequireTenant,
    actor: Actor,
    Path(raw_id): Path<String>,
) -> Result<Response, AppError> {
    actor.require("identity.people.write")?;
    let person = PersonId(parse_path_id(&raw_id)?);
    let mut tx = begin(&state.pool, tenant.id, actor.context()).await?;
    let result =
        people::deactivate_person(state.auth.directory(), &mut tx, &actor, person, state.clock.now()).await;
    finish_mutation(tx, result).await
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
    let context = field_context(&tenant, state.clock.now())?;
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

fn summary_response(person: DirectoryPersonSummary) -> PersonSummaryResponse {
    PersonSummaryResponse {
        id: person.person_id,
        display_name: person.display_name,
        roles: person.roles,
        status: MembershipStatus::Active,
        avatar_url: None,
    }
}

fn person_response_from_summary(person: DirectoryPersonSummary) -> PersonResponse {
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

fn person_response(person: eq_core::auth::directory::DirectoryPersonDetail) -> PersonResponse {
    let version = person.summary.version;
    PersonResponse {
        summary: summary_response(person.summary),
        legal_name: person.legal_name.map(|value| value.expose_secret().to_owned()),
        email: person.email.map(|value| value.expose_secret().to_owned()),
        phone: person.phone.map(|value| value.expose_secret().to_owned()),
        birth_date: person.birth_date.map(|value| value.expose_secret().to_owned()),
        locale: person.locale,
        guardians: person.guardians.into_iter().map(link_response).collect(),
        students: person.students.into_iter().map(link_response).collect(),
        version,
    }
}

fn link_response(link: eq_core::auth::directory::DirectoryPersonLink) -> PersonLinkResponse {
    PersonLinkResponse {
        id: link.person_id,
        display_name: link.display_name,
        relationship: link.relationship,
        is_primary: link.is_primary,
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

fn field_context(tenant: &TenantInfo, now: DateTime<Utc>) -> Result<FieldCtx, AppError> {
    let timezone = tenant
        .timezone
        .parse::<chrono_tz::Tz>()
        .map_err(|_| AppError::Internal(anyhow::anyhow!("tenant timezone is invalid")))?;
    Ok(FieldCtx {
        country: None,
        tenant_currency: tenant.currency,
        tenant_locale: tenant.locale,
        today: now.with_timezone(&timezone).date_naive(),
        password_min_length: None,
    })
}

fn parse_sort(raw: Option<&str>) -> Result<PeopleSort, AppError> {
    match raw.unwrap_or("display_name") {
        "display_name" => Ok(PeopleSort::DisplayName),
        "-created_at" => Ok(PeopleSort::CreatedAtDesc),
        _ => Err(invalid_field("/sort")),
    }
}

fn decode_cursor(raw: &str, sort: PeopleSort, context: &FieldCtx) -> Result<PeopleCursor, AppError> {
    if raw.len() > MAX_CURSOR_BYTES {
        return Err(invalid_field("/cursor"));
    }
    let bytes = URL_SAFE_NO_PAD
        .decode(raw)
        .map_err(|_| invalid_field("/cursor"))?;
    let payload: CursorPayload = serde_json::from_slice(&bytes).map_err(|_| invalid_field("/cursor"))?;
    match sort {
        PeopleSort::DisplayName => {
            let name = DisplayName::parse(payload.k, context)
                .map_err(|issues| validation_error("/cursor/k", issues))?;
            Ok(PeopleCursor::display_name(
                name.as_ref().to_owned(),
                PersonId(payload.id),
            ))
        }
        PeopleSort::CreatedAtDesc => {
            let created_at = DateTime::parse_from_rfc3339(&payload.k)
                .map_err(|_| invalid_field("/cursor/k"))?
                .with_timezone(&Utc);
            Ok(PeopleCursor::created_at(created_at, PersonId(payload.id)))
        }
    }
}

fn encode_cursor(sort: PeopleSort, person: &DirectoryPersonSummary) -> Result<String, AppError> {
    let key = match sort {
        PeopleSort::DisplayName => person.display_name.clone(),
        PeopleSort::CreatedAtDesc => person.created_at.to_rfc3339(),
    };
    let payload = CursorPayload {
        k: key,
        id: person.person_id.0,
    };
    let json = serde_json::to_vec(&payload).map_err(|error| AppError::Internal(error.into()))?;
    Ok(URL_SAFE_NO_PAD.encode(json))
}

fn parse_new_person(
    body: CreatePersonBody,
    tenant: &TenantInfo,
    now: DateTime<Utc>,
) -> Result<NewDirectoryPerson, AppError> {
    let context = field_context(tenant, now)?;
    let mut collector = Collector::new(&context);
    let display_name = collector.field("/display_name", Raw::<DisplayName>::new(body.display_name));
    let legal_name = collector.optional("/legal_name", body.legal_name.map(Raw::<LegalName>::new));
    let email = collector.optional("/email", body.email.map(Raw::<Email>::new));
    let phone = collector.optional("/phone", body.phone.map(Raw::<Phone>::new));
    let birth_date = collector.optional("/birth_date", body.birth_date.map(Raw::<BirthDate>::new));
    collector.finish()?;
    let display_name = display_name
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("validated display name was missing")))?;
    let legal_name = legal_name.flatten().map(secret_field).transpose()?;
    let email = email.flatten().map(secret_field).transpose()?;
    let phone = phone.flatten().map(secret_field).transpose()?;
    let birth_date = birth_date.flatten().map(secret_field).transpose()?;
    Ok(NewDirectoryPerson::new(
        display_name,
        legal_name,
        email,
        phone,
        birth_date,
        body.roles,
        body.invite,
    ))
}

fn secret_field<K: FieldKind>(value: K) -> Result<SecretString, AppError> {
    value
        .canonical_json()
        .and_then(|value| value.as_str().map(|text| SecretString::from(text.to_owned())))
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("validated secret field has no canonical string")))
}

fn invalid_field(pointer: &str) -> AppError {
    validation_error(pointer, vec![FieldIssue::new(eq_core::errors::INVALID_FORMAT)])
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
