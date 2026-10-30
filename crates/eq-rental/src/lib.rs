//! Hourly studio rental for Ritma (spec/02 §2).

use eq_audit as _;
use eq_core as _;
use eq_core::actor::ActorRole;
use eq_core::module::PermissionDef;
use eq_identity as _;
use eq_scheduling as _;
use serde_json as _;
use utoipa_axum as _;

static OWNER_ADMIN_INSTRUCTOR_GUARDIAN_STUDENT: &[ActorRole] = &[
    ActorRole::Owner,
    ActorRole::Admin,
    ActorRole::Instructor,
    ActorRole::Guardian,
    ActorRole::Student,
];
static ADMIN_INSTRUCTOR_GUARDIAN_STUDENT: &[ActorRole] = &[
    ActorRole::Admin,
    ActorRole::Instructor,
    ActorRole::Guardian,
    ActorRole::Student,
];
static ADMIN_ONLY: &[ActorRole] = &[ActorRole::Admin];
static RENTAL_PERMISSIONS: [PermissionDef; 4] = [
    PermissionDef {
        key: "rental.reserve",
        roles: ADMIN_INSTRUCTOR_GUARDIAN_STUDENT,
    },
    PermissionDef {
        key: "rental.read",
        roles: OWNER_ADMIN_INSTRUCTOR_GUARDIAN_STUDENT,
    },
    PermissionDef {
        key: "rental.decide",
        roles: ADMIN_ONLY,
    },
    PermissionDef {
        key: "rental.rates.write",
        roles: ADMIN_ONLY,
    },
];

eq_core::impl_module_stub!(
    RentalModule,
    Rental,
    "rental",
    "key-round",
    false,
    &[eq_core::module::ModuleId::Scheduling],
    &[],
    &RENTAL_PERMISSIONS
);
