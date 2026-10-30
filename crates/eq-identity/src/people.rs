//! IDN-10/11: actor-scoped people reads and membership/link transitions.

use chrono::{DateTime, Duration, LocalResult, TimeZone, Utc};
use chrono_tz::Tz;
use eq_audit::{AuditEvent, Category, record, record_once_for_actor_entity_window};
use eq_core::auth::Actor;
use eq_core::auth::directory::{
    CoreIdentityDirectory, DirectoryPeoplePage, DirectoryPersonDetail, DirectoryPersonSummary,
    NewDirectoryPerson, PeoplePageQuery, PeopleScope, ScopedPersonDetailQuery,
};
use eq_core::crypto::Keyring;
use eq_core::db::Tx;
use eq_core::error::{AppError, FieldError, UNKNOWN_REFERENCE};
use eq_core::ids::PersonId;
use eq_core::model::Role;
use serde_json::{Map, json};

use crate::domain::people::{RoleRemovalError, validate_guardian_link, validate_role_removal};
use crate::errors::codes::{EMAIL_TAKEN, LAST_OWNER, PRIMARY_EXISTS};
use crate::repo;

const LOCAL_DAY_BOUNDARY_SEARCH_MINUTES: i64 = 1_440; // technical: bound timezone gap search to one full local day.
const ALL_ROLES: [Role; 5] = [
    Role::Owner,
    Role::Admin,
    Role::Instructor,
    Role::Guardian,
    Role::Student,
];

pub(crate) struct PersonDetailRequest<'a> {
    pub person: PersonId,
    pub include_pii: bool,
    pub tenant_timezone: &'a str,
    pub now: DateTime<Utc>,
}

/// IDN-10: lists only people Core returned from a tenant- and actor-scoped SQL query.
pub(crate) async fn list_people(
    directory: &dyn CoreIdentityDirectory,
    tx: &mut Tx,
    actor: &Actor,
    query: &PeoplePageQuery,
) -> Result<DirectoryPeoplePage, AppError> {
    directory
        .people_page_in_scope(tx, actor.tenant(), actor.person_id(), scope(actor), query)
        .await
}

/// IDN-10/AUTH-04/AUD-01: reads scoped detail and records only an actual permitted PII disclosure.
pub(crate) async fn person_detail_in_scope(
    directory: &dyn CoreIdentityDirectory,
    tx: &mut Tx,
    actor: &Actor,
    request: PersonDetailRequest<'_>,
    keyring: &Keyring,
) -> Result<Option<DirectoryPersonDetail>, AppError> {
    let query = ScopedPersonDetailQuery::new(
        actor.tenant(),
        actor.person_id(),
        scope(actor),
        request.person,
        request.include_pii,
    );
    let detail = directory.person_detail_in_scope(tx, &query, keyring).await?;
    if request.include_pii
        && detail
            .as_ref()
            .is_some_and(|value| value.summary.person_id != actor.person_id() && value.has_decrypted_pii())
    {
        let (from, until) = tenant_local_day_window(request.now, request.tenant_timezone)?;
        record_once_for_actor_entity_window(
            tx,
            AuditEvent {
                action: "identity.person.pii_read",
                category: Category::Compliance,
                entity_type: "person",
                entity_id: Some(request.person.0),
                before: None,
                after: None,
                actor: actor.context(),
            },
            request.now,
            from,
            until,
        )
        .await?;
    }
    Ok(detail)
}

/// IDN-10/ERD-02: creates a Core-owned person and audits without including PII in the snapshot.
pub(crate) async fn create_person(
    directory: &dyn CoreIdentityDirectory,
    tx: &mut Tx,
    actor: &Actor,
    input: NewDirectoryPerson,
    keyring: &Keyring,
) -> Result<DirectoryPersonSummary, AppError> {
    actor.require("identity.people.write")?;
    if input
        .roles()
        .iter()
        .any(|role| matches!(role, Role::Owner | Role::Admin))
    {
        actor.require("identity.admins.manage")?;
    }
    let Some(person) = directory
        .create_person(tx, actor.tenant(), input, keyring)
        .await?
    else {
        return Err(AppError::Conflict {
            code: &EMAIL_TAKEN,
            detail: String::from("The email address is already used in this tenant"),
            extra: Map::new(),
        });
    };
    record(
        tx,
        AuditEvent {
            action: "identity.person.created",
            category: Category::Operational,
            entity_type: "person",
            entity_id: Some(person.person_id.0),
            before: None,
            after: Some(
                json!({ "roles": person.roles.iter().map(|role| role_name(*role)).collect::<Vec<_>>() }),
            ),
            actor: actor.context(),
        },
    )
    .await?;
    Ok(person)
}

/// IDN-13/STM-20: deactivates all memberships, protects the last owner and revokes sessions atomically.
pub(crate) async fn deactivate_person(
    directory: &dyn CoreIdentityDirectory,
    tx: &mut Tx,
    actor: &Actor,
    person: PersonId,
    now: DateTime<Utc>,
) -> Result<(), AppError> {
    actor.require("identity.people.write")?;
    let active_owners = directory.lock_active_owners(tx, actor.tenant()).await?;
    if active_owners.contains(&person) {
        validate_role_removal(Role::Owner, &active_owners, person).map_err(map_role_removal_error)?;
    }
    if !directory
        .deactivate_person_memberships(tx, actor.tenant(), person)
        .await?
    {
        return Err(AppError::NotFound { entity: "person" });
    }
    repo::revoke_person_refresh_sessions(tx, actor.tenant(), person, now).await?;
    for role in ALL_ROLES {
        repo::expire_pending_invites(tx, actor.tenant(), person, role, now).await?;
    }
    record(
        tx,
        AuditEvent {
            action: "identity.person.deactivated",
            category: Category::Security,
            entity_type: "person",
            entity_id: Some(person.0),
            before: None,
            after: None,
            actor: actor.context(),
        },
    )
    .await?;
    Ok(())
}

fn tenant_local_day_window(
    now: DateTime<Utc>,
    timezone: &str,
) -> Result<(DateTime<Utc>, DateTime<Utc>), AppError> {
    let timezone = timezone
        .parse::<Tz>()
        .map_err(|_| AppError::Internal(anyhow::anyhow!("tenant timezone is invalid")))?;
    let today = now.with_timezone(&timezone).date_naive();
    let tomorrow = today
        .succ_opt()
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("tenant local date is out of range")))?;
    Ok((
        local_day_start(today, timezone)?,
        local_day_start(tomorrow, timezone)?,
    ))
}

fn local_day_start(date: chrono::NaiveDate, timezone: Tz) -> Result<DateTime<Utc>, AppError> {
    let midnight = date
        .and_hms_opt(0, 0, 0)
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("tenant local day boundary is invalid")))?;
    for minute in 0..=LOCAL_DAY_BOUNDARY_SEARCH_MINUTES {
        let local = midnight + Duration::minutes(minute);
        match timezone.from_local_datetime(&local) {
            LocalResult::Single(value) => return Ok(value.with_timezone(&Utc)),
            LocalResult::Ambiguous(first, second) => {
                return Ok(first.min(second).with_timezone(&Utc));
            }
            LocalResult::None => {}
        }
    }
    Err(AppError::Internal(anyhow::anyhow!(
        "tenant local day boundary is unavailable"
    )))
}

/// STM-20: grants an active role through the Core-owned identity directory and audits atomically.
pub(crate) async fn grant_role(
    directory: &dyn CoreIdentityDirectory,
    tx: &mut Tx,
    actor: &Actor,
    person: PersonId,
    role: Role,
) -> Result<(), AppError> {
    actor.require("identity.roles.write")?;
    if matches!(role, Role::Owner | Role::Admin) {
        actor.require("identity.admins.manage")?;
    }

    if !directory
        .activate_membership(tx, actor.tenant(), person, role)
        .await?
    {
        return Err(AppError::NotFound { entity: "person" });
    }
    record(
        tx,
        AuditEvent {
            action: "identity.role.granted",
            category: Category::Security,
            entity_type: "membership",
            entity_id: Some(person.0),
            before: None,
            after: Some(json!({ "role": role_name(role), "status": "active" })),
            actor: actor.context(),
        },
    )
    .await?;
    Ok(())
}

/// STM-20: removes one role while preserving the active-owner invariant and revoking sessions.
pub(crate) async fn remove_role(
    directory: &dyn CoreIdentityDirectory,
    tx: &mut Tx,
    actor: &Actor,
    person: PersonId,
    role: Role,
    now: DateTime<Utc>,
) -> Result<(), AppError> {
    actor.require("identity.roles.write")?;
    if matches!(role, Role::Owner | Role::Admin) {
        actor.require("identity.admins.manage")?;
    }

    let active_owners = if role == Role::Owner {
        directory.lock_active_owners(tx, actor.tenant()).await?
    } else {
        Vec::new()
    };
    validate_role_removal(role, &active_owners, person).map_err(|error| match error {
        RoleRemovalError::OwnerNotActive => AppError::NotFound { entity: "membership" },
        RoleRemovalError::LastOwner => AppError::Conflict {
            code: &LAST_OWNER,
            detail: String::from("The last active owner cannot be removed"),
            extra: Map::new(),
        },
    })?;

    if !directory
        .deactivate_membership(tx, actor.tenant(), person, role)
        .await?
    {
        return Err(AppError::NotFound { entity: "membership" });
    }
    repo::revoke_person_refresh_sessions(tx, actor.tenant(), person, now).await?;
    repo::expire_pending_invites(tx, actor.tenant(), person, role, now).await?;
    record(
        tx,
        AuditEvent {
            action: "identity.role.revoked",
            category: Category::Security,
            entity_type: "membership",
            entity_id: Some(person.0),
            before: Some(json!({ "role": role_name(role), "status": "active" })),
            after: Some(json!({ "role": role_name(role), "status": "deactivated" })),
            actor: actor.context(),
        },
    )
    .await?;
    Ok(())
}

/// STM-27: creates a guardian link only between active guardian/student roles.
pub(crate) async fn create_guardian_link(
    directory: &dyn CoreIdentityDirectory,
    tx: &mut Tx,
    actor: &Actor,
    guardian: PersonId,
    student: PersonId,
    relationship: Option<String>,
    is_primary: bool,
) -> Result<(), AppError> {
    actor.require("identity.people.write")?;
    let (guardian_is_active, student_is_active) = if guardian == student {
        (false, false)
    } else {
        (
            directory
                .has_active_role(tx, actor.tenant(), guardian, Role::Guardian)
                .await?,
            directory
                .has_active_role(tx, actor.tenant(), student, Role::Student)
                .await?,
        )
    };
    validate_guardian_link(guardian, student, guardian_is_active, student_is_active).map_err(|_| {
        AppError::Validation {
            errors: vec![FieldError {
                pointer: String::from("/student_id"),
                code: UNKNOWN_REFERENCE.to_owned(),
                params: Map::new(),
            }],
        }
    })?;
    if !directory
        .create_guardian_link(tx, actor.tenant(), guardian, student, relationship, is_primary)
        .await?
    {
        return Err(AppError::Conflict {
            code: &PRIMARY_EXISTS,
            detail: String::from("The student already has a primary guardian"),
            extra: Map::new(),
        });
    }
    record(
        tx,
        AuditEvent {
            action: "identity.link.created",
            category: Category::Operational,
            entity_type: "person_link",
            entity_id: Some(student.0),
            before: None,
            after: Some(
                json!({ "guardian_id": guardian.0, "student_id": student.0, "is_primary": is_primary }),
            ),
            actor: actor.context(),
        },
    )
    .await?;
    Ok(())
}

fn scope(actor: &Actor) -> PeopleScope {
    match actor.role() {
        Role::Owner | Role::Admin => PeopleScope::All,
        Role::Guardian => PeopleScope::Guardian,
        Role::Instructor | Role::Student => PeopleScope::SelfOnly,
    }
}

fn map_role_removal_error(error: RoleRemovalError) -> AppError {
    match error {
        RoleRemovalError::OwnerNotActive => AppError::NotFound { entity: "membership" },
        RoleRemovalError::LastOwner => AppError::Conflict {
            code: &LAST_OWNER,
            detail: String::from("The last active owner cannot be removed"),
            extra: Map::new(),
        },
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

#[cfg(test)]
mod tests {
    use super::scope;
    use eq_core::actor::ActorCtx;
    use eq_core::auth::Actor;
    use eq_core::ids::{PersonId, TenantId};
    use eq_core::model::Role;
    use std::collections::BTreeSet;
    use uuid::Uuid;

    #[test]
    fn scope_follows_verified_actor_role() {
        let guardian = Actor::from_verified_token(
            PersonId(Uuid::from_u128(1)),
            TenantId(Uuid::from_u128(2)),
            Role::Guardian,
            Uuid::from_u128(3),
            BTreeSet::new(),
            ActorCtx::anonymous(None, None),
        );
        assert_eq!(scope(&guardian), eq_core::auth::directory::PeopleScope::Guardian);
    }
}
