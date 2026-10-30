//! IDN-10/11: actor-scoped people reads and membership/link transitions.

use chrono::Utc;
use eq_audit::{AuditEvent, Category, record};
use eq_core::auth::Actor;
use eq_core::auth::directory::{CoreIdentityDirectory, DirectoryPersonSummary, PeopleScope};
use eq_core::db::Tx;
use eq_core::error::{AppError, FieldError, UNKNOWN_REFERENCE};
use eq_core::ids::PersonId;
use eq_core::model::Role;
use serde_json::{Map, json};

use crate::domain::people::{RoleRemovalError, validate_guardian_link, validate_role_removal};
use crate::errors::codes::{LAST_OWNER, PRIMARY_EXISTS};
use crate::repo;

/// IDN-10: lists only people Core returned from a tenant- and actor-scoped SQL query.
pub(crate) async fn list_people(
    directory: &dyn CoreIdentityDirectory,
    tx: &mut Tx,
    actor: &Actor,
    role: Option<Role>,
) -> Result<Vec<DirectoryPersonSummary>, AppError> {
    directory
        .people_in_scope(tx, actor.tenant(), actor.person_id(), scope(actor), role, None)
        .await
}

/// IDN-10/AUTH-04: returns a person only when Core's SQL scope includes the requested id.
pub(crate) async fn person_in_scope(
    directory: &dyn CoreIdentityDirectory,
    tx: &mut Tx,
    actor: &Actor,
    person: PersonId,
) -> Result<Option<DirectoryPersonSummary>, AppError> {
    Ok(directory
        .people_in_scope(
            tx,
            actor.tenant(),
            actor.person_id(),
            scope(actor),
            None,
            Some(person),
        )
        .await?
        .into_iter()
        .next())
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
    now: chrono::DateTime<Utc>,
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
