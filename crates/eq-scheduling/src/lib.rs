//! Scheduling and conflict-free session booking for Ritma (spec/02 §2).

pub mod errors;

use eq_audit as _;
use eq_core as _;
use eq_core::actor::ActorRole;
use eq_core::module::PermissionDef;
use eq_identity as _;
use serde_json as _;
use utoipa_axum as _;

static ALL_ROLES: &[ActorRole] = &[
    ActorRole::Owner,
    ActorRole::Admin,
    ActorRole::Instructor,
    ActorRole::Guardian,
    ActorRole::Student,
];
static ADMIN_ONLY: &[ActorRole] = &[ActorRole::Admin];
static ADMIN_GUARDIAN_STUDENT: &[ActorRole] = &[ActorRole::Admin, ActorRole::Guardian, ActorRole::Student];
static OWNER_ONLY: &[ActorRole] = &[ActorRole::Owner];
static SCHEDULING_PERMISSIONS: [PermissionDef; 9] = [
    PermissionDef {
        key: "scheduling.catalog.read",
        roles: ALL_ROLES,
    },
    PermissionDef {
        key: "scheduling.catalog.write",
        roles: ADMIN_ONLY,
    },
    PermissionDef {
        key: "scheduling.series.write",
        roles: ADMIN_ONLY,
    },
    PermissionDef {
        key: "scheduling.session.read",
        roles: ALL_ROLES,
    },
    PermissionDef {
        key: "scheduling.session.write",
        roles: ADMIN_ONLY,
    },
    PermissionDef {
        key: "scheduling.session.cancel",
        roles: ADMIN_GUARDIAN_STUDENT,
    },
    PermissionDef {
        key: "scheduling.session.override_hours",
        roles: ADMIN_ONLY,
    },
    PermissionDef {
        key: "scheduling.session.custom_duration",
        roles: ADMIN_ONLY,
    },
    PermissionDef {
        key: "scheduling.override.revert",
        roles: OWNER_ONLY,
    },
];

eq_core::impl_module_stub!(
    SchedulingModule,
    Scheduling,
    "scheduling",
    "calendar-days",
    true,
    &[],
    &[],
    &SCHEDULING_PERMISSIONS
);
