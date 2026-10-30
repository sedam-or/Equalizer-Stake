//! IDN-11/STM-20/27: pure people, membership and guardian-link invariants.

use eq_core::ids::PersonId;
use eq_core::model::Role;
use thiserror::Error;

/// STM-20: owner membership transition rejected by a domain invariant.
#[derive(Debug, Error, PartialEq, Eq)]
pub(crate) enum RoleRemovalError {
    /// The requested owner membership is not active.
    #[error("owner membership is not active")]
    OwnerNotActive,
    /// Removing the membership would leave no active owner.
    #[error("cannot remove the last active owner")]
    LastOwner,
}

/// STM-27: a guardian link requires distinct people with the required active roles.
#[derive(Debug, Error, PartialEq, Eq)]
#[error("guardian and student roles are required")]
pub(crate) struct InvalidGuardianLink;

/// STM-20: validates removal against the locked active-owner facts.
pub(crate) fn validate_role_removal(
    role: Role,
    active_owners: &[PersonId],
    person: PersonId,
) -> Result<(), RoleRemovalError> {
    if role != Role::Owner {
        return Ok(());
    }
    if !active_owners.contains(&person) {
        return Err(RoleRemovalError::OwnerNotActive);
    }
    if active_owners.len() <= 1 {
        return Err(RoleRemovalError::LastOwner);
    }
    Ok(())
}

/// STM-27: validates distinct active guardian and student role facts.
pub(crate) fn validate_guardian_link(
    guardian: PersonId,
    student: PersonId,
    guardian_is_active: bool,
    student_is_active: bool,
) -> Result<(), InvalidGuardianLink> {
    if guardian == student || !guardian_is_active || !student_is_active {
        return Err(InvalidGuardianLink);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{InvalidGuardianLink, RoleRemovalError, validate_guardian_link, validate_role_removal};
    use eq_core::ids::PersonId;
    use eq_core::model::Role;
    use uuid::Uuid;

    #[test]
    fn last_active_owner_cannot_be_removed() {
        let owner = PersonId(Uuid::from_u128(1));
        assert_eq!(
            validate_role_removal(Role::Owner, &[owner], owner),
            Err(RoleRemovalError::LastOwner)
        );
    }

    #[test]
    fn guardian_link_requires_active_distinct_roles() {
        let guardian = PersonId(Uuid::from_u128(1));
        let student = PersonId(Uuid::from_u128(2));
        assert_eq!(
            validate_guardian_link(guardian, student, true, false),
            Err(InvalidGuardianLink)
        );
        assert_eq!(
            validate_guardian_link(guardian, guardian, true, true),
            Err(InvalidGuardianLink)
        );
    }
}
