//! Pure module dependency validation and toggle planning (MOD-03, MOD-06).

use super::{ModuleId, ModuleManifest};
use std::collections::BTreeSet;

/// A single optional module state change.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ToggleChange {
    pub module: ModuleId,
    pub enabled: bool,
}

/// Effective module set and sorted optional-module changes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TogglePlan {
    pub enabled: BTreeSet<ModuleId>,
    pub changes: Vec<ToggleChange>,
}

/// MOD-06: dependency failures are mapped to stable problem codes by the service layer.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ToggleError {
    #[error("module is not in the compiled registry")]
    UnknownModule(ModuleId),
    #[error("module requirements are not satisfied")]
    Dependency {
        module: ModuleId,
        missing: Vec<ModuleId>,
    },
    #[error("enabled modules require this module")]
    RequiredBy {
        module: ModuleId,
        required_by: Vec<ModuleId>,
    },
}

/// MOD-06: computes an optional-module change plan using manifest dependencies only.
pub fn plan_toggle(
    current: &BTreeSet<ModuleId>,
    requested: &BTreeSet<ModuleId>,
    manifests: &[&ModuleManifest],
) -> Result<TogglePlan, ToggleError> {
    let by_id = manifests
        .iter()
        .map(|manifest| (manifest.id, *manifest))
        .collect::<std::collections::HashMap<_, _>>();

    if let Some(module) = requested.iter().find(|module| !by_id.contains_key(module)) {
        return Err(ToggleError::UnknownModule(*module));
    }

    let always_on = manifests
        .iter()
        .filter(|manifest| manifest.always_on)
        .map(|manifest| manifest.id)
        .collect::<BTreeSet<_>>();
    let current_effective = current.union(&always_on).copied().collect::<BTreeSet<_>>();
    let requested_effective = requested.union(&always_on).copied().collect::<BTreeSet<_>>();

    let disabled = current_effective
        .difference(&requested_effective)
        .copied()
        .filter(|module| by_id.get(module).is_some_and(|manifest| !manifest.always_on))
        .collect::<Vec<_>>();

    for module in &disabled {
        let required_by = requested_effective
            .iter()
            .filter_map(|enabled| {
                let manifest = by_id.get(enabled)?;
                let requires_disabled_module = manifest.requires_all.contains(module)
                    || (manifest.requires_any.contains(module)
                        && !manifest
                            .requires_any
                            .iter()
                            .any(|required| requested_effective.contains(required)));
                requires_disabled_module.then_some(*enabled)
            })
            .collect::<BTreeSet<_>>();
        if !required_by.is_empty() {
            return Err(ToggleError::RequiredBy {
                module: *module,
                required_by: required_by.into_iter().collect(),
            });
        }
    }

    for module in &requested_effective {
        let Some(manifest) = by_id.get(module) else {
            continue;
        };
        let missing = missing_requirements(manifest, &requested_effective);
        if !missing.is_empty() {
            return Err(ToggleError::Dependency {
                module: *module,
                missing,
            });
        }
    }

    let changes = manifests
        .iter()
        .filter(|manifest| !manifest.always_on)
        .filter_map(|manifest| {
            let was_enabled = current.contains(&manifest.id);
            let will_be_enabled = requested.contains(&manifest.id);
            (was_enabled != will_be_enabled).then_some(ToggleChange {
                module: manifest.id,
                enabled: will_be_enabled,
            })
        })
        .collect();

    Ok(TogglePlan {
        enabled: requested_effective,
        changes,
    })
}

fn missing_requirements(manifest: &ModuleManifest, enabled: &BTreeSet<ModuleId>) -> Vec<ModuleId> {
    let mut missing = manifest
        .requires_all
        .iter()
        .filter(|required| !enabled.contains(required))
        .copied()
        .collect::<Vec<_>>();

    if !manifest.requires_any.is_empty()
        && !manifest
            .requires_any
            .iter()
            .any(|required| enabled.contains(required))
    {
        missing.extend(manifest.requires_any.iter().copied());
    }

    missing.sort_unstable();
    missing.dedup();
    missing
}

#[cfg(test)]
mod tests {
    use super::{ToggleError, plan_toggle};
    use crate::module::{ModuleId, ModuleManifest};
    use proptest::prelude::*;
    use std::collections::BTreeSet;

    static ALL_ON: &[ModuleId] = &[
        ModuleId::Core,
        ModuleId::Identity,
        ModuleId::Audit,
        ModuleId::Scheduling,
    ];
    static NO_DEPS: &[ModuleId] = &[];
    static REQUIRES_SCHEDULING: &[ModuleId] = &[ModuleId::Scheduling];
    static BILLING_ALTERNATIVES: &[ModuleId] = &[ModuleId::Enrollment, ModuleId::Rental];
    static FEEDBACK_REQUIRES_ENROLLMENT: &[ModuleId] = &[ModuleId::Enrollment];

    static MANIFESTS: [ModuleManifest; 11] = [
        ModuleManifest {
            id: ModuleId::Core,
            name_key: "module.core.name",
            description_key: "module.core.description",
            icon: "settings",
            always_on: true,
            requires_all: NO_DEPS,
            requires_any: NO_DEPS,
            policy_schema_version: 1,
        },
        ModuleManifest {
            id: ModuleId::Identity,
            name_key: "module.identity.name",
            description_key: "module.identity.description",
            icon: "users",
            always_on: true,
            requires_all: NO_DEPS,
            requires_any: NO_DEPS,
            policy_schema_version: 1,
        },
        ModuleManifest {
            id: ModuleId::Audit,
            name_key: "module.audit.name",
            description_key: "module.audit.description",
            icon: "scroll-text",
            always_on: true,
            requires_all: NO_DEPS,
            requires_any: NO_DEPS,
            policy_schema_version: 1,
        },
        ModuleManifest {
            id: ModuleId::Scheduling,
            name_key: "module.scheduling.name",
            description_key: "module.scheduling.description",
            icon: "calendar-days",
            always_on: true,
            requires_all: NO_DEPS,
            requires_any: NO_DEPS,
            policy_schema_version: 1,
        },
        ModuleManifest {
            id: ModuleId::Enrollment,
            name_key: "module.enrollment.name",
            description_key: "module.enrollment.description",
            icon: "user-round-plus",
            always_on: false,
            requires_all: REQUIRES_SCHEDULING,
            requires_any: NO_DEPS,
            policy_schema_version: 1,
        },
        ModuleManifest {
            id: ModuleId::Billing,
            name_key: "module.billing.name",
            description_key: "module.billing.description",
            icon: "receipt",
            always_on: false,
            requires_all: NO_DEPS,
            requires_any: BILLING_ALTERNATIVES,
            policy_schema_version: 1,
        },
        ModuleManifest {
            id: ModuleId::Attendance,
            name_key: "module.attendance.name",
            description_key: "module.attendance.description",
            icon: "clipboard-check",
            always_on: false,
            requires_all: REQUIRES_SCHEDULING,
            requires_any: NO_DEPS,
            policy_schema_version: 1,
        },
        ModuleManifest {
            id: ModuleId::Rental,
            name_key: "module.rental.name",
            description_key: "module.rental.description",
            icon: "key-round",
            always_on: false,
            requires_all: REQUIRES_SCHEDULING,
            requires_any: NO_DEPS,
            policy_schema_version: 1,
        },
        ModuleManifest {
            id: ModuleId::Notify,
            name_key: "module.notify.name",
            description_key: "module.notify.description",
            icon: "bell",
            always_on: false,
            requires_all: NO_DEPS,
            requires_any: NO_DEPS,
            policy_schema_version: 1,
        },
        ModuleManifest {
            id: ModuleId::Reports,
            name_key: "module.reports.name",
            description_key: "module.reports.description",
            icon: "chart-no-axes-column",
            always_on: false,
            requires_all: NO_DEPS,
            requires_any: NO_DEPS,
            policy_schema_version: 1,
        },
        ModuleManifest {
            id: ModuleId::Feedback,
            name_key: "module.feedback.name",
            description_key: "module.feedback.description",
            icon: "message-square-heart",
            always_on: false,
            requires_all: FEEDBACK_REQUIRES_ENROLLMENT,
            requires_any: NO_DEPS,
            policy_schema_version: 1,
        },
    ];

    fn manifests() -> Vec<&'static ModuleManifest> {
        MANIFESTS.iter().collect()
    }

    #[test]
    fn billing_needs_enrollment_or_rental() {
        let result = plan_toggle(
            &BTreeSet::new(),
            &BTreeSet::from([ModuleId::Billing]),
            &manifests(),
        );
        assert_eq!(
            result,
            Err(ToggleError::Dependency {
                module: ModuleId::Billing,
                missing: vec![ModuleId::Enrollment, ModuleId::Rental],
            })
        );
    }

    #[test]
    fn disable_required_module_rejected() {
        let current = BTreeSet::from([ModuleId::Enrollment, ModuleId::Feedback]);
        let requested = BTreeSet::from([ModuleId::Feedback]);
        let result = plan_toggle(&current, &requested, &manifests());
        assert_eq!(
            result,
            Err(ToggleError::RequiredBy {
                module: ModuleId::Enrollment,
                required_by: vec![ModuleId::Feedback],
            })
        );
    }

    #[test]
    fn required_by_only_reports_removed_dependencies() {
        let current = BTreeSet::from([ModuleId::Billing, ModuleId::Rental, ModuleId::Attendance]);
        let requested = BTreeSet::from([ModuleId::Billing]);

        assert_eq!(
            plan_toggle(&current, &requested, &manifests()),
            Err(ToggleError::RequiredBy {
                module: ModuleId::Rental,
                required_by: vec![ModuleId::Billing],
            })
        );
    }

    #[test]
    fn always_on_cannot_be_disabled() {
        let plan = plan_toggle(&BTreeSet::new(), &BTreeSet::new(), &manifests())
            .expect("an empty optional set is valid");
        assert!(ALL_ON.iter().all(|module| plan.enabled.contains(module)));
        assert!(plan.changes.iter().all(|change| !ALL_ON.contains(&change.module)));
    }

    proptest! {
        #[test]
        fn plan_toggle_closure_holds(requested_bits in prop::collection::vec(any::<bool>(), 0..=7)) {
            let optional = [ModuleId::Enrollment, ModuleId::Billing, ModuleId::Attendance, ModuleId::Rental, ModuleId::Notify, ModuleId::Reports, ModuleId::Feedback];
            let requested = optional.iter().zip(requested_bits.iter().cycle()).filter_map(|(module, enabled)| enabled.then_some(*module)).collect::<BTreeSet<_>>();
            if let Ok(plan) = plan_toggle(&BTreeSet::new(), &requested, &manifests()) {
                for module in &plan.enabled {
                    let manifest = MANIFESTS.iter().find(|manifest| manifest.id == *module).expect("manifest exists");
                    prop_assert!(manifest.requires_all.iter().all(|required| plan.enabled.contains(required)));
                    prop_assert!(manifest.requires_any.is_empty() || manifest.requires_any.iter().any(|required| plan.enabled.contains(required)));
                }
            }
        }
    }
}
