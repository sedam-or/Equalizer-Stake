//! Student enrollment and lifecycle management for Ritma (spec/02 §2).

pub mod errors;

use eq_audit as _;
use eq_core as _;
use eq_core::actor::ActorRole;
use eq_core::module::PermissionDef;
use eq_identity as _;
use eq_scheduling as _;
use serde_json as _;
use utoipa_axum as _;

static OWNER_ADMIN: &[ActorRole] = &[ActorRole::Owner, ActorRole::Admin];
static ADMIN_ONLY: &[ActorRole] = &[ActorRole::Admin];
static ALL_ROLES: &[ActorRole] = &[
    ActorRole::Owner,
    ActorRole::Admin,
    ActorRole::Instructor,
    ActorRole::Guardian,
    ActorRole::Student,
];
static ADMIN_GUARDIAN_STUDENT: &[ActorRole] = &[ActorRole::Admin, ActorRole::Guardian, ActorRole::Student];
static ENROLLMENT_PERMISSIONS: [PermissionDef; 7] = [
    PermissionDef {
        key: "enrollment.registration.read",
        roles: OWNER_ADMIN,
    },
    PermissionDef {
        key: "enrollment.registration.write",
        roles: ADMIN_ONLY,
    },
    PermissionDef {
        key: "enrollment.registration.read_pii",
        roles: OWNER_ADMIN,
    },
    PermissionDef {
        key: "enrollment.read",
        roles: ALL_ROLES,
    },
    PermissionDef {
        key: "enrollment.write",
        roles: ADMIN_ONLY,
    },
    PermissionDef {
        key: "enrollment.leave.request",
        roles: ADMIN_GUARDIAN_STUDENT,
    },
    PermissionDef {
        key: "enrollment.leave.decide",
        roles: ADMIN_ONLY,
    },
];

eq_core::impl_module_stub!(
    EnrollmentModule,
    Enrollment,
    "enrollment",
    "user-round-plus",
    false,
    &[eq_core::module::ModuleId::Scheduling],
    &[],
    &ENROLLMENT_PERMISSIONS
);
